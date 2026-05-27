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
    

    // Initialize the frame buffer
    unsafe {
        os::frame_buffer::init(vbe_info as *const u8);
    }
    os::frame_buffer::clear(ColorRGB::new(0x00, 0x00, 0x00));
    os::apps::hud::init();
    serial_println!("Boot: frame buffer init done");


    // Initialize the heap allocator
    let memory_regions = unsafe {
        core::slice::from_raw_parts(memory_map, memory_map_len)
    };
    unsafe {
        os::boot_info::init(boot_manifest);
    }
    os::allocator::init(memory_regions);
    serial_println!("Boot: allocator init done");


    // Initialize the keyboard task
    x86_64::instructions::interrupts::without_interrupts(|| {
        os::task::keyboard::init();
        serial_println!("Boot: keyboard init done");
    });


    // Initialize the executor and spawn tasks
    let mut executor = Executor::new();
    serial_println!("Boot: executor created");
    executor.spawn(Task::new("Keyboard", os::task::keyboard::print_keypresses()));
    executor.spawn(Task::new("badapple", os::apps::badapple::bad_apple()));
    serial_println!("Boot: tasks spawned");


    // Enable interrupts and run the executor
    os::interrupts::mask_all_irqs();
    os::interrupts::unmask_timer_and_keyboard();
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
