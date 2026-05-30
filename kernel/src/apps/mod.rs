use core::{future::Future, pin::Pin};

use conquer_once::spin::Lazy;
use alloc::{boxed::Box, vec::Vec};
use spin::Mutex;

use crate::{frame_buffer::{self, ColorRGB, FRAMEBUFFER, FontName}, task::mouse::ClickType};

pub mod badapple;
pub mod shell;
pub mod doom;
pub mod hud;


pub type AppFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[derive(Clone)]
pub struct AppInfo {
    pub name: &'static str,
    pub icon_name: &'static str,
    pub position: (u8, u8),
    pub window_width: usize,
    pub window_height: usize,
    pub launcher: fn(AppInfo) -> AppFuture,
    pub task_id: Option<u64>,
    pub layer_id: Option<u64>,
}

pub static APP_MANAGER: Lazy<Mutex<Vec<AppInfo>>> = Lazy::new(|| {
    Mutex::new(Vec::new())
});




fn bad_apple_launcher(app_info: AppInfo) -> AppFuture {
    Box::pin(badapple::bad_apple(app_info))
}

fn shell_launcher(app_info: AppInfo) -> AppFuture {
    Box::pin(shell::shell(app_info))
}

fn doom_launcher(app_info: AppInfo) -> AppFuture {
    Box::pin(doom::doom(app_info))
}


pub fn init() {
    {
        let mut manager = APP_MANAGER.lock();

        manager.push(AppInfo {
            name: "Bad Apple!!",
            icon_name: "bad_apple_icon.bin",
            position: (0, 0),
            window_width: 480,
            window_height: 378,
            launcher: bad_apple_launcher,
            task_id: None,
            layer_id: None,
        });

        manager.push(AppInfo {
            name: "Shell",
            icon_name: "shell_icon.bin",
            position: (1, 0),
            window_width: 700,
            window_height: 500,
            launcher: shell_launcher,
            task_id: None,
            layer_id: None,
        });

        manager.push(AppInfo {
            name: "Doom",
            icon_name: "doom_icon.bin",
            position: (2, 0),
            window_width: 320,
            window_height: 200,
            launcher: doom_launcher,
            task_id: None,
            layer_id: None,
        });
    };
}


const BORDER_SIZE: usize = 4;
const TITLE_BAR_HEIGHT: usize = 20;

pub fn draw_window_app(x_app: usize, y_app: usize, width_app: usize, height_app: usize, title: &str) -> u64 {
    let (width_screen, height_screen) = {
        let fb = FRAMEBUFFER.lock();
        (fb.width, fb.height)
    };

    let app_x = (width_screen - width_app) / 2;
    let app_y = (height_screen - height_app) / 2;
    let layer_id = frame_buffer::LAYER_MANAGER.lock().create_layer(
        width_app + BORDER_SIZE * 2, 
        height_app + TITLE_BAR_HEIGHT + BORDER_SIZE * 2, 
        app_x, 
        app_y, 
        10
    );
    
    {
        let mut manager = frame_buffer::LAYER_MANAGER.lock();
        if let Some(layer) = manager.get_layer_mut(layer_id) {
            layer.clear_transparent();
            
            let x_window = x_app.saturating_sub(BORDER_SIZE);
            let y_window = y_app.saturating_sub(TITLE_BAR_HEIGHT + BORDER_SIZE);
        
            let width_window = width_app + (BORDER_SIZE * 2);
            let height_window = height_app + TITLE_BAR_HEIGHT + (BORDER_SIZE * 2);
        
        
            // Window background
            layer.draw_rect(x_window, y_window, width_window, height_window, ColorRGB::new(0xd9, 0xd9, 0xd9));
        
            // Shadow effect
            layer.draw_rect(x_window, y_window, width_window, 1, ColorRGB::new(0xff, 0xff, 0xff));
            layer.draw_rect(x_window, y_window, 1, height_window, ColorRGB::new(0xff, 0xff, 0xff));
            layer.draw_rect(x_window, y_window + height_window - 1, width_window, 1, ColorRGB::new(0x55, 0x55, 0x55));
            layer.draw_rect(x_window + width_window - 1, y_window, 1, height_window, ColorRGB::new(0x55, 0x55, 0x55));
        
            // Title bar
            let title_x = x_window + BORDER_SIZE;
            let title_y = y_window + BORDER_SIZE;
            let title_w = width_window - (BORDER_SIZE * 2);
            layer.draw_rect(title_x, title_y, title_w, TITLE_BAR_HEIGHT, ColorRGB::new(0x4a, 0x46, 0x75));
        
            layer.text_draw(
                title_x + 6,
                title_y + 2,
                title,
                FontName::SpleenSmall,
                ColorRGB::new(0xff, 0xff, 0xff),
                ColorRGB::new(0x4a, 0x46, 0x75)
            );
        
            // Close button
            let close_button_size = TITLE_BAR_HEIGHT - 10;
            let close_button_x = x_window + width_window - BORDER_SIZE - close_button_size - 4;
            let close_button_y = y_window + BORDER_SIZE + 5;
            layer.draw_circle(
                close_button_x, close_button_y, close_button_size, close_button_size, 
                ColorRGB::new(0xd1, 0x1d, 0x27)
            );
        
            // Shadow effect
            layer.draw_rect(x_app - 1, y_app - 1, width_app + 2, 1, ColorRGB::new(0x80, 0x80, 0x80));
            layer.draw_rect(x_app - 1, y_app - 1, 1, height_app + 2, ColorRGB::new(0x80, 0x80, 0x80));
            layer.draw_rect(x_app - 1, y_app + height_app, width_app + 2, 1, ColorRGB::new(0xff, 0xff, 0xff));
            layer.draw_rect(x_app + width_app, y_app - 1, 1, height_app + 2, ColorRGB::new(0xff, 0xff, 0xff));
        
        
        
            let gl_x = layer.x as i32;
            let gl_y = layer.y as i32;
            let layer_id = layer.id;
        
            crate::task::mouse::register_click_zone(
                gl_x + x_window as i32, 
                gl_y + y_window as i32, 
                width_window as i32, 
                height_window as i32,
                layer_id,
                ClickType::Single,
                move || { }
            );
        
            crate::task::mouse::register_click_zone(
                gl_x + title_x as i32, 
                gl_y + title_y as i32,
                title_w as i32, 
                TITLE_BAR_HEIGHT as i32,
                layer_id,
                ClickType::Single,
                move || { 
                    crate::task::mouse::start_drag(layer_id);
                }
            );
        
            crate::task::mouse::register_click_zone(
                gl_x + close_button_x as i32, 
                gl_y + close_button_y as i32, 
                close_button_size as i32, 
                close_button_size as i32, 
                layer_id,
                ClickType::Single,
                move || {
                    crate::frame_buffer::LAYER_MANAGER.lock().remove_layer(layer_id);
                    crate::task::mouse::unregister_click_zones_for_layer(layer_id);
                    crate::task::manager::TASK_MANAGER.lock().kill_tasks_by_layer(layer_id);
                }
            );
        }
    }
    
    layer_id
}