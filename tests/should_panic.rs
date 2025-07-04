#![no_std]
#![no_main]

use core::panic::PanicInfo;
use sparr_os::{QemuExitCode, exit_qemu, serial_println};

#[panic_handler]
fn panic(_info: &PanicInfo) -> !{
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop{};
}

#[unsafe(no_mangle)] 
pub extern "C" fn _start() -> ! {
    should_fail();
    serial_println!("[test did not fail]");
    exit_qemu(QemuExitCode::Failed);
    sparr_os::hlt_loop();
}

fn should_fail(){
    serial_println!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

