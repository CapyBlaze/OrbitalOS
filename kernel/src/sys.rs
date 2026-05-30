use x86_64::instructions::port::Port;

pub fn shutdown() -> ! {
    unsafe {
        let mut port = Port::new(0x604);
        port.write(0x2000u16);
    }

    loop {
        x86_64::instructions::hlt();
    }
}