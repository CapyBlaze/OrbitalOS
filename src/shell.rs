use alloc::string::String;

use crate::{color_print, print, println, vga_buffer::{Color, ColorCode}};

pub struct Shell {
    buffer: String,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn prompt(&self) {
        color_print!(ColorCode::new(Color::Green, Color::Black), "> ");
    }

    pub fn handle_char(&mut self, c: char) {
        match c {
            '\n' => {
                println!();

                self.execute();
                self.buffer.clear();
                self.prompt();
            }

            '\x08' => {
                if self.buffer.pop().is_some() {
                    print!("\x08 \x08");
                }
            }

            _ => {
                self.buffer.push(c);
                print!("{}", c);
            }
        }
    }

    fn execute(&self) {
        match self.buffer.trim() {
            "help" => {
                println!("help clear echo");
            }

            "clear" => {
                for _ in 0..50 {
                    println!();
                }
            }

            "" => {}

            command => {
                println!("unknown command: {}", command);
            }
        }
    }
}