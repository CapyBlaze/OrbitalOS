#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

pub mod serial;
pub mod interrupts;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod task;
pub mod boot_info;
pub mod frame_buffer;
pub mod apps {
    pub mod badapple;
    pub mod shell;
    pub mod hud;
}
pub mod drivers {
    pub mod rtc;
    pub mod ata;
}
pub mod generated {
    pub mod layout;
}

use core::{panic::PanicInfo};
use crate::task::keyboard;

#[cfg(test)]
use crate::memory::MemoryRegion;



#[cfg(test)]
#[no_mangle]
pub extern "C" fn _start(
    _vbe_info: *const u32,
    memory_map: *const MemoryRegion,
    memory_map_len: usize,
) -> ! {
    init();
    
    let memory_regions = unsafe {
        core::slice::from_raw_parts(memory_map, memory_map_len)
    };

    crate::allocator::init(memory_regions);
    
    test_main();
    hlt_loop()
}



pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}



pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where T: Fn() {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop()
}



#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}