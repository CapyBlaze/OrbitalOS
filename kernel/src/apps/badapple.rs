use alloc::vec;

use crate::{boot_info, drivers::ata, frame_buffer, serial_println, task::sleep};

pub async fn bad_apple() {
    let Some(entry) = boot_info::find_file("bad_apple.bin") else {
        serial_println!("badapple: payload not found in microfs");
        return;
    };

    let mut header = [0u8; 512];
    ata::read_sectors(entry.start_sector, 1, &mut header);

    let width = u32::from_le_bytes(header[0..4].try_into().unwrap()) as usize;
    let height = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    let frame_count = u32::from_le_bytes(header[8..12].try_into().unwrap()) as usize;

    if width == 0 || height == 0 || frame_count == 0 {
        serial_println!("badapple: invalid header");
        return;
    }

    let frame_size = width * height / 8;
    let frame_disk_sectors = (frame_size + 511) / 512;
    let mut frame_disk_buffer = vec![0u8; frame_disk_sectors * 512];

    for index in 0..frame_count {
        let frame_lba = entry.start_sector + 1 + (index as u32 * frame_disk_sectors as u32);
        ata::read_sectors(frame_lba, frame_disk_sectors as u32, frame_disk_buffer.as_mut_slice());

        serial_println!("badapple: frame {}/{}", index + 1, frame_count);

        frame_buffer::draw_bitmap_1bpp(
            0,
            0,
            width,
            height,
            frame_disk_buffer.as_mut_slice(),
            frame_buffer::ColorRGB::new(0xFF, 0xFF, 0xFF),
            frame_buffer::ColorRGB::new(0x00, 0x00, 0x00),
        );
        sleep::sleep(1).await;
    }
}
