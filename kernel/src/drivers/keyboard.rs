use core::pin::Pin;
use core::task::{Context, Poll};

use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;
use x86_64::instructions::port::Port;

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();


pub fn init() {
    let init_result = SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100));
    if init_result.is_ok() {
        WAKER.wake();
    }

    enable_scanning();
}

fn enable_scanning() {
    unsafe {
        let mut data = Port::new(0x60);

        data.write(0xF4_u8);
    }
}

pub fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            return;
        }
        WAKER.wake();
    }
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
        let queue = match SCANCODE_QUEUE.try_get() {
            Ok(queue) => queue,
            Err(_) => {
                WAKER.register(cx.waker());
                return Poll::Pending;
            }
        };

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