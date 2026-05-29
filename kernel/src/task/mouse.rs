use core::sync::atomic::{AtomicI32, AtomicU8, Ordering};

use alloc::vec::Vec;
use futures_util::StreamExt;
use spin::Mutex;
use crate::{boot_info, drivers::mouse::MousePacketStream, frame_buffer::{self, FRAMEBUFFER}, serial_println};



pub static MOUSE_TEXTURE: Mutex<Vec<u8>> = Mutex::new(Vec::new());
pub const MOUSE_WIDTH: usize = 10;
pub const MOUSE_HEIGHT: usize = 17;

pub static MOUSE_X: AtomicI32 = AtomicI32::new(100);
pub static MOUSE_Y: AtomicI32 = AtomicI32::new(100);
pub static MOUSE_BUTTONS: AtomicU8 = AtomicU8::new(0);


// pub struct MouseState  {
//     pub x: i32,
//     pub y: i32,
//     pub width: usize,
//     pub height: usize,
//     pub buttons: u8,
//     pub mouse_data: Vec<u8>,
// }

// pub static MOUSE_POSITION: Mutex<MouseState> = Mutex::new(MouseState {
//     x: 100,
//     y: 100,
//     width: 10,
//     height: 17,
//     buttons: 0,
//     mouse_data: Vec::new()
// });

pub fn init() {
    if let Some(bytes) = boot_info::load_file("mouse_default.bin") {
        let mut texture = MOUSE_TEXTURE.lock();
        *texture = bytes;
    }
}

fn update_mouse_position(packet: [u8; 3]) {
    let flags = packet[0];
    let mut dx = packet[1] as i32;
    let mut dy = packet[2] as i32;

    if flags & 0x10 != 0 { dx |= !0xFF; }
    if flags & 0x20 != 0 { dy |= !0xFF; }
    dy = -dy;


    let (width_screen, height_screen) = {
        let fb = FRAMEBUFFER.lock();
        (fb.width as i32, fb.height as i32)
    };

    let old_x = MOUSE_X.load(Ordering::Relaxed);
    let old_y = MOUSE_Y.load(Ordering::Relaxed);

    let new_x = (old_x + dx).clamp(0, width_screen);
    let new_y = (old_y + dy).clamp(0, height_screen);

    MOUSE_X.store(new_x, Ordering::Relaxed);
    MOUSE_Y.store(new_y, Ordering::Relaxed);
    MOUSE_BUTTONS.store(flags & 0x07, Ordering::Relaxed);
}

pub fn update_mouse_icon() {
    let x = MOUSE_X.load(Ordering::Relaxed) as usize;
    let y = MOUSE_Y.load(Ordering::Relaxed) as usize;
    
    let texture = MOUSE_TEXTURE.lock();
    if !texture.is_empty() {
        // frame_buffer::buffer_image_rgba_draw(
        //     x, y, 
        //     MOUSE_WIDTH, MOUSE_HEIGHT, 
        //     texture.as_slice()
        // );
    }
}




pub async fn print_mouse_packets() {
    let mut stream = MousePacketStream::new();

    while let Some(packet) = stream.next().await {
        update_mouse_position(packet);

        let mouse_x = MOUSE_X.load(Ordering::Relaxed);
        let mouse_y = MOUSE_Y.load(Ordering::Relaxed);
        let buttons = MOUSE_BUTTONS.load(Ordering::Relaxed);

        serial_println!("mouse: left click at ({}, {})", mouse_x, mouse_y);
        if (buttons & 0x01) != 0 {
            let apps_guard = crate::apps::APP_MANAGER.lock();
            for app in apps_guard.iter() {
                let x_offset_start = app.position.0 as i32 * (56 + 20) + 20;
                let x_offset_end = x_offset_start + 56;
                let y_offset_start = app.position.1 as i32 * (56 + 20) + 20;
                let y_offset_end = y_offset_start + 56 + 5 + (16 * 2);

                if x_offset_start <= mouse_x && mouse_x < x_offset_end && 
                   y_offset_start <= mouse_y && mouse_y < y_offset_end 
                {
                    serial_println!("mouse: clicked on app '{}'", app.name);
                }
            }
        }
    }
}
