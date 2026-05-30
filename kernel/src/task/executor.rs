use crate::{serial_println, task::{TaskState, manager::{TASK_MANAGER, TaskInfo}}};
use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake, vec::Vec};
use x86_64::instructions::interrupts;
use core::task::{Waker, Context, Poll};
use crossbeam_queue::ArrayQueue;

static NEW_TASKS_QUEUE: spin::Mutex<Vec<Task>> = spin::Mutex::new(Vec::new());

pub fn spawn(task: Task) {
    interrupts::without_interrupts(|| {
        NEW_TASKS_QUEUE.lock().push(task);
    });
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }

    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Arc::new(TaskWaker {
            task_id,
            task_queue,
        }).into()
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}


pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    fn accept_new_tasks(&mut self) {
        let mut global_queue = NEW_TASKS_QUEUE.lock();
        
        while let Some(task) = global_queue.pop() {
            let task_id = task.id;

            interrupts::without_interrupts(|| {
                TASK_MANAGER.lock().add_task(
                    TaskInfo {
                        id: task.id(),
                        name: task.name(),
                        state: task.state(),
                        cpu_ticks: task.cpu_ticks(),
                        layer_id: task.layer_id(),
                    }
                );
            });

            if self.tasks.insert(task.id, task).is_some() {
                panic!("task with same ID already in tasks");
            }

            self.task_queue.push(task_id).expect("queue full");
        }
    }

    pub fn spawn(&mut self, task: Task) {
        interrupts::without_interrupts(|| {
            TASK_MANAGER.lock().add_task(
                TaskInfo {
                    id: task.id(),
                    name: task.name(),
                    state: task.state(),
                    cpu_ticks: task.cpu_ticks(),
                    layer_id: task.layer_id(),
                }
            );
        });


        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }

        self.task_queue
            .push(task_id)
            .expect("queue full");
    }

    fn run_ready_tasks(&mut self) {
        self.accept_new_tasks();

        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            {
                let mut manager = TASK_MANAGER.lock();
                let is_killed = interrupts::without_interrupts(|| {
                    manager.is_killed(task_id)
                });

                if is_killed {
                    interrupts::without_interrupts(|| {
                        manager.remove_task(task_id);
                    });

                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                    continue;
                }
            }

            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| {
                    TaskWaker::new(
                        task_id,
                        task_queue.clone(),
                    )
                });

            let mut context = Context::from_waker(waker);

            task.state = TaskState::Running;
            task.tick();

            
            interrupts::without_interrupts(|| {
                let mut manager = TASK_MANAGER.lock();

                manager.update_state(
                    task_id,
                    TaskState::Running
                );

                manager.increment_ticks(task_id);
            });


            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    task.state = TaskState::Finished;

                    interrupts::without_interrupts(|| {
                        TASK_MANAGER.lock().remove_task(task_id);
                    });

                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }

                Poll::Pending => {
                    task.state = TaskState::Ready;
                    
                    interrupts::without_interrupts(|| {
                        TASK_MANAGER.lock().update_state(
                            task_id,
                            TaskState::Ready
                        );
                    });
                }
            }
        }
    }

    pub fn run(&mut self) -> ! {
        serial_println!("Executor: starting main loop");
    
        x86_64::instructions::interrupts::enable();

        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.task_queue.is_empty() && NEW_TASKS_QUEUE.lock().is_empty() {
            enable_and_hlt();
            
        } else {
            interrupts::enable();
        }
    }
}
