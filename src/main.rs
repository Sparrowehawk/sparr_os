#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sparr_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use sparr_os::task::executor::Executor;
use core::panic::PanicInfo;
use sparr_os::{allocator, println, task::keyboard};

use sparr_os::task::Task;

entry_point!(kernal_main);

fn kernal_main(boot_info: &'static BootInfo) -> ! {
    use sparr_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    sparr_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed"); // I hate american spelling

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    println!("It did not crash!");
    sparr_os::hlt_loop();
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let num = async_number().await;
    println!("async num is: {num}");
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("{}", _info);
    sparr_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    sparr_os::test_panic_handler(info)
}
