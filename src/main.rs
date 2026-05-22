#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]


extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use os::println;

entry_point!(kernel_main);


fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World!");

    os::init();
    os::allocator::init(boot_info);
    os::keyboard::init();

    #[cfg(test)]    
    test_main();
    
    os::hlt_loop()
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}