//! C ABI shim for `libobs/util/task.c`.

use core::ffi::c_void;
use core::ptr;

use crate::task::TaskQueue;

/// `os_task_t`. C callers may pass NULL, hence the `Option`.
pub type OsTask = Option<unsafe extern "C" fn(*mut c_void)>;

/// `os_task_queue_t` — opaque in the public header, so no `#[repr(C)]`
/// layout contract applies; the handle is just a boxed `TaskQueue`.
///
/// Raw `param` pointers are not `Send`; the C API explicitly hands
/// ownership of `param` to the task, which runs on the worker thread.
struct SendParam(*mut c_void);

// SAFETY: the C contract for os_task_queue_queue_task transfers `param`
// to the worker thread; callers must ensure it stays valid until the task
// runs (the cmocka test does exactly this).
unsafe impl Send for SendParam {}

impl SendParam {
    /// Called as a method so the closure captures the whole wrapper, not
    /// the raw field (which is not `Send`).
    fn get(self) -> *mut c_void {
        self.0
    }
}

/// # Safety
///
/// Returned pointer is freed by [`os_task_queue_destroy`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_task_queue_create() -> *mut TaskQueue {
    match TaskQueue::new() {
        Some(tq) => Box::into_raw(Box::new(tq)),
        None => ptr::null_mut(),
    }
}

/// # Safety
///
/// `tq` came from [`os_task_queue_create`]. `param` must stay valid until
/// `task` runs, per the C contract.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_task_queue_queue_task(
    tq: *mut TaskQueue,
    task: OsTask,
    param: *mut c_void,
) -> bool {
    if tq.is_null() {
        return false;
    }
    let param = SendParam(param);
    // SAFETY: `tq` is a live `TaskQueue` per the function contract.
    unsafe { &*tq }.queue_task(Box::new(move || match task {
        // SAFETY: `task` is a valid C function pointer and `param` is still
        // live per the caller's contract.
        Some(task) => unsafe { task(param.get()) },
        // C invokes the function pointer unconditionally, so a NULL task
        // crashes there. Abort rather than silently succeeding.
        None => std::process::abort(),
    }))
}

/// # Safety
///
/// `tq` came from [`os_task_queue_create`] or is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_task_queue_destroy(tq: *mut TaskQueue) {
    if tq.is_null() {
        return;
    }
    // SAFETY: `tq` is ours and is destroyed exactly once.
    unsafe { drop(Box::from_raw(tq)) };
}

/// # Safety
///
/// `tq` came from [`os_task_queue_create`] or is null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_task_queue_wait(tq: *mut TaskQueue) -> bool {
    if tq.is_null() {
        return false;
    }
    // SAFETY: `tq` is a live `TaskQueue` per the function contract.
    unsafe { &*tq }.wait()
}

/// # Safety
///
/// `tq` came from [`os_task_queue_create`]. C reads `tq->id` with no NULL
/// check; a null queue therefore has no defined behavior and returns false
/// here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_task_queue_inside(tq: *mut TaskQueue) -> bool {
    if tq.is_null() {
        return false;
    }
    // SAFETY: `tq` is a live `TaskQueue` per the function contract.
    unsafe { &*tq }.inside()
}
