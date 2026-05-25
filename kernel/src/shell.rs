use alloc::string::String;

use crate::{backspace, clear_screen, color_print, drivers, print, println, vga_buffer::{Color, ColorCode}};

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
        color_print!(ColorCode::new(Color::LightGreen, Color::Black), "> ");
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
                    backspace!();
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
                println!("help clear tasks");
            }

            "clear" => {
                clear_screen!();
            }

            "tasks" => {
                let manager = crate::task::manager::TASK_MANAGER.lock();

                println!("ID NAME STATE CPU");
                for task in manager.list_tasks() {
                    println!(
                        "{} {} {:?} {}",
                        task.id.get(),
                        task.name,
                        task.state,
                        task.cpu_ticks
                    );
                }
            }

            "rtc" => {
                let time = drivers::rtc::read_rtc();

                println!(
                    "{:02}:{:02}:{:02}",
                    time.hour,
                    time.minute,
                    time.second
                );

                println!(
                    "{:02}/{:02}/20{:02}",
                    time.day,
                    time.month,
                    time.year
                );
            }

            "" => {}

            command => {
                println!("unknown command: {}", command);
            }
        }
    }
}