use core::sync::atomic::{AtomicI32, AtomicU8, Ordering};

use alloc::vec::Vec;
use futures_util::StreamExt;
use spin::Mutex;
use crate::{boot_info, drivers::mouse::MousePacketStream, frame_buffer::{FRAMEBUFFER, LAYER_MANAGER}, serial_println};



pub static MOUSE_TEXTURE: Mutex<Vec<u8>> = Mutex::new(Vec::new());
pub const MOUSE_WIDTH: usize = 10;
pub const MOUSE_HEIGHT: usize = 17;

pub static MOUSE_X: AtomicI32 = AtomicI32::new(512);
pub static MOUSE_Y: AtomicI32 = AtomicI32::new(382);
pub static MOUSE_BUTTONS: AtomicU8 = AtomicU8::new(0);

static mut MOUSE_LAYER_ID: u64 = 0;


pub fn init() {
    let id = LAYER_MANAGER.lock().create_layer(
        MOUSE_WIDTH, 
        MOUSE_HEIGHT, 
        MOUSE_X.load(Ordering::Relaxed) as usize,
        MOUSE_Y.load(Ordering::Relaxed) as usize,
        999
    );
    unsafe { MOUSE_LAYER_ID = id; }

    if let Some(bytes) = boot_info::load_file("mouse_default.bin") {
        let mut manager = LAYER_MANAGER.lock();
        if let Some(layer) = manager.get_layer_mut(id) {
            layer.data = bytes;
        }
    }
}

fn update_mouse_position(packet: [u8; 3]) {
    let flags = packet[0];
    let mut dx = packet[1] as i32;
    let mut dy = packet[2] as i32;

    if flags & 0x10 != 0 { dx |= !0xFF; }
    if flags & 0x20 != 0 { dy |= !0xFF; }
    dx = -dx;

    let (width_screen, height_screen) = {
        let fb = FRAMEBUFFER.lock();
        (fb.width as i32, fb.height as i32)
    };

    let old_x = MOUSE_X.load(Ordering::Relaxed);
    let old_y = MOUSE_Y.load(Ordering::Relaxed);

    let new_x = (old_x + dx).clamp(0, width_screen - MOUSE_WIDTH as i32);
    let new_y = (old_y + dy).clamp(0, height_screen - MOUSE_HEIGHT as i32);

    MOUSE_X.store(new_x, Ordering::Relaxed);
    MOUSE_Y.store(new_y, Ordering::Relaxed);
    MOUSE_BUTTONS.store(flags & 0x07, Ordering::Relaxed);
}

pub fn update_mouse_icon() {
    let x = MOUSE_X.load(Ordering::Relaxed) as usize;
    let y = MOUSE_Y.load(Ordering::Relaxed) as usize;

    let mut manager = LAYER_MANAGER.lock();
    if let Some(layer) = manager.get_layer_mut(unsafe { MOUSE_LAYER_ID }) {
        layer.x = x;
        layer.y = y;
    }
}



pub async fn print_mouse_packets() {
    let mut stream = MousePacketStream::new();

    while let Some(packet) = stream.next().await {
        update_mouse_position(packet);

        let mouse_x = MOUSE_X.load(Ordering::Relaxed);
        let mouse_y = MOUSE_Y.load(Ordering::Relaxed);
        let buttons = MOUSE_BUTTONS.load(Ordering::Relaxed);

        update_mouse_icon();

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
