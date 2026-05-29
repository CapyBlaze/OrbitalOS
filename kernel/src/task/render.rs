use crate::{frame_buffer, task::{mouse, sleep}};


pub const RENDER_FPS: u64 = 24;

pub async fn render_loop() {
    loop {
        frame_buffer::swap_buffers();
        mouse::update_mouse_icon();

        sleep::sleep_ms(1000 / RENDER_FPS).await;
    }
}