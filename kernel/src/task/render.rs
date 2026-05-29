use crate::{frame_buffer, task::{mouse, sleep}};


pub const RENDER_FPS: u64 = 60;

pub async fn render_loop() {
    loop {
        mouse::update_mouse_icon();
        frame_buffer::draw_layers_to_screen();

        sleep::sleep_ms(1000 / RENDER_FPS).await;
    }
}