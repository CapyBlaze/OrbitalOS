use core::{future::Future, pin::Pin, sync::atomic::{AtomicU64, Ordering}, task::{Context, Poll}};
use alloc::{boxed::Box};

pub mod simple_executor;
pub mod keyboard;
pub mod executor;
pub mod yield_now;
pub mod sleep;
pub mod manager;
pub mod channel;


pub static TICKS: AtomicU64 = AtomicU64::new(0);



#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

#[derive(Debug, Clone, Copy)]
pub enum TaskState {
    Ready,
    Running,
    Sleeping,
    Finished,
}

pub struct Task {
    id: TaskId,
    name: &'static str,
    state: TaskState,
    cpu_ticks: u64,
    wake_up_time: Option<u64>,
    future: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl Task {
    pub fn new(name: &'static str, future: impl Future<Output = ()> + Send + 'static) -> Task {
        Task {
            id: TaskId::new(),
            name,
            state: TaskState::Ready,
            cpu_ticks: 0,
            wake_up_time: None,
            future: Box::pin(future),
        }
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }

    pub fn tick(&mut self) {
        self.cpu_ticks += 1;
    }

    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn cpu_ticks(&self) -> u64 {
        self.cpu_ticks
    }

    pub fn wake_up_time(&self) -> Option<u64> {
        self.wake_up_time
    }
}

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}
