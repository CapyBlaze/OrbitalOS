use core::sync::atomic::{AtomicI32, AtomicU8, Ordering};

use alloc::{sync::Arc, vec::Vec};
use futures_util::StreamExt;
use spin::Mutex;
use crate::{boot_info, drivers::mouse::MousePacketStream, frame_buffer::{FRAMEBUFFER, LAYER_MANAGER}};



pub static MOUSE_TEXTURE: Mutex<Vec<u8>> = Mutex::new(Vec::new());
pub const MOUSE_WIDTH: usize = 10;
pub const MOUSE_HEIGHT: usize = 17;

pub static MOUSE_X: AtomicI32 = AtomicI32::new(512);
pub static MOUSE_Y: AtomicI32 = AtomicI32::new(382);
pub static MOUSE_BUTTONS: AtomicU8 = AtomicU8::new(0);

static mut MOUSE_LAYER_ID: u64 = 0;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    Single,
    Double,
}

pub struct ClickZone {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub layer_id: u64,
    pub click_type: ClickType,
    pub action: Arc<dyn Fn() + Send + Sync>,
}

static LAST_CLICK_TIME: Mutex<u64> = Mutex::new(0);
static LAST_CLICK_X: AtomicI32 = AtomicI32::new(0);
static LAST_CLICK_Y: AtomicI32 = AtomicI32::new(0);

const DOUBLE_CLICK_THRESHOLD_MS: u64 = 300;
const DOUBLE_CLICK_SPATIAL_TOLERANCE: i32 = 5;


pub static CLICK_REGISTRY: Mutex<Vec<ClickZone>> = Mutex::new(Vec::new());

pub fn register_click_zone<F>(x: i32, y: i32, width: i32, height: i32, layer_id: u64, click_type: ClickType, action: F)
where F: Fn() + Send + Sync + 'static { 
    let mut registry = CLICK_REGISTRY.lock();
    registry.push(ClickZone {
        x,
        y,
        width,
        height,
        layer_id,
        click_type,
        action: Arc::new(action),
    });
}

pub fn unregister_click_zones_for_layer(layer_id: u64) {
    let mut registry = CLICK_REGISTRY.lock();
    registry.retain(|zone| zone.layer_id != layer_id);
}

