use core::{future::Future, pin::Pin, task::{Context, Poll}};

pub struct YieldNow {
    yielded: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<()> {

        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;

            cx.waker().wake_by_ref();

            Poll::Pending
        }
    }
}

pub fn yield_now() -> YieldNow {
    YieldNow {
        yielded: false,
    }
}