use crate::{print, println};
use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use crossbeam_queue::ArrayQueue;
use futures_util::{
    stream::{Stream, StreamExt},
    task::AtomicWaker,
};
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};

static WAKER: AtomicWaker = AtomicWaker::new();
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream:: new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Default for ScancodeStream {
    fn default() -> Self {
        Self::new()
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancode = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancode.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode)
            && let Some(key) = keyboard.process_keyevent(key_event)
        {
            match key {
                DecodedKey::Unicode(char) => print!("{char}"),
                DecodedKey::RawKey(key) => print!("{key:?}"),
            }
        }
    }
}
