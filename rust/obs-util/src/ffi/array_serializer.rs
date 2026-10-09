//! C ABI shim for `libobs/util/array-serializer.c`.

use core::ffi::{c_int, c_void};
use core::ptr;

use super::darray::{
    darray, darray_clear, darray_ensure_capacity, darray_free, darray_push_back_array,
};
use crate::array_serializer::{WritePlan, plan_write, seek_target, seek_type_from_c};

/// Mirrors `struct serializer` in `libobs/util/serializer.h`.
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct serializer {
    pub data: *mut c_void,
    pub read: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> usize>,
    pub write: Option<unsafe extern "C" fn(*mut c_void, *const c_void, usize) -> usize>,
    pub seek: Option<unsafe extern "C" fn(*mut c_void, i64, c_int) -> i64>,
    pub get_pos: Option<unsafe extern "C" fn(*mut c_void) -> i64>,
}

/// Mirrors `struct array_output_data` (`DARRAY(uint8_t)` has the layout of
/// `struct darray`).
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct array_output_data {
    pub bytes: darray,
    pub cur_pos: usize,
}

// Static in C: deliberately not exported.
unsafe extern "C" fn array_output_write(
    param: *mut c_void,
    data: *const c_void,
    size: usize,
) -> usize {
    let output = param as *mut array_output_data;
    // SAFETY: `param` is the `array_output_data` installed by init.
    let (cur_pos, num) = unsafe { ((*output).cur_pos, (*output).bytes.num) };
    match plan_write(cur_pos, num, size) {
        WritePlan::Overwrite { offset, new_num } => {
            if new_num > num {
                // SAFETY: `output` is valid; the darray holds 1-byte items.
                unsafe {
                    darray_ensure_capacity(1, &raw mut (*output).bytes, new_num);
                    (*output).bytes.num = new_num;
                }
            }
            if size > 0 {
                // SAFETY: capacity covers `new_num >= offset + size` bytes and
                // the caller guarantees `data` is valid for `size` bytes.
                unsafe {
                    ptr::copy_nonoverlapping(
                        data as *const u8,
                        ((*output).bytes.array as *mut u8).add(offset),
                        size,
                    );
                }
            }
        }
        WritePlan::Append => {
            // SAFETY: `output` is valid; `data` is valid for `size` bytes.
            unsafe {
                darray_push_back_array(1, &raw mut (*output).bytes, data, size);
            }
        }
    }
    // SAFETY: `output` is valid.
    unsafe { (*output).cur_pos = cur_pos.wrapping_add(size) };
    size
}

// Static in C: deliberately not exported.
unsafe extern "C" fn array_output_get_pos(param: *mut c_void) -> i64 {
    let data = param as *mut array_output_data;
    // SAFETY: `param` is the `array_output_data` installed by init.
    unsafe { (*data).bytes.num as i64 }
}

// Static in C: deliberately not exported.
unsafe extern "C" fn array_output_seek(param: *mut c_void, offset: i64, seek_type: c_int) -> i64 {
    let output = param as *mut array_output_data;
    // SAFETY: `param` is the `array_output_data` installed by init.
    let (cur_pos, num) = unsafe { ((*output).cur_pos, (*output).bytes.num) };
    match seek_target(cur_pos, num, offset, seek_type_from_c(seek_type)) {
        Some(target) => {
            // SAFETY: `output` is valid.
            unsafe { (*output).cur_pos = target };
            target as i64
        }
        None => -1,
    }
}

/// # Safety
///
/// `s` and `data` must be valid for writes and must not overlap.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn array_output_serializer_init(
    s: *mut serializer,
    data: *mut array_output_data,
) {
    // SAFETY: the caller guarantees both pointers are valid for writes; an
    // all-zero bit pattern is valid for both structs.
    unsafe {
        ptr::write_bytes(s, 0, 1);
        ptr::write_bytes(data, 0, 1);
        (*s).data = data as *mut c_void;
        (*s).write = Some(array_output_write);
        (*s).get_pos = Some(array_output_get_pos);
        (*s).seek = Some(array_output_seek);
    }
}

/// # Safety
///
/// `data` must point to an initialized `array_output_data`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn array_output_serializer_free(data: *mut array_output_data) {
    // SAFETY: the caller guarantees `data` is valid.
    unsafe { darray_free(&raw mut (*data).bytes) };
}

/// # Safety
///
/// `data` must point to an initialized `array_output_data`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn array_output_serializer_reset(data: *mut array_output_data) {
    // SAFETY: the caller guarantees `data` is valid.
    unsafe {
        darray_clear(&raw mut (*data).bytes);
        (*data).cur_pos = 0;
    }
}
