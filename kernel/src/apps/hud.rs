use core::sync::atomic::Ordering;
use alloc::format;
use crate::{apps::{self, APP_MANAGER, AppInfo}, boot_info, drivers, frame_buffer::{self, ColorRGB, FRAMEBUFFER, FontName, Layer}, task::sleep};

pub const HUD_BACKGROUND: ColorRGB = ColorRGB::new(0x0b, 0x0a, 0x1a);
static mut HUD_LAYER_ID: u64 = 0;


pub fn init() {
    let (width, height) = {
        let fb = FRAMEBUFFER.lock();
        (fb.width, fb.height)
    };

    let id = frame_buffer::LAYER_MANAGER.lock().create_layer(width, height, 0, 0, 1);
    unsafe { HUD_LAYER_ID = id; }

    {
        let mut manager = frame_buffer::LAYER_MANAGER.lock();
        if let Some(layer) = manager.get_layer_mut(id) {
            layer.clear(HUD_BACKGROUND);

            layer.draw_rect(0, height - 42, width, 2,  ColorRGB::new(0xff, 0xff, 0xff));
            layer.draw_rect(0, height - 40, width, 40, ColorRGB::new(0x4a, 0x46, 0x75));

            layer.draw_rect(width - 105, height - 35, 100, 30, ColorRGB::new(0x4a, 0x46, 0x75));
            layer.draw_rect(width - 105, height - 35, 98, 2, ColorRGB::new(0x99, 0x99, 0x99));
            layer.draw_rect(width - 105, height - 35, 2, 30, ColorRGB::new(0x99, 0x99, 0x99));
            layer.draw_rect(width - 105, height - 7, 100, 2, ColorRGB::new(0xff, 0xff, 0xff));
            layer.draw_rect(width - 7, height - 35, 2, 28,   ColorRGB::new(0xff, 0xff, 0xff));
        }
    }
    apps::init();

    {
        let manager = APP_MANAGER.lock();
        let apps = manager.clone();

        if let Some(layer) = frame_buffer::LAYER_MANAGER.lock().get_layer_mut(unsafe { HUD_LAYER_ID }) {
            for app in apps.iter() {
                draw_icon_app(layer, app.clone());
            }
        }
    }
}

pub async fn time_update() {
    let mut next_tick = crate::task::TICKS.load(Ordering::Relaxed) + 1000;

    loop {
        let (width, height) = {
            let fb = FRAMEBUFFER.lock();
            (fb.width, fb.height)
        };

        let time = drivers::rtc::read_rtc();
    
        {
            let mut manager = frame_buffer::LAYER_MANAGER.lock();
            if let Some(layer) = manager.get_layer_mut(unsafe { HUD_LAYER_ID }) {
                
                layer.text_draw(
                    width - 70,
                    height - 31,
                    format!("  {:02}:{:02}:{:02}", time.hour, time.minute, time.second).as_str(), 
                    FontName::SpleenSmallSmall, 
                    ColorRGB::new(0xff, 0xff, 0xff), 
                    ColorRGB::new(0x4a, 0x46, 0x75)
                );
                
                layer.text_draw(
                    width - 70,
                    height - 19,
                    format!("{:02}/{:02}/20{:02}", time.day, time.month, time.year).as_str(), 
                    FontName::SpleenSmallSmall, 
                    ColorRGB::new(0xff, 0xff, 0xff), 
                    ColorRGB::new(0x4a, 0x46, 0x75)
                );
            }
        }

        let now = crate::task::TICKS.load(Ordering::Relaxed);
        if now < next_tick {
            sleep::sleep(next_tick - now).await;
        }
        next_tick += 1000;
    }
}



fn draw_icon_app(layer: &mut Layer, app: AppInfo) {
    let icon_size: usize = 56;
    let spacing: usize = 20;
    let max_chars = 7;

    let icon_x = spacing + (app.position.0 as usize) * (icon_size + spacing);
    let icon_y = spacing + (app.position.1 as usize) * (icon_size + spacing);

    if let Some(bytes) = boot_info::load_file(app.icon_name) {
        if let Some(data) = bytes.get(4..) {
            layer.image_rgba_draw(icon_x, icon_y, icon_size, icon_size, data);
        }
    }

    let mut lines = alloc::vec::Vec::new();
    let mut remaining = app.name;

    while !remaining.is_empty() && lines.len() < 2 {
        if remaining.len() <= max_chars {
            lines.push(remaining);
            break;

        } else {
            let mut cut_index = max_chars;
            if let Some(space_pos) = remaining[..max_chars].rfind(' ') {
                if space_pos > 0 {
                    cut_index = space_pos;
                }
            }
            lines.push(&remaining[..cut_index]);

            let next_start = if remaining.as_bytes().get(cut_index) == Some(&b' ') {
                cut_index + 1
            } else {
                cut_index
            };
            remaining = &remaining[next_start..];
        }
    }

    let text_start_y = icon_y + icon_size + 5;

    for (line_index, line_text) in lines.iter().enumerate() {
        let trimmed_line = line_text.trim();
        let text_width = trimmed_line.len() * 8;
        let center_offset = if text_width < icon_size {
            (icon_size - text_width) / 2
        } else {
            0
        };

        let current_x = icon_x + center_offset;
        let current_y = text_start_y + (line_index * 16);

        layer.text_draw(
            current_x,
            current_y,
            trimmed_line,
            FontName::SpleenSmall,
            ColorRGB::new(0xff, 0xff, 0xff),
            HUD_BACKGROUND
        );
    }
}