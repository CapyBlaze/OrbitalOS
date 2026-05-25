#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use os::{
    framebuffer::ColorRGB, task::{Task, executor::Executor} 
};
use x86_64::VirtAddr;


#[no_mangle]
pub extern "C" fn _start(
    vbe_info: *const u32
) -> ! {
    os::init();

    // let phys_offset = VirtAddr::new(physical_memory_offset);
    // os::allocator::init(phys_offset, memory_map);
    
    // os::task::keyboard::init();

    // FRAMEBUFFER
    
    // let mut executor = Executor::new();
    // executor.spawn(Task::new("Keyboard", os::task::keyboard::print_keypresses()));
    // executor.run();



    unsafe {
        core::arch::asm!("cli");
    }

    unsafe {
        os::framebuffer::init(vbe_info as *const u8);
    }

    os::framebuffer::clear(ColorRGB::new(0x00, 0x00, 0x00));
    os::framebuffer::draw_test();

    loop {
        unsafe { 
            core::arch::asm!("hlt") 
        }
    }
}


#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}