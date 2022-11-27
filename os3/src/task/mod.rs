

mod context;
mod switch;
use switch::__switch;
#[allow(clippy::module_inception)]
mod task;
use task::*;
pub use task::TaskStatus;

use context::TaskContext;
use crate::config::{MAX_APP_NUM, MAX_SYSCALL_NUM};
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::timer::get_time_us;
use lazy_static::lazy_static;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    current_task: usize,
    last_time: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        println!("b");
        let mut tasks = Vec::with_capacity(MAX_APP_NUM);
        for _i in 0..MAX_APP_NUM {
            tasks.push(TaskControlBlock::new())
        }
        println!("{}", tasks.len());
        println!("d");
        for i in 0..num_app {
            tasks[i].task_cx = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].task_status = TaskStatus::Ready;
        }
        println!("e");
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                    last_time: 0,
                })
            }
        }
    };
}

impl TaskManager{
    fn run_first_task(&self) -> ! {
        println!("b");
        let mut inner = self.inner.exclusive_access();
        println!("1");
        let task0 = &mut inner.tasks[0];
        println!("1");
        task0.task_status = TaskStatus::Running;
        println!("b");
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        let mut _unused = TaskContext::zero_init();
        inner.last_time = get_time_us(); 
        inner.tasks[0].time = inner.last_time;
        drop(inner);
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr)
        }
        panic!("unreachable in run_first_task!");
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| inner.tasks[*id].task_status == TaskStatus::Ready)
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            //inner.tasks[current].time += get_time_us() - inner.last_time;
            //inner.last_time = get_time_us();
            //inner.tasks[current].task_status = TaskStatus::Ready;
            inner.tasks[next].task_status = TaskStatus::Running;
            if inner.tasks[next].time == 0 {inner.tasks[next].time = get_time_us()}
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe { __switch(current_task_cx_ptr, next_task_cx_ptr)}
        } else {
            panic!("All application completed!");
        }
    }

    fn current_syscall_increase(&self, syscall_id: usize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].syscall_times[syscall_id] += 1;
    }

    fn current_task_info(&self) -> (TaskStatus, Box<[i32; MAX_SYSCALL_NUM]>, usize) {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (
            inner.tasks[current].task_status,
            inner.tasks[current].syscall_times.clone(),
            get_time_us() - inner.tasks[current].time,
        )
    }
    
    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }


}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}
pub fn run_first_task() {
    println!("c");
    TASK_MANAGER.run_first_task();
}
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

pub fn current_syscall_increase(syscall_id: usize) {
    TASK_MANAGER.current_syscall_increase(syscall_id)
}

pub fn current_task_info() -> (TaskStatus, Box<[i32; MAX_SYSCALL_NUM]>, usize) {
    TASK_MANAGER.current_task_info()
}
