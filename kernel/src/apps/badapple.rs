use alloc::{format, vec};

use crate::{apps::{AppInfo}, boot_info, drivers::ata, frame_buffer::{self, ColorRGB, FontName}, serial_println, task::sleep};

pub async fn bad_apple(app_info: AppInfo) {
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
    let (local_x, local_y) = (4, 24);


    for index in 0..frame_count {
        let frame_lba = entry.start_sector + 1 + (index as u32 * frame_disk_sectors as u32);
        ata::read_sectors(frame_lba, frame_disk_sectors as u32, frame_disk_buffer.as_mut_slice());

        {
            let mut manager = frame_buffer::LAYER_MANAGER.lock();
            if let Some(app_layer_id) = app_info.layer_id {
                if let Some(layer) = manager.get_layer_mut(app_layer_id) {
                    layer.draw_bitmap_1bpp(
                        local_x,
                        local_y,
                        width,
                        height,
                        frame_disk_buffer.as_mut_slice(),
                        ColorRGB::new(0xFF, 0xFF, 0xFF),
                        ColorRGB::new(0x00, 0x00, 0x00),
                    );

                    let text_counter = format!("{:04}/{:04}", index + 1, frame_count);
                    layer.text_draw(
                        local_x + 4,
                        local_y + height + 2,
                        text_counter.as_str(), 
                        FontName::SpleenSmall, 
                        ColorRGB::new(0x0a, 0x0a, 0x0a),
                        ColorRGB::new(0xd9, 0xd9, 0xd9),
                    );
                }
            }
        }

        sleep::sleep_ms(1000 / 24).await;
    }
}
