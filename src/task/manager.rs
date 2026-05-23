use alloc::collections::BTreeMap;
use lazy_static::lazy_static;
use spin::Mutex;
use super::{TaskId, TaskState};


#[derive(Clone)]
pub struct TaskInfo {
    pub id: TaskId,
    pub name: &'static str,
    pub state: TaskState,
    pub cpu_ticks: u64,
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}


pub struct TaskManager {
    tasks: BTreeMap<TaskId, TaskInfo>,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
        }
    }

    pub fn add_task(&mut self, info: TaskInfo) {
        self.tasks.insert(info.id, info);
    }

    pub fn remove_task(&mut self, id: TaskId) {
        self.tasks.remove(&id);
    }

    pub fn update_state(&mut self, id: TaskId, state: TaskState) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.state = state;
        }
    }

    pub fn increment_ticks(&mut self, id: TaskId) {
        if let Some(task) = self.tasks.get_mut(&id) {
            task.cpu_ticks += 1;
        }
    }

    pub fn list_tasks(&self) -> impl Iterator<Item = &TaskInfo> {
        self.tasks.values()
    }
}