use alloc::{collections::VecDeque, sync::Arc};
use spin::Mutex;
use core::{future::Future, pin::Pin, task::{Context, Poll, Waker}};

struct ChannelInner<T> {
    queue: VecDeque<T>,
    waker: Option<Waker>,
}

pub struct Sender<T> {
    inner: Arc<Mutex<ChannelInner<T>>>,
}

pub struct Receiver<T> {
    inner: Arc<Mutex<ChannelInner<T>>>,
}

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(
        Mutex::new(
            ChannelInner {
                queue: VecDeque::new(),
                waker: None,
            }
        )
    );

    (
        Sender {
            inner: inner.clone(),
        },

        Receiver {
            inner,
        },
    )
}


impl<T> Sender<T> {
    pub fn send(&self, value: T) {

        let mut inner =
            self.inner.lock();

        inner.queue.push_back(value);

        if let Some(waker) =
            inner.waker.take()
        {
            waker.wake();
        }
    }
}

pub struct RecvFuture<T> {
    inner: Arc<Mutex<ChannelInner<T>>>,
}

impl<T> Future for RecvFuture<T> {
    type Output = T;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<T> {

        let mut inner =
            self.inner.lock();

        if let Some(value) =
            inner.queue.pop_front()
        {
            Poll::Ready(value)

        } else {

            inner.waker =
                Some(cx.waker().clone());

            Poll::Pending
        }
    }
}

impl<T> Receiver<T> {
    pub fn recv(&self) -> RecvFuture<T> {
        RecvFuture {
            inner: self.inner.clone(),
        }
    }
}