pub fn clear_click_zones() {
    let mut registry = CLICK_REGISTRY.lock();
    registry.clear();
}


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
    let mut last_buttons = 0u8;

    while let Some(packet) = stream.next().await {
        update_mouse_position(packet);

        let mouse_x = MOUSE_X.load(Ordering::Relaxed);
        let mouse_y = MOUSE_Y.load(Ordering::Relaxed);
        let buttons = MOUSE_BUTTONS.load(Ordering::Relaxed);

        update_mouse_icon();


        let left_pressed = (buttons & 0x01) != 0;
        let left_clicked_pressed = left_pressed && (last_buttons & 0x01) == 0;
        let left_released = !left_pressed && (last_buttons & 0x01) != 0;
        last_buttons = buttons;

        if left_released {
            *DRAGGED_LAYER_ID.lock() = None;
        }

        let mut current_drag = None;
        {
            if let Some(id) = *DRAGGED_LAYER_ID.lock() {
                let off_x = DRAG_OFFSET_X.load(Ordering::Relaxed);
                let off_y = DRAG_OFFSET_Y.load(Ordering::Relaxed);
                current_drag = Some((id, off_x, off_y));
            }
        }

        if let Some((id, off_x, off_y)) = current_drag {
            let new_x = (mouse_x - off_x).max(0) as usize;
            let new_y = (mouse_y - off_y).max(0) as usize;

            let mut old_x = 0;
            let mut old_y = 0;

            {
                let mut manager = LAYER_MANAGER.lock();
                if let Some(layer) = manager.get_layer_mut(id) {
                    old_x = layer.x;
                    old_y = layer.y;
                    
                    layer.x = new_x;
                    layer.y = new_y;
                }
            }

            let delta_x = new_x as i32 - old_x as i32;
            let delta_y = new_y as i32 - old_y as i32;

            {
                let mut registry = CLICK_REGISTRY.lock();
                for zone in registry.iter_mut() {
                    if zone.layer_id == id {
                        zone.x += delta_x;
                        zone.y += delta_y;
                    }
                }
            }

        } else if left_clicked_pressed {
            let mut action_to_execute = None;

            let current_time = crate::task::TICKS.load(Ordering::Relaxed); 
            
            let mut last_time = LAST_CLICK_TIME.lock();
            let last_x = LAST_CLICK_X.load(Ordering::Relaxed);
            let last_y = LAST_CLICK_Y.load(Ordering::Relaxed);

            let time_delta = current_time.saturating_sub(*last_time);
            let space_delta_x = (mouse_x - last_x).abs();
            let space_delta_y = (mouse_y - last_y).abs();

            let detected_click_type = if time_delta < DOUBLE_CLICK_THRESHOLD_MS 
                && space_delta_x < DOUBLE_CLICK_SPATIAL_TOLERANCE 
                && space_delta_y < DOUBLE_CLICK_SPATIAL_TOLERANCE 
            {
                *last_time = 0; 
                ClickType::Double

            } else {
                *last_time = current_time;
                LAST_CLICK_X.store(mouse_x, Ordering::Relaxed);
                LAST_CLICK_Y.store(mouse_y, Ordering::Relaxed);
                ClickType::Single
            };


            let top_layer_id = {
                let manager = LAYER_MANAGER.lock();
                let mouse_layer_id = unsafe { MOUSE_LAYER_ID };
                let mut highest_z = 0;
                let mut found_id = None;

                for layer in manager.layers.iter() {
                    if layer.id == mouse_layer_id {
                        continue;
                    }
                    
                    if mouse_x >= layer.x as i32 && mouse_x < (layer.x + layer.width) as i32 &&
                       mouse_y >= layer.y as i32 && mouse_y < (layer.y + layer.height) as i32 
                    {
                        if found_id.is_none() || layer.z_index > highest_z {
                            highest_z = layer.z_index;
                            found_id = Some(layer.id);
                        }
                    }
                }
                found_id
            };

            {
                let registry = CLICK_REGISTRY.lock();
                for zone in registry.iter().rev() {
                    if zone.x <= mouse_x && mouse_x < zone.x + zone.width &&
                       zone.y <= mouse_y && mouse_y < zone.y + zone.height 
                    {
                        if Some(zone.layer_id) == top_layer_id && zone.click_type == detected_click_type {
                            action_to_execute = Some(zone.action.clone());
                            break;
                        }
                    }
                }
            }

            if let Some(action) = action_to_execute {
                if let Some(layer_id) = top_layer_id {
                    LAYER_MANAGER.lock().bring_to_front(layer_id);
                }
                action();
            }
        }
    }
}






pub static DRAGGED_LAYER_ID: Mutex<Option<u64>> = Mutex::new(None);
pub static DRAG_OFFSET_X: AtomicI32 = AtomicI32::new(0);
pub static DRAG_OFFSET_Y: AtomicI32 = AtomicI32::new(0);

pub fn start_drag(layer_id: u64) {
    let mouse_x = MOUSE_X.load(Ordering::Relaxed);
    let mouse_y = MOUSE_Y.load(Ordering::Relaxed);

    let mut layer_pos = None;

    {
        let mut manager = LAYER_MANAGER.lock();
        if let Some(layer) = manager.get_layer_mut(layer_id) {
            layer_pos = Some((layer.x as i32, layer.y as i32));
        }
    }

    if let Some((lx, ly)) = layer_pos {
        DRAG_OFFSET_X.store(mouse_x - lx, Ordering::Relaxed);
        DRAG_OFFSET_Y.store(mouse_y - ly, Ordering::Relaxed);
        *DRAGGED_LAYER_ID.lock() = Some(layer_id);
    }
}