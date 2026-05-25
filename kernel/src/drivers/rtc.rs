use x86_64::instructions::port::Port;

fn read_rtc_register(reg: u8) -> u8 {
    unsafe {
        let mut index_port = Port::<u8>::new(0x70);
        let mut data_port = Port::<u8>::new(0x71);

        index_port.write(reg);
        data_port.read()
    }
}

fn bcd_to_binary(value: u8) -> u8 {
    ((value >> 4) * 10) + (value & 0x0F)
}


#[derive(Debug)]
pub struct RtcTime {
    pub second: u8,
    pub minute: u8,
    pub hour: u8,
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

pub fn read_rtc() -> RtcTime {
    let second = bcd_to_binary(read_rtc_register(0x00));
    let minute = bcd_to_binary(read_rtc_register(0x02));
    let hour   = bcd_to_binary(read_rtc_register(0x04));

    let day    = bcd_to_binary(read_rtc_register(0x07));
    let month  = bcd_to_binary(read_rtc_register(0x08));
    let year   = bcd_to_binary(read_rtc_register(0x09));

    RtcTime {
        second,
        minute,
        hour,
        day,
        month,
        year,
    }
}