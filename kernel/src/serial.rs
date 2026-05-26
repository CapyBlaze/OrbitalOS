use uart_16550::{Config, Uart16550Tty, backend::PioBackend};
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SERIAL1: Mutex<Option<Uart16550Tty<PioBackend>>> = Mutex::new(unsafe {
        Uart16550Tty::new_port(0x3F8, Config::default()).ok()
    });
}


#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        if let Some(ref mut serial) = *SERIAL1.lock() {
            let _ = serial.write_fmt(args);
        }
    });
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}   