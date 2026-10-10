//! Safe core for `libobs/util/pipe.c` and the POSIX half,
//! `libobs/util/pipe-posix.c` (`libobs/util/pipe-windows.c` stays in C).
//!
//! The C ABI shim lives in [`crate::ffi::pipe`]. `Args` is platform-neutral
//! (a null-terminated argv like the C darray); process spawning in `Pipe`
//! is Unix-only, matching the `#ifndef _WIN32` process tests in
//! `test/cmocka/test_pipe.c`.

use core::ffi::{c_char, c_int};
use std::ffi::CString;

/// Owned, null-terminated argument vector. Mirrors `struct os_process_args`:
/// a list of duplicated strings with a trailing NULL kept valid after every
/// mutation, like the C darray.
pub struct Args {
    args: Vec<CString>,
    /// Cached `argv`: one pointer per arg plus a trailing null, rebuilt on
    /// every mutation so [`Args::argv_ptr`] never dangles.
    ptrs: Vec<*mut c_char>,
}

/// Truncate at the first NUL, mirroring `bstrdup` (which copies with
/// `strlen` semantics and cannot represent bytes past a NUL).
fn truncated(input: &[u8]) -> &[u8] {
    match input.iter().position(|&b| b == 0) {
        Some(idx) => &input[..idx],
        None => input,
    }
}

impl Args {
    /// Mirrors `os_process_args_create`: argv starts as `[executable, NULL]`.
    pub fn new(executable: &[u8]) -> Self {
        todo!()
    }

    /// Insert before the NULL terminator, mirroring `os_process_args_add_arg`.
    pub fn add_arg(&mut self, arg: &[u8]) {
        todo!()
    }

    /// Core half of `os_process_args_add_argf`: stores already-formatted
    /// bytes. Formatting with `vsnprintf` stays in the C wrapper in
    /// `util/pipe-variadic.c` (stable Rust cannot define a C-variadic
    /// function), exactly like `util/base-variadic.c` does for `blog`.
    pub fn add_formatted(&mut self, formatted: &[u8]) {
        todo!()
    }

    /// Argument count without the trailing NULL, mirroring
    /// `os_process_args_get_argc`.
    pub fn argc(&self) -> usize {
        todo!()
    }

    /// The `index`-th argument, or `None` past the end. The trailing NULL
    /// is not an argument; asserting it is a C-ABI property checked in
    /// `tests/pipe_parity.rs`.
    pub fn arg(&self, index: usize) -> Option<&[u8]> {
        todo!()
    }

    /// Null-terminated argv for the spawn call, mirroring
    /// `os_process_args_get_argv`. Valid until the next mutation, like the
    /// C darray buffer.
    pub fn argv_ptr(&self) -> *mut *mut c_char {
        todo!()
    }
}

/// A spawned child with its stdio pipes. Mirrors `struct os_process_pipe`
/// on POSIX: `read_mode` selects `stdout` vs `stdin`, and `stderr` is always
/// captured.
#[cfg(unix)]
pub struct Pipe {
    read_mode: bool,
    child: std::process::Child,
    reader: Option<std::process::ChildStdout>,
    writer: Option<std::process::ChildStdin>,
    err: Option<std::process::ChildStderr>,
}

#[cfg(unix)]
impl Pipe {
    /// Mirrors `os_process_pipe_create`: runs `/bin/sh -c cmd_line`.
    /// A null command is `None`; anything but a leading `r` in `type_`
    /// selects write mode, exactly like `*type == 'r'`.
    pub fn create(cmd_line: Option<&[u8]>, type_: &[u8]) -> Option<Self> {
        todo!()
    }

    /// Mirrors `os_process_pipe_create2`: spawns `args` directly. A null
    /// `args` is `None`; C would dereference it.
    pub fn create2(args: Option<&Args>, type_: &[u8]) -> Option<Self> {
        todo!()
    }

    /// Closes the streams, reaps the child, and returns its exit status with
    /// the C `(int)(char)` cast applied, mirroring
    /// `os_process_pipe_destroy`.
    pub fn destroy(self) -> c_int {
        todo!()
    }

    /// One read from the child's stdout, mirroring the single `fread` in
    /// `os_process_pipe_read`. Zero for a write-mode pipe.
    pub fn read(&mut self, buf: &mut [u8]) -> usize {
        todo!()
    }

    /// One read from the child's stderr, mirroring `os_process_pipe_read_err`.
    /// Unlike [`Pipe::read`] this works on a write-mode pipe too.
    pub fn read_err(&mut self, buf: &mut [u8]) -> usize {
        todo!()
    }

    /// Write until `buf` is consumed or the pipe errors, mirroring the
    /// `fwrite` loop in `os_process_pipe_write`. Zero for a read-mode pipe.
    pub fn write(&mut self, buf: &[u8]) -> usize {
        todo!()
    }
}
