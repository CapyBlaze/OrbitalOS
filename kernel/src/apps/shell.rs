use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use alloc::{string::String, vec::Vec};
use futures_util::stream::StreamExt;
use crate::{apps::AppInfo, drivers::keyboard::ScancodeStream, frame_buffer::{self, ColorRGB, FontName}};


#[derive(Clone)]
pub struct TextScreen {
    pub buffer: &'static str,
    pub color: ColorRGB,
}

pub async fn shell(app_info: AppInfo) {
    let mut stream = ScancodeStream::new();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Azerty,
        HandleControl::Ignore,
    );


    let mut buffer = String::new();
    let mut screens: Vec<TextScreen> = Vec::new();
    let mut cursor: usize = 0;

    screens.push(TextScreen {
        buffer: "Welcome to the Orbital OS Shell!",
        color: ColorRGB::new(0xf8, 0xf8, 0xf8),
    });

    screens.push(TextScreen {
        buffer: "Shell commands will be available soon",
        color: ColorRGB::new(0xf8, 0xf8, 0xf8),
    });

    screens.push(TextScreen {
        buffer: "",
        color: ColorRGB::new(0x00, 0x00, 0x00),
    });


    if let Some(app_layer_id) = app_info.layer_id {
        redraw_terminal(&screens, &buffer, &app_info, app_layer_id);

        while let Some(scancode) = stream.next().await {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(c) => {
                            match c {
                                '\x08' => {
                                    if cursor > 0 {
                                        cursor -= 1;
                                        buffer.remove(cursor);
                                        redraw_terminal(&screens, &buffer, &app_info, app_layer_id);
                                    }
                                }
                                '\n' => {
                                    buffer.clear();
                                    cursor = 0;
                                    redraw_terminal(&screens, &buffer, &app_info, app_layer_id);
                                }

                                _ => {
                                    buffer.insert(cursor, c);
                                    cursor += 1;
                                    redraw_terminal(&screens, &buffer, &app_info, app_layer_id);
                                }
                            }
                        }
    
                        DecodedKey::RawKey(_keycode) => { }
                    }
                }
            }
        }
    }
}


fn redraw_terminal(screens: &Vec<TextScreen>, buffer: &String, app_info: &AppInfo, layer_id: u64) {
    let mut x = 10;
    let mut y = 34;

    let mut manager = frame_buffer::LAYER_MANAGER.lock();
    if let Some(layer) = manager.get_layer_mut(layer_id) {
        layer.draw_rect(
            4, 24, 
            app_info.window_width, app_info.window_height, 
            ColorRGB::new(0x1a, 0x1a, 0x1a)
        );

        for screen in screens {
            layer.text_draw(
                x,
                y,
                screen.buffer,
                FontName::SpleenSmall,
                screen.color,
                ColorRGB::new(0x1a, 0x1a, 0x1a),
            );
    
            y += 16;
        }

        layer.text_draw(
            x,
            y,
            "> ",
            FontName::SpleenSmall,
            ColorRGB::new(0xb4, 0x65, 0xe0),
            ColorRGB::new(0x1a, 0x1a, 0x1a),
        );
        x += 16;

    
        for c in buffer.chars() {
            let mut tmp = [0; 4];
            let s = c.encode_utf8(&mut tmp);
    
            layer.text_draw(
                x,
                y,
                s,
                FontName::SpleenSmall,
                ColorRGB::new(0xb4, 0x65, 0xe0),
                ColorRGB::new(0x1a, 0x1a, 0x1a),
            );
    
            x += 8;
    
            if x > app_info.window_width - 16 {
                x = 10 + 16;
                y += 16;
            }
        }
    }
}