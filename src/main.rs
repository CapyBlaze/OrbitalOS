#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]


extern crate alloc;

use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use os::{color_println, task::{Task, executor::Executor}, vga_buffer::{Color, ColorCode}};

entry_point!(kernel_main);


fn kernel_main(boot_info: &'static BootInfo) -> ! {
    color_println!(ColorCode::new(Color::LightCyan, Color::Black), "Hello World!");

    os::init();
    os::allocator::init(boot_info);
    os::task::keyboard::init();

    #[cfg(test)]
    test_main();
    
    let mut executor = Executor::new();
    executor.spawn(Task::new("Keyboard", os::task::keyboard::print_keypresses()));
    executor.spawn(Task::new("Counter", os::task::counter_task()));
    executor.run();
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    color_println!(ColorCode::new(Color::LightRed, Color::Black), "{}", info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}