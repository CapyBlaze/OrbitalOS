#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(os::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use os::{
    boot_info::BootManifest,
    frame_buffer::ColorRGB,
    memory::MemoryRegion,
    serial_println,
    task::{Task, executor::Executor},
};



#[no_mangle]
pub extern "C" fn _start(
    vbe_info: *const u32,
    memory_map: *const MemoryRegion,
    memory_map_len: usize,
    boot_manifest: *const BootManifest,
) -> ! {
    x86_64::instructions::interrupts::disable();

    // Initialize the kernel subsystems
    os::init();
    serial_println!("Boot: init");
    

    // Initialize the heap allocator
    let memory_regions = unsafe {
        core::slice::from_raw_parts(memory_map, memory_map_len)
    };
    unsafe {
        os::boot_info::init(boot_manifest);
    }
    os::allocator::init(memory_regions);
    serial_println!("Boot: allocator init done");

    
    // Initialize the frame buffer
    unsafe {
        os::frame_buffer::init(vbe_info as *const u8);
    }
    os::frame_buffer::clear(ColorRGB::new(0x00, 0x00, 0x00));
    os::frame_buffer::init_fonts();
    serial_println!("Boot: frame buffer init done");
    
    // Initialize the HUD OS
    os::apps::hud::init();


    // Initialize mouse and keyboard drivers
    os::drivers::mouse::init();
    os::drivers::keyboard::init();
    os::drivers::pit::init();
    serial_println!("Boot: drivers init done");


    // Initialize mouse task
    os::task::mouse::init();


    // Initialize the executor and spawn tasks
    let mut executor = Executor::new();
    serial_println!("Boot: executor created");
    
    executor.spawn(Task::new("Keyboard", os::task::keyboard::print_keypresses()));
    executor.spawn(Task::new("Mouse", os::task::mouse::print_mouse_packets()));
    executor.spawn(Task::new("Render", os::task::render::render_loop()));
    executor.spawn(Task::new("BadApple", os::apps::badapple::bad_apple()));
    executor.spawn(Task::new("HudTime", os::apps::hud::time_update()));
    serial_println!("Boot: tasks spawned");


    // Enable interrupts and run the executor
    os::interrupts::mask_all_irqs();
    os::interrupts::unmask_timer_keyboard_mouse();
    executor.run();
}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("KERNEL PANIC: {}", info);
    os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    os::test_panic_handler(info)
}
