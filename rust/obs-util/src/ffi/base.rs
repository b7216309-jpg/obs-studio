//! C ABI shim for the non-variadic half of `libobs/util/base.h`.
//!
//! `blog`, `blogva`, and `bcrash` are defined in `libobs/util/base-variadic.c`.
//! Handler updates take a mutex and drop it before any handler runs, so a
//! handler may call back into `blog`. C assigned the pointers with plain
//! stores. Single-threaded results match; concurrent updates do not tear.

use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Mutex;

use crate::base::{CRASH_HANDLER_ALREADY_SET, LOG_WARNING, claim_once};

unsafe extern "C" {
    fn base_default_log_handler() -> *mut c_void;
    fn base_log_literal(handler: *mut c_void, param: *mut c_void, level: c_int, msg: *const c_char);
}

struct State {
    log_handler: *mut c_void,
    log_is_default: bool,
    log_param: *mut c_void,
    crash_handler: *mut c_void,
    crash_installed: bool,
    crash_param: *mut c_void,
    crash_locked: bool,
    crashing: bool,
}

// The pointers are shared with C the same way `base.c` shared them: any
// thread may log. The mutex orders updates; it does not make the param
// itself safe to free while a handler still holds it.
unsafe impl Send for State {}

impl State {
    const fn new() -> Self {
        Self {
            log_handler: ptr::null_mut(),
            log_is_default: true,
            log_param: ptr::null_mut(),
            crash_handler: ptr::null_mut(),
            crash_installed: false,
            crash_param: ptr::null_mut(),
            crash_locked: false,
            crashing: false,
        }
    }
}

static STATE: Mutex<State> = Mutex::new(State::new());

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut guard = STATE.lock().unwrap_or_else(|err| err.into_inner());
    f(&mut guard)
}

fn resolved_log() -> (*mut c_void, *mut c_void) {
    let (handler, param, is_default) =
        with_state(|state| (state.log_handler, state.log_param, state.log_is_default));
    let handler = if is_default {
        // SAFETY: `base_default_log_handler` returns the process-lifetime
        // default and does not call back into this module.
        unsafe { base_default_log_handler() }
    } else {
        handler
    };
    (handler, param)
}

/// # Safety
///
/// `handler` and `param` are the C out-pointers. Either may be null. When
/// non-null they must point to writable pointer-sized slots.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn base_get_log_handler(handler: *mut *mut c_void, param: *mut *mut c_void) {
    if handler.is_null() && param.is_null() {
        return;
    }
    let (current, current_param) = resolved_log();
    if !handler.is_null() {
        // SAFETY: caller guarantees `handler` is writable.
        unsafe { *handler = current };
    }
    if !param.is_null() {
        // SAFETY: caller guarantees `param` is writable.
        unsafe { *param = current_param };
    }
}

/// # Safety
///
/// `handler` is a `log_handler_t` bit-pattern, or null to restore the
/// default. `param` is an opaque value stored and later passed back to the
/// handler. Neither is dereferenced here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn base_set_log_handler(handler: *mut c_void, param: *mut c_void) {
    with_state(|state| {
        if handler.is_null() {
            state.log_handler = ptr::null_mut();
            state.log_is_default = true;
        } else {
            state.log_handler = handler;
            state.log_is_default = false;
        }
        state.log_param = param;
    });
}

/// # Safety
///
/// `handler` is a crash-handler function pointer, or null. C stores a null
/// handler as-is on the first call; a later `bcrash` would call it. `param`
/// is opaque and is not dereferenced here.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn base_set_crash_handler(handler: *mut c_void, param: *mut c_void) {
    let rejected = with_state(|state| {
        if !claim_once(&mut state.crash_locked) {
            return true;
        }
        state.crash_installed = true;
        state.crash_handler = handler;
        state.crash_param = param;
        false
    });
    if rejected {
        let (log_handler, log_param) = resolved_log();
        // SAFETY: `log_handler` is the default or a handler previously stored
        // by `base_set_log_handler`. The message has no printf conversions.
        // The mutex is not held, so the handler may re-enter.
        unsafe {
            base_log_literal(
                log_handler,
                log_param,
                LOG_WARNING,
                CRASH_HANDLER_ALREADY_SET.as_ptr(),
            );
        }
    }
}

/// Reports whether `bcrash` is already on the stack.
///
/// Returns 1 if this call is a re-entry. Otherwise returns 0 and writes the
/// handler to invoke. `*use_default` is 1 when the handler is still the C
/// default. When it is 0, `*handler` is the installed pointer and may be
/// null, which is what C did if the first `base_set_crash_handler` passed
/// null.
///
/// # Safety
///
/// Each out-pointer may be null. Non-null slots must be writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn base_begin_crash(
    handler: *mut *mut c_void,
    param: *mut *mut c_void,
    use_default: *mut c_int,
) -> c_int {
    let started = with_state(|state| {
        if !claim_once(&mut state.crashing) {
            return None;
        }
        Some((
            state.crash_installed,
            state.crash_handler,
            state.crash_param,
        ))
    });
    let Some((installed, crash_handler, crash_param)) = started else {
        return 1;
    };
    if !use_default.is_null() {
        // SAFETY: caller guarantees the slot is writable.
        unsafe { *use_default = if installed { 0 } else { 1 } };
    }
    if !handler.is_null() {
        // SAFETY: caller guarantees the slot is writable.
        unsafe {
            *handler = if installed {
                crash_handler
            } else {
                ptr::null_mut()
            }
        };
    }
    if !param.is_null() {
        // SAFETY: caller guarantees the slot is writable.
        unsafe { *param = crash_param };
    }
    0
}

// Keep the hidden entry point in the staticlib even when this crate's Rust
// callers never name it. `bcrash` in the C adapter is the caller.
#[used]
static KEEP_BASE_BEGIN_CRASH: unsafe extern "C" fn(
    *mut *mut c_void,
    *mut *mut c_void,
    *mut c_int,
) -> c_int = base_begin_crash;
