use x86_64::instructions::port::Port;

fn read_rtc_register(reg: u8) -> u8 {
    unsafe {
        let mut index_port = Port::<u8>::new(0x70);
        let mut data_port = Port::<u8>::new(0x71);

        index_port.write(reg);
        data_port.read()
    }
}

fn is_update_in_progress() -> bool {
    read_rtc_register(0x0A) & 0x80 != 0
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
    while is_update_in_progress() {}

    let s1 = read_rtc_register(0x00);
    let m1 = read_rtc_register(0x02);
    let h1 = read_rtc_register(0x04);
    let d1 = read_rtc_register(0x07);
    let mo1 = read_rtc_register(0x08);
    let y1 = read_rtc_register(0x09);

    loop {
        while is_update_in_progress() {}

        let s2  = read_rtc_register(0x00);
        let m2  = read_rtc_register(0x02);
        let h2  = read_rtc_register(0x04);
        let d2  = read_rtc_register(0x07);
        let mo2 = read_rtc_register(0x08);
        let y2  = read_rtc_register(0x09);

        if s1 == s2 && m1 == m2 && h1 == h2 && d1 == d2 && mo1 == mo2 && y1 == y2 {
            return RtcTime {
                second: bcd_to_binary(s2),
                minute: bcd_to_binary(m2),
                hour:   bcd_to_binary(h2),
                day:    bcd_to_binary(d2),
                month:  bcd_to_binary(mo2),
                year:   bcd_to_binary(y2),
            };
        }
    }
}