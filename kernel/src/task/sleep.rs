use core::{future::Future, pin::Pin, sync::atomic::Ordering, task::{Context, Poll, Waker}};
use crate::{task::TICKS};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;



lazy_static! {
    static ref SLEEPERS:
        Mutex<Vec<Sleeper>> =
            Mutex::new(Vec::new());
}

struct Sleeper {
    wake_tick: u64,
    waker: Waker,
}

pub struct Sleep {
    wake_tick: u64,
    resistered: bool,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        let current = TICKS.load(Ordering::Relaxed);

        if current >= self.wake_tick {
            return Poll::Ready(())
        }

        if !self.resistered {
            SLEEPERS.lock().push(
                Sleeper {
                    wake_tick: self.wake_tick,
                    waker: _cx.waker().clone(),
                }
            );

            self.resistered = true;
        }

        Poll::Pending
    }
}

pub fn sleep(ticks: u64) -> Sleep {
    let current = TICKS.load(Ordering::Relaxed);

    Sleep {
        wake_tick: current + ticks,
        resistered: false,
    }
}


pub fn wake_sleeping_tasks() {
    let current = TICKS.load(Ordering::Relaxed);
    let mut sleepers = SLEEPERS.lock();
    let mut i = 0;

    while i < sleepers.len() {
        if current >= sleepers[i].wake_tick {
            sleepers[i]
                .waker
                .wake_by_ref();

            sleepers.remove(i);

        } else {
            i += 1;
        }
    }
}