//! C ABI shim for `os_get_path_extension` in `libobs/util/platform.h`.
//!
//! The function only converts to and from the safe core in
//! [`crate::path_extension`].

use core::ffi::{CStr, c_char};

use crate::path_extension::path_extension;

/// Returns a pointer to the extension (including the `.`) inside `path`, or
/// null if there is none.
///
/// Intentional difference from C: a null `path` returns null; the C original
/// calls `strlen(NULL)`, which is undefined behavior.
///
/// # Safety
///
/// `path` must be null or point to a valid NUL-terminated string that stays
/// valid for as long as the returned pointer is used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn os_get_path_extension(path: *const c_char) -> *const c_char {
    if path.is_null() {
        return core::ptr::null();
    }
    // SAFETY: `path` is non-null and the caller guarantees it is a valid
    // NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(path) }.to_bytes();
    match path_extension(bytes) {
        Some(ext) => {
            let offset = bytes.len() - ext.len();
            // SAFETY: `offset` is less than the string length, so the result
            // stays inside the same allocation.
            unsafe { path.add(offset) }
        }
        None => core::ptr::null(),
    }
}
