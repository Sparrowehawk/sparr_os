#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sparr_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use sparr_os::{allocator, println};
use bootloader::{BootInfo, entry_point};
use alloc::{boxed::Box, vec::Vec, vec, rc::Rc};

entry_point!(kernal_main);


fn kernal_main(boot_info: &'static BootInfo) -> ! {
    use sparr_os::memory::{self, BootInfoFrameAllocator};
    use x86_64::VirtAddr;

    println!("Hello World{}", "!");
    sparr_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper,&mut frame_allocator)
        .expect("heap initialization failed"); // I hate american spelling

    let heap_value = Box::new(41);
    println!("heap value at {heap_value:?}");

    let mut vec = Vec::new();
    for i in 0..500{
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let ref_counted = Rc::new(vec![1, 2, 3]); 
    let cloned_ref = ref_counted.clone();
    println!("current refrence count is {}", Rc::strong_count(&cloned_ref));
    core::mem::drop(ref_counted);
    println!("refrence count is {} now", Rc::strong_count(&cloned_ref));

    #[cfg(test)]
    test_main();

    println!("It did not crash!");
    sparr_os::hlt_loop();
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
