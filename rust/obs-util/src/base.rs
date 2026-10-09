//! Safe core of `libobs/util/base.c`: who receives a log line, and whether a
//! crash handler may still be installed.
//!
//! `blog` and `bcrash` stay in `libobs/util/base-variadic.c`. Stable Rust
//! cannot define those variadic entry points, and the default handlers are
//! `vsnprintf` / `vfprintf`. This module owns the handler slots. The C file
//! asks the shim for the current pointer and then calls it.

/// Same values as the `LOG_*` enum in `libobs/util/base.h`.
pub const LOG_ERROR: i32 = 100;
pub const LOG_WARNING: i32 = 200;
pub const LOG_INFO: i32 = 300;
pub const LOG_DEBUG: i32 = 400;

/// `vsnprintf` buffer in the C default handler, including the trailing NUL.
pub const LOG_LINE_CAP: usize = 8192;

pub const CRASH_HANDLER_ALREADY_SET: &std::ffi::CStr =
    c"Tried to set a crash handler when one already exists.";

pub const CRASHED_IN_CRASH_HANDLER: &str = "Crashed in the crash handler";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogStream {
    Stdout,
    Stderr,
}

/// Prefix format and stream the C default handler uses for `level`.
///
/// `None` means the C `switch` prints nothing. The format is the fprintf
/// pattern (`"info: %s\n"`), not the user format string.
pub fn default_log_format(level: i32) -> Option<(&'static str, LogStream)> {
    match level {
        LOG_DEBUG => Some(("debug: %s\n", LogStream::Stdout)),
        LOG_INFO => Some(("info: %s\n", LogStream::Stdout)),
        LOG_WARNING => Some(("warning: %s\n", LogStream::Stdout)),
        LOG_ERROR => Some(("error: %s\n", LogStream::Stderr)),
        _ => None,
    }
}

/// First caller gets `true` and the flag stays set. Later callers get `false`.
pub(crate) fn claim_once(flag: &mut bool) -> bool {
    if *flag {
        false
    } else {
        *flag = true;
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashInstall {
    Installed,
    AlreadyInstalled,
}

pub type LogHandler<P> = fn(i32, &str, &P);
pub type CrashHandler<P> = fn(&str, &P);
pub type LogSlot<P> = (Option<LogHandler<P>>, P);
pub type CrashSlot<P> = (Option<CrashHandler<P>>, P);

/// Single-threaded log and crash slots.
///
/// `blog` delivers an already-formatted message. The C shim delivers a
/// printf format and a `va_list` instead; Tier 3 checks that path.
///
/// A `None` log handler is the default. The safe default discards the line.
/// The C default handler prints it. Prefix selection for that print is
/// [`default_log_format`].
pub struct Logger<P> {
    log_handler: Option<LogHandler<P>>,
    log_param: P,
    crash_handler: Option<CrashHandler<P>>,
    crash_param: P,
    crash_locked: bool,
    crashing: bool,
}

impl<P: Clone> Logger<P> {
    pub fn new(param: P) -> Self {
        Self {
            log_handler: None,
            log_param: param.clone(),
            crash_handler: None,
            crash_param: param,
            crash_locked: false,
            crashing: false,
        }
    }

    pub fn set_log_handler(&mut self, handler: Option<LogHandler<P>>, param: P) {
        self.log_handler = handler;
        self.log_param = param;
    }

    /// `None` is the default handler.
    pub fn log_handler(&self) -> LogSlot<P> {
        (self.log_handler, self.log_param.clone())
    }

    pub fn blog(&self, level: i32, message: &str) {
        if let Some(handler) = self.log_handler {
            handler(level, message, &self.log_param);
        }
    }

    pub fn crash_handler(&self) -> CrashSlot<P> {
        (self.crash_handler, self.crash_param.clone())
    }

    pub fn install_crash_handler(&mut self, handler: CrashHandler<P>, param: P) -> CrashInstall {
        if !claim_once(&mut self.crash_locked) {
            self.blog(
                LOG_WARNING,
                CRASH_HANDLER_ALREADY_SET.to_str().expect("ascii warning"),
            );
            return CrashInstall::AlreadyInstalled;
        }
        self.crash_handler = Some(handler);
        self.crash_param = param;
        CrashInstall::Installed
    }

    /// `None` means this call is a re-entrant crash. The C adapter then
    /// writes [`CRASHED_IN_CRASH_HANDLER`] and exits 2. The first call
    /// returns the installed handler (`None` if it is still the default).
    pub fn begin_crash(&mut self) -> Option<CrashSlot<P>> {
        if !claim_once(&mut self.crashing) {
            return None;
        }
        Some((self.crash_handler, self.crash_param.clone()))
    }
}
