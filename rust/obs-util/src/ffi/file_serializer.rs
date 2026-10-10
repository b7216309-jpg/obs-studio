//! C ABI shim for `libobs/util/file-serializer.c`.

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use super::array_serializer::serializer;
use crate::file_serializer::{Input, Output, seek_type_from_c};

unsafe extern "C" fn input_read(param: *mut c_void, data: *mut c_void, size: usize) -> usize {
    if data.is_null() || size == 0 {
        return 0;
    }
    // SAFETY: `param` is the `Input` installed by init. `data` covers `size`.
    let input = unsafe { &mut *(param as *mut Input) };
    let buf = unsafe { std::slice::from_raw_parts_mut(data.cast::<u8>(), size) };
    input.read(buf)
}

unsafe extern "C" fn input_seek(param: *mut c_void, offset: i64, seek_type: c_int) -> i64 {
    // SAFETY: `param` is the `Input` installed by init.
    let input = unsafe { &mut *(param as *mut Input) };
    input.seek(offset, seek_type_from_c(seek_type))
}

unsafe extern "C" fn input_pos(param: *mut c_void) -> i64 {
    // SAFETY: `param` is the `Input` installed by init.
    let input = unsafe { &mut *(param as *mut Input) };
    input.pos()
}

unsafe extern "C" fn output_write(param: *mut c_void, data: *const c_void, size: usize) -> usize {
    if data.is_null() || size == 0 {
        return 0;
    }
    // SAFETY: `param` is the `Output` installed by init. `data` covers `size`.
    let output = unsafe { &mut *(param as *mut Output) };
    let buf = unsafe { std::slice::from_raw_parts(data.cast::<u8>(), size) };
    output.write(buf)
}

unsafe extern "C" fn output_seek(param: *mut c_void, offset: i64, seek_type: c_int) -> i64 {
    // SAFETY: `param` is the `Output` installed by init.
    let output = unsafe { &mut *(param as *mut Output) };
    output.seek(offset, seek_type_from_c(seek_type))
}

unsafe extern "C" fn output_pos(param: *mut c_void) -> i64 {
    // SAFETY: `param` is the `Output` installed by init.
    let output = unsafe { &mut *(param as *mut Output) };
    output.pos()
}

fn bytes<'a>(ptr: *const c_char) -> Option<&'a [u8]> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: `ptr` is a C string when non-null.
    Some(unsafe { CStr::from_ptr(ptr) }.to_bytes())
}

/// # Safety
///
/// `s` must be writable. `path` is a C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_input_serializer_init(
    s: *mut serializer,
    path: *const c_char,
) -> bool {
    let Some(path) = bytes(path) else {
        // SAFETY: `s` is writable. C stores the null `FILE *` and returns false.
        unsafe { (*s).data = ptr::null_mut() };
        return false;
    };
    let Some(input) = Input::open(path) else {
        // SAFETY: `s` is writable. C stores the null `FILE *` and returns false.
        unsafe { (*s).data = ptr::null_mut() };
        return false;
    };
    // SAFETY: `s` is writable.
    unsafe {
        (*s).data = Box::into_raw(Box::new(input)).cast();
        (*s).read = Some(input_read);
        (*s).write = None;
        (*s).seek = Some(input_seek);
        (*s).get_pos = Some(input_pos);
    }
    true
}

/// # Safety
///
/// `s` came from [`file_input_serializer_init`] or has a null `data` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_input_serializer_free(s: *mut serializer) {
    // SAFETY: `s` is readable. A non-null `data` is an `Input` we allocated.
    unsafe {
        if s.is_null() || (*s).data.is_null() {
            return;
        }
        drop(Box::from_raw((*s).data.cast::<Input>()));
    }
}

/// # Safety
///
/// `s` must be writable. `path` is a C string or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_output_serializer_init(
    s: *mut serializer,
    path: *const c_char,
) -> bool {
    let Some(path) = bytes(path) else {
        return false;
    };
    let Some(output) = Output::create(path) else {
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
/// `s` must be writable. `path` and `temp_ext` are C strings or null.
///
/// A null `path` returns false. C would build a temp name from the extension
/// alone and then call `os_unlink(NULL)` on free.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_output_serializer_init_safe(
    s: *mut serializer,
    path: *const c_char,
    temp_ext: *const c_char,
) -> bool {
    let (Some(path), Some(ext)) = (bytes(path), bytes(temp_ext)) else {
        return false;
    };
    let Some(output) = Output::create_safe(path, ext) else {
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
/// `s` came from an output init or has a null `data` pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn file_output_serializer_free(s: *mut serializer) {
    // SAFETY: `s` is readable. A non-null `data` is an `Output` we allocated.
    // Drop closes the file and, for a safe init, renames the temp into place.
    unsafe {
        if s.is_null() || (*s).data.is_null() {
            return;
        }
        drop(Box::from_raw((*s).data.cast::<Output>()));
    }
}
