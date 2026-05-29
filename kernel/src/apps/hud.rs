use core::sync::atomic::Ordering;

use alloc::format;
use crate::{apps, drivers, frame_buffer::{self, ColorRGB, FRAMEBUFFER, FontName}, task::sleep};


pub const HUD_BACKGROUND: ColorRGB = ColorRGB::new(0x0b, 0x0a, 0x1a);

pub fn init() {
    frame_buffer::clear(HUD_BACKGROUND);

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

    apps::init();
}


pub async fn time_update() {
    let mut next_tick = crate::task::TICKS.load(Ordering::Relaxed) + 1000;

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
        

        let now = crate::task::TICKS.load(Ordering::Relaxed);

        if now < next_tick {
            sleep::sleep(next_tick - now).await;
        }

        next_tick += 1000;
    }
}