use core::sync::atomic::Ordering;

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use lazy_static::lazy_static;
use crate::{gdt, hlt_loop, serial_println};
use pic8259::ChainedPics;
use spin;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });


#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}


lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

pub fn mask_all_irqs() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        unsafe {
            PICS.lock().write_masks(0xff, 0xff);
        }
    });
}

pub fn unmask_timer_and_keyboard() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        unsafe {
            PICS.lock().write_masks(0xfc, 0xff);
        }
    });
}



extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    crate::task::TICKS.fetch_add(
        1,
        Ordering::Relaxed
    );
    crate::task::sleep::wake_sleeping_tasks();

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode,) {
    use x86_64::registers::control::Cr2;

    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame,) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}


#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3();
}