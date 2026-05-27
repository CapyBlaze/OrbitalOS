use crate::frame_buffer::{self, ColorRGB, FRAMEBUFFER};

pub fn init() {
    frame_buffer::clear(ColorRGB::new(0x48, 0x82, 0x83));

    let fb = FRAMEBUFFER.lock();
    let width = fb.width;
    let height = fb.height;

    // bottom bar
    frame_buffer::draw_rect(0, height - 42, width, 2,  ColorRGB::new(0xff, 0xff, 0xff));
    frame_buffer::draw_rect(0, height - 40, width, 40, ColorRGB::new(0xc3, 0xc3, 0xc3));

    // Time
    frame_buffer::draw_rect(width - 105, height - 35, 100, 30, ColorRGB::new(0xd9, 0xd9, 0xd9));

    frame_buffer::draw_rect(width - 105, height - 35, 98, 2, ColorRGB::new(0x99, 0x99, 0x99));
    frame_buffer::draw_rect(width - 105, height - 35, 2, 30, ColorRGB::new(0x99, 0x99, 0x99));
    frame_buffer::draw_rect(width - 105, height - 7, 100, 2, ColorRGB::new(0xff, 0xff, 0xff));
    frame_buffer::draw_rect(width - 7, height - 35, 2, 28,   ColorRGB::new(0xff, 0xff, 0xff));
}