use core::ptr;
use x86_64::instructions::port::Port;

const ATA_PRIMARY: u16 = 0x1F0;
const ATA_DRIVE: u16 = 0x1F6;
const ATA_STATUS: u16 = 0x1F7;
const ATA_COMMAND: u16 = 0x1F7;

const ATA_CMD_READ_SECTORS: u8 = 0x20;

fn status_port() -> Port<u8> {
    Port::new(ATA_STATUS)
}

fn data_port() -> Port<u16> {
    Port::new(ATA_PRIMARY)
}

fn write_command(value: u8) {
    unsafe { Port::new(ATA_COMMAND).write(value); }
}

fn write_drive(value: u8) {
    unsafe { Port::new(ATA_DRIVE).write(value); }
}

fn wait_not_busy() {
    let mut status = status_port();
    loop {
        let st = unsafe { status.read() };
        if st & 0x80 == 0 {
            break;
        }
    }
}

fn wait_data_request() {
    let mut status = status_port();
    loop {
        let st = unsafe { status.read() };
        if st & 0x80 == 0 && st & 0x08 != 0 {
            break;
        }
    }
}

fn select_drive_and_lba(lba: u32, sector_count: u8) {
    unsafe {
        Port::new(0x1F1).write(0u8);
        Port::new(0x1F2).write(sector_count);
        Port::new(0x1F3).write((lba & 0xFF) as u8);
        Port::new(0x1F4).write(((lba >> 8) & 0xFF) as u8);
        Port::new(0x1F5).write(((lba >> 16) & 0xFF) as u8);
        write_drive(0xE0 | (((lba >> 24) & 0x0F) as u8));
    }
}

pub fn read_sectors_lba28(lba: u32, count: u8, buffer: &mut [u8]) {
    assert!(buffer.len() >= (count as usize) * 512);
    select_drive_and_lba(lba, count);
    write_command(ATA_CMD_READ_SECTORS);

    let mut data = data_port();
    let mut buffer_ptr = buffer.as_mut_ptr();

    for _ in 0..count {
        wait_not_busy();
        wait_data_request();

        for _ in 0..256 {
            let word = unsafe { data.read() };
            unsafe {
                ptr::write_unaligned(buffer_ptr as *mut u16, word);
                buffer_ptr = buffer_ptr.add(2);
            }
        }
    }
}

pub fn read_sectors(lba: u32, count: u32, buffer: &mut [u8]) {
    assert!(buffer.len() >= (count as usize) * 512);
    let mut remaining = count;
    let mut current_lba = lba;
    let mut buffer_offset = 0;

    while remaining > 0 {
        let chunk = remaining.min(255) as u8;
        let end = buffer_offset + (chunk as usize) * 512;
        read_sectors_lba28(current_lba, chunk, &mut buffer[buffer_offset..end]);
        current_lba += chunk as u32;
        buffer_offset = end;
        remaining -= chunk as u32;
    }
}
