//! Safe core for `libobs/util/task.c`.
//!
//! The C ABI shim lives in [`crate::ffi::task`]. The queue owns one worker
//! thread that runs `FnOnce` tasks FIFO; [`TaskQueue::wait`] queues a marker
//! behind the pending tasks and blocks the caller until the worker reaches
//! it.

/// Boxed task, the safe equivalent of `os_task_t` + `void *param`.
pub type Task = Box<dyn FnOnce() + Send + 'static>;

/// `struct os_task_queue`.
pub struct TaskQueue {
    _private: (),
}

impl TaskQueue {
    /// `os_task_queue_create`.
    pub fn new() -> Option<Self> {
        todo!()
    }

    /// `os_task_queue_queue_task`.
    pub fn queue_task(&self, task: Task) -> bool {
        let _ = task;
        todo!()
    }

    /// `os_task_queue_wait`.
    pub fn wait(&self) -> bool {
        todo!()
    }

    /// `os_task_queue_inside`.
    pub fn inside(&self) -> bool {
        todo!()
    }
}

impl Drop for TaskQueue {
    /// `os_task_queue_destroy`.
    fn drop(&mut self) {
        todo!()
    }
}
