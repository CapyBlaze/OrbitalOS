use conquer_once::spin::Lazy;
use alloc::vec::Vec;
use spin::Mutex;

use crate::{boot_info, frame_buffer::{self, ColorRGB, FontName}};

pub mod badapple;
pub mod shell;
pub mod doom;
pub mod hud;


#[derive(Clone)]
pub struct AppInfo {
    pub name: &'static str,
    pub icon_name: &'static str,
    pub position: (u8, u8),
}

pub static APP_MANAGER: Lazy<Mutex<Vec<AppInfo>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});


pub fn init() {
    let apps = {
        let mut manager = APP_MANAGER.lock();

        manager.push(AppInfo {
            name: "Bad Apple!!",
            icon_name: "bad_apple_icon.bin",
            position: (0, 0),
        });

        manager.push(AppInfo {
            name: "Shell",
            icon_name: "shell_icon.bin",
            position: (1, 0),
        });

        manager.push(AppInfo {
            name: "Doom",
            icon_name: "doom_icon.bin",
            position: (2, 0),
        });
    
        manager.clone()
    };

    for app in apps {
        draw_icon_app(app);
    }
}


fn draw_icon_app(app: AppInfo) {
    let icon_size: usize = 56;
    let spacing: usize = 20;
    let max_chars = 7;


    let icon_x = spacing + (app.position.0 as usize) * (icon_size + spacing);
    let icon_y = spacing + (app.position.1 as usize) * (icon_size + spacing);

    if let Some(bytes) = boot_info::load_file(app.icon_name) {
        if let Some(data) = bytes.get(4..) {
            frame_buffer::image_rgba_draw(icon_x, icon_y, icon_size, icon_size, data);
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

        frame_buffer::text_draw(
            current_x,
            current_y,
            trimmed_line,
            FontName::SpleenSmall,
            ColorRGB::new(0xff, 0xff, 0xff),
            hud::HUD_BACKGROUND
        );
    }
}

pub fn draw_window_app(x_app: usize, y_app: usize, width_app: usize, height_app: usize, title: &str) {
    let border_size = 4;
    let title_bar_height = 20;

    let x_window = x_app - border_size;
    let y_window = y_app - title_bar_height - border_size;
    let width_window = width_app + (border_size * 2);
    let height_window = height_app + title_bar_height + (border_size * 2);

    frame_buffer::draw_rect(x_window, y_window, width_window, height_window, ColorRGB::new(0xd9, 0xd9, 0xd9));

    frame_buffer::draw_rect(x_window, y_window, width_window, 1, ColorRGB::new(0xff, 0xff, 0xff));
    frame_buffer::draw_rect(x_window, y_window, 1, height_window, ColorRGB::new(0xff, 0xff, 0xff));
    frame_buffer::draw_rect(x_window, y_window + height_window - 1, width_window, 1, ColorRGB::new(0x55, 0x55, 0x55));
    frame_buffer::draw_rect(x_window + width_window - 1, y_window, 1, height_window, ColorRGB::new(0x55, 0x55, 0x55));

    let title_x = x_window + border_size;
    let title_y = y_window + border_size;
    let title_w = width_window - (border_size * 2);
    frame_buffer::draw_rect(title_x, title_y, title_w, title_bar_height, ColorRGB::new(0x4a, 0x46, 0x75));

    frame_buffer::text_draw(
        title_x + 6,
        title_y + 2,
        title,
        FontName::SpleenSmall,
        ColorRGB::new(0xff, 0xff, 0xff),
        ColorRGB::new(0x4a, 0x46, 0x75)
    );

    frame_buffer::draw_rect(x_app - 1, y_app - 1, width_app + 2, 1, ColorRGB::new(0x80, 0x80, 0x80));
    frame_buffer::draw_rect(x_app - 1, y_app - 1, 1, height_app + 2, ColorRGB::new(0x80, 0x80, 0x80));
    frame_buffer::draw_rect(x_app - 1, y_app + height_app, width_app + 2, 1, ColorRGB::new(0xff, 0xff, 0xff));
    frame_buffer::draw_rect(x_app + width_app, y_app - 1, 1, height_app + 2, ColorRGB::new(0xff, 0xff, 0xff));
}
