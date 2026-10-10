//! C ABI shim for `libobs/util/buffered-file-serializer.c`.

use core::ffi::{CStr, c_char, c_int, c_void};

use super::array_serializer::serializer;
use crate::buffered_file_serializer::{Output, seek_type_from_c};

unsafe extern "C" fn output_write(param: *mut c_void, data: *const c_void, size: usize) -> usize {
    if data.is_null() || size == 0 {
        return 0;
    }
    // SAFETY: `param` is the `Output` installed by init. `data` covers `size`.
    let output = unsafe { &*(param as *const Output) };
    let buf = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) };
    output.write(buf)
}

unsafe extern "C" fn output_seek(param: *mut c_void, offset: i64, seek_type: c_int) -> i64 {
    // SAFETY: `param` is the `Output` installed by init.
    let output = unsafe { &*(param as *const Output) };
    output.seek(offset, seek_type_from_c(seek_type))
}

unsafe extern "C" fn output_pos(param: *mut c_void) -> i64 {
    // SAFETY: `param` is the `Output` installed by init.
    let output = unsafe { &*(param as *const Output) };
    output.pos()
}

fn path_from_c(path: *const c_char) -> Option<std::path::PathBuf> {
    if path.is_null() {
        return None;
    }
    // SAFETY: `path` is a C string when non-null.
    let bytes = unsafe { CStr::from_ptr(path) }.to_bytes();
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(std::path::PathBuf::from(std::ffi::OsStr::from_bytes(bytes)))
    }
    #[cfg(not(unix))]
    {
        std::str::from_utf8(bytes)
            .ok()
            .map(std::path::PathBuf::from)
    }
}

/// # Safety
///
/// `s` must be writable. `path` is a C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffered_file_serializer_init(
    s: *mut serializer,
    path: *const c_char,
    max_bufsize: usize,
    chunk_size: usize,
) -> bool {
    let Some(path) = path_from_c(path) else {
        return false;
    };
    let Some(output) = Output::create(&path, max_bufsize, chunk_size) else {
        return false;
    };
    // SAFETY: `s` is writable.
    unsafe {
        (*s).data = Box::into_raw(Box::new(output)).cast();
        (*s).read = None;
        (*s).write = Some(output_write);
        (*s).seek = Some(output_seek);
        (*s).get_pos = Some(output_pos);
    }
    true
}

/// # Safety
///
/// `s` must be writable. `path` is a C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffered_file_serializer_init_defaults(
    s: *mut serializer,
    path: *const c_char,
) -> bool {
    // SAFETY: `s` is writable. `path` is a C string or null.
    unsafe { buffered_file_serializer_init(s, path, 0, 0) }
}

/// # Safety
///
/// `s` came from a buffered init, or it is null, or its `data` pointer is null.
/// A null `s` returns. C would crash. The dangling `data` pointer is left in
/// place after a real free, matching C.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn buffered_file_serializer_free(s: *mut serializer) {
    // SAFETY: `s` is readable. A non-null `data` is an `Output` we allocated.
    unsafe {
        if s.is_null() || (*s).data.is_null() {
            return;
        }
        drop(Box::from_raw((*s).data.cast::<Output>()));
    }
}
