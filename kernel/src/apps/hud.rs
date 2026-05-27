use alloc::format;

use crate::{drivers, frame_buffer::{self, ColorRGB, FRAMEBUFFER, FontName}, task::sleep};

pub fn init() {
    frame_buffer::clear(ColorRGB::new(0x0b, 0x0a, 0x1a));
    // frame_buffer::clear(ColorRGB::new(0x16, 0x15, 0x33));

    let (width, height) = {
        let fb = FRAMEBUFFER.lock();
        (fb.width, fb.height)
    };

    // bottom bar
    frame_buffer::draw_rect(0, height - 42, width, 2,  ColorRGB::new(0xff, 0xff, 0xff));
    frame_buffer::draw_rect(0, height - 40, width, 40, ColorRGB::new(0x4a, 0x46, 0x75));

    // Time
    frame_buffer::draw_rect(width - 105, height - 35, 100, 30, ColorRGB::new(0x4a, 0x46, 0x75));

    frame_buffer::draw_rect(width - 105, height - 35, 98, 2, ColorRGB::new(0x99, 0x99, 0x99));
    frame_buffer::draw_rect(width - 105, height - 35, 2, 30, ColorRGB::new(0x99, 0x99, 0x99));
    frame_buffer::draw_rect(width - 105, height - 7, 100, 2, ColorRGB::new(0xff, 0xff, 0xff));
    frame_buffer::draw_rect(width - 7, height - 35, 2, 28,   ColorRGB::new(0xff, 0xff, 0xff));
}


pub async fn time_update() {
    loop {
        let (width, height) = {
            let fb = FRAMEBUFFER.lock();
            (fb.width, fb.height)
        };

        let time = drivers::rtc::read_rtc();
    
        frame_buffer::text_draw(
            width - 70,
            height - 31,
            format!("  {:02}:{:02}:{:02}", time.hour, time.minute, time.second).as_str(), 
            FontName::SpleenSmallSmall, 
            ColorRGB::new(0xff, 0xff, 0xff), 
            ColorRGB::new(0x4a, 0x46, 0x75)
        );
        
        frame_buffer::text_draw(
            width - 70,
            height - 19,
            format!("{:02}/{:02}/20{:02}", time.day, time.month, time.year).as_str(), 
            FontName::SpleenSmallSmall, 
            ColorRGB::new(0xff, 0xff, 0xff), 
            ColorRGB::new(0x4a, 0x46, 0x75)
        );
    
        sleep::sleep(10).await;
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
