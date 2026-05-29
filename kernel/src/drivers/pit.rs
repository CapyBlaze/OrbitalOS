use x86_64::instructions::port::Port;

pub fn init() {
    unsafe {
        let mut cmd_port = Port::new(0x43);
        let mut data_port = Port::new(0x40);

        cmd_port.write(0x36_u8);

        let divisor: u16 = 1193;

        data_port.write((divisor & 0xFF) as u8);
        data_port.write(((divisor >> 8) & 0xFF) as u8);
    }
}
