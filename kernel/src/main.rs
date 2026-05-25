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
    // os::init();

    // os::serial_println!("Hello World{}", "!");

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

    let vbe = vbe_info as *const u8;

    let pitch  = unsafe { *(vbe.add(16) as *const u16) } as usize;
    let width  = unsafe { *(vbe.add(18) as *const u16) } as usize;
    let height = unsafe { *(vbe.add(20) as *const u16) } as usize;

    let fb_addr = unsafe { *(vbe.add(40) as *const u32) };
    let fb_ptr = fb_addr as *mut u8;

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

    os::framebuffer::clear(ColorRGB::new(0x00, 0x00, 0x00));
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