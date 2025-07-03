#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(sparr_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use sparr_os::{memory::{self}, println};
use bootloader::{BootInfo, entry_point};
use x86_64::structures::paging::Translate;

entry_point!(kernal_main);

fn kernal_main(boot_info: &'static BootInfo) -> !{
    use x86_64::VirtAddr;

    println!("Hello World {}", "!");
    sparr_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe {memory::init(phys_mem_offset)};

    let addresses = [
        // The identity-mapped VGA buffer page
        0xb8000,
        // Some code page
        0x201008,
        // Some stack page
        0x0100_0020_1a10,
        // virtual addres mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{virt:?} -> {phys:?}");
    }

    
    #[cfg(test)]
    test_main();

    println!("DID NOT CRASH !!");
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
