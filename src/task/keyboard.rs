use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use core::{
    pin::Pin,
    task::{Context, Poll},
};

use crate::println;
use crate::shell::Shell;


static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();



pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_ok() {
            WAKER.wake();

        } else {
            println!("WARNING: scancode queue full; dropping keyboard input");
        }

    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub fn init() {
    SCANCODE_QUEUE
        .try_init_once(|| ArrayQueue::new(100))
        .expect("keyboard::init called twice");
}

pub struct ScancodeStream;

impl ScancodeStream {
    pub fn new() -> Self {
        ScancodeStream
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("queue not initialized");

        if let Some(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());

        match queue.pop() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }

            None => Poll::Pending,
        }
    }
}



pub async fn print_keypresses() {
    let mut stream = ScancodeStream::new();

    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Azerty,
        HandleControl::Ignore,
    );

    let mut shell = Shell::new();
    shell.prompt();

    while let Some(scancode) = stream.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(c) => shell.handle_char(c),
                    DecodedKey::RawKey(pc_keyboard::KeyCode::Return) => shell.handle_char('\n'),
                    DecodedKey::RawKey(pc_keyboard::KeyCode::Backspace) => shell.handle_char('\x08'),
                    _ => {}
                }
            }
        }
    }
}