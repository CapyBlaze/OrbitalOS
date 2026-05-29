use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use futures_util::stream::StreamExt;

use crate::{drivers::keyboard::ScancodeStream, serial_println};

pub async fn print_keypresses() {
    let mut stream = ScancodeStream::new();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Azerty,
        HandleControl::Ignore,
    );

    while let Some(scancode) = stream.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(c) => {
                        serial_println!("{}", c);
                    }

                    DecodedKey::RawKey(keycode) => {
                        serial_println!("{:?}", keycode);
                    }
                }
            }
        }
    }
}
