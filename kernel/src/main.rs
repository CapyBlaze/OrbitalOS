#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use alloc::boxed::Box;
use os::{
    task::{Task, executor::Executor}, 
};
use x86_64::VirtAddr;


#[no_mangle]
pub extern "C" fn _start(
    _memory_map: *const u8,
    _physical_memory_offset: u64,
    fb_addr: u64,
    fb_width: u32,
    fb_height: u32,
    fb_stride_bytes: u32
) -> ! {
    // os::init();

    // os::serial_println!("Hello World{}", "!");

    // let phys_offset = VirtAddr::new(physical_memory_offset);
    // os::allocator::init(phys_offset, memory_map);
    
    // os::task::keyboard::init();

    // let fb_struct = os::framebuffer::FrameBuffer {
    //     buffer_ptr: fb_addr as *mut u8,
    //     buffer_size: 1024 * 768 * 4,
    //     width: 1024,
    //     height: 768,
    //     stride: 1024,
    // };

    // os::framebuffer::init(Box::leak(Box::new(fb_struct)));
    // os::framebuffer::clear([0, 128, 0, 255]);
    // os::framebuffer::draw_test();
    
    // let mut executor = Executor::new();
    // executor.spawn(Task::new("Keyboard", os::task::keyboard::print_keypresses()));
    // executor.run();


    os::serial_println!("Kernel: _start atteint.");


    static mut FB_STRUCT: os::framebuffer::FrameBuffer = os::framebuffer::FrameBuffer {
        buffer_ptr: core::ptr::null_mut(),
        buffer_size: 0,
        width: 0,
        height: 0,
        stride: 0,
    };

    unsafe {
        FB_STRUCT.buffer_ptr = fb_addr as *mut u8;
        FB_STRUCT.width = fb_width as usize;
        FB_STRUCT.height = fb_height as usize;
        FB_STRUCT.stride = fb_stride_bytes as usize;
        FB_STRUCT.buffer_size = FB_STRUCT.stride * FB_STRUCT.height;
        os::framebuffer::init(&FB_STRUCT);
    }

    // VBE LFB is typically BGRA (little-endian), so red is [0, 0, 255, 0].
    os::framebuffer::clear([0, 0, 255, 0]);


    loop {
        unsafe { core::arch::asm!("hlt") }
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