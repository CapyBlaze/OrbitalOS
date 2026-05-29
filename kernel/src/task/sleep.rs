use core::{future::Future, pin::Pin, sync::atomic::Ordering, task::{Context, Poll, Waker}};
use crate::{task::TICKS};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;



lazy_static! {
    static ref SLEEPERS: Mutex<Vec<Sleeper>> = Mutex::new(Vec::new());
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

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let current = TICKS.load(Ordering::Relaxed);

        if current >= self.wake_tick {
            return Poll::Ready(())
        }

        if !self.resistered {
            x86_64::instructions::interrupts::without_interrupts(|| {
                SLEEPERS.lock().push(Sleeper {
                    wake_tick: self.wake_tick,
                    waker: cx.waker().clone(),
                });
            });
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

pub async fn sleep_ms(ms: u64) {
    self::sleep(ms.max(1)).await;
}


pub fn wake_sleeping_tasks() {
    let current = TICKS.load(Ordering::Relaxed);
    let mut sleepers = SLEEPERS.lock();
    
    sleepers.retain(|sleeper| {
        if current >= sleeper.wake_tick {
            sleeper.waker.wake_by_ref();
            false

        } else {
            true
        }
    });
}