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
    vbe_info: *const u32
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


    // os::serial_println!("Kernel: _start atteint.");


    // static mut FB_STRUCT: os::framebuffer::FrameBuffer = os::framebuffer::FrameBuffer {
    //     buffer_ptr: core::ptr::null_mut(),
    //     buffer_size: 0,
    //     width: 0,
    //     height: 0,
    //     stride: 0,
    // };

    // unsafe {
    //     FB_STRUCT.buffer_ptr = fb_addr as *mut u8;
    //     FB_STRUCT.width = fb_width as usize;
    //     FB_STRUCT.height = fb_height as usize;
    //     FB_STRUCT.stride = fb_stride_bytes as usize;
    //     FB_STRUCT.buffer_size = FB_STRUCT.stride * FB_STRUCT.height;
    //     os::framebuffer::init(&FB_STRUCT);
    // }

    // // VBE LFB is typically BGRA (little-endian), so red is [0, 0, 255, 0].
    // os::framebuffer::clear([0, 0, 255, 0]);


    // loop {
    //     unsafe { core::arch::asm!("hlt") }
    // }

    unsafe {
        core::arch::asm!("cli");
    }

    let vbe = vbe_info as *const u8;

    let pitch  = unsafe { *(vbe.add(16) as *const u16) } as usize;
    let width  = unsafe { *(vbe.add(18) as *const u16) } as usize;
    let height = unsafe { *(vbe.add(20) as *const u16) } as usize;

    let fb_addr = unsafe { *(vbe.add(40) as *const u32) };
    let fb_ptr = fb_addr as *mut u8;

    let bpp = unsafe { *(vbe.add(25) as *const u8) };

    os::serial_println!("pitch {}", pitch);
    os::serial_println!("width {}", width);
    os::serial_println!("height {}", height);
    os::serial_println!("bpp {}", bpp);

    static mut FB_STRUCT: os::framebuffer::FrameBuffer = os::framebuffer::FrameBuffer {
        buffer_ptr: core::ptr::null_mut(),
        buffer_size: 0,
        width: 0,
        height: 0,
        stride: 0,
    };

    unsafe {
        FB_STRUCT.buffer_ptr = fb_ptr;
        FB_STRUCT.width = width;
        FB_STRUCT.height = height;
        FB_STRUCT.stride = pitch;
        FB_STRUCT.buffer_size = pitch * height;
        os::framebuffer::init(&FB_STRUCT);
    }

    os::framebuffer::clear(0x0000);
    os::framebuffer::draw_test();

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