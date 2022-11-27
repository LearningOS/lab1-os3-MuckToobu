
use super::context::TaskContext;
use alloc::boxed::Box;
use crate::config::MAX_SYSCALL_NUM;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    UnInit,
    Ready,
    Running,
    Exited,
}


#[derive(Clone)]
pub struct TaskControlBlock {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub syscall_times: Box<[i32; MAX_SYSCALL_NUM]>,
    pub time: usize,
}

impl TaskControlBlock {
    pub fn new() -> Self {
        TaskControlBlock {
            task_status: TaskStatus::UnInit,
            task_cx: TaskContext::zero_init(),
            syscall_times: Box::new([0; MAX_SYSCALL_NUM]),
            time: 0,
        }
    }
}