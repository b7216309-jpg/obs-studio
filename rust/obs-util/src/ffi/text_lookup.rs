//! C ABI shim for `libobs/util/text-lookup.h`.
//!
//! `struct text_lookup` is opaque to C, so a `lookup_t *` is a boxed
//! [`TextLookup`]. The functions only convert pointers and delegate to
//! [`crate::text_lookup`].

use core::ffi::{CStr, c_char};
use core::ptr;

use crate::text_lookup::TextLookup;

/// The opaque `struct text_lookup`.
#[allow(non_camel_case_types)]
pub type lookup_t = TextLookup;

/// A C path as a `Path`; `None` for NULL or (on Windows) invalid UTF-8,
/// which `os_fopen` cannot open either.
///
/// # Safety
///
/// `path` must be NULL or a NUL-terminated string.
unsafe fn path<'a>(path: *const c_char) -> Option<&'a std::path::Path> {
    if path.is_null() {
        return None;
    }
    // SAFETY: per the contract.
    let bytes = unsafe { CStr::from_ptr(path) }.to_bytes();
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        Some(std::path::Path::new(std::ffi::OsStr::from_bytes(bytes)))
    }
    #[cfg(not(unix))]
    {
        std::str::from_utf8(bytes).ok().map(std::path::Path::new)
    }
}

/// Returns a new lookup holding the entries of `path`, or NULL if the file
/// cannot be read or is empty.
///
/// # Safety
///
/// `path` must be NULL or a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn text_lookup_create(path: *const c_char) -> *mut lookup_t {
    let mut lookup = Box::new(TextLookup::new());
    // SAFETY: per the contract.
    match unsafe { self::path(path) } {
        Some(p) if lookup.add_file(p) => Box::into_raw(lookup),
        _ => ptr::null_mut(),
    }
}

/// Adds the entries of `path`; later keys replace earlier ones.
///
/// Intentional difference: a NULL `lookup` returns `false` (C dereferences
/// it once the file is read).
///
/// # Safety
///
/// `lookup` must be NULL or from [`text_lookup_create`]; `path` must be
/// NULL or a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn text_lookup_add(lookup: *mut lookup_t, path: *const c_char) -> bool {
    // SAFETY: per the contract.
    unsafe {
        match (lookup.as_mut(), self::path(path)) {
            (Some(l), Some(p)) => l.add_file(p),
            _ => false,
        }
    }
}

/// # Safety
///
/// `lookup` must be NULL or from [`text_lookup_create`], not used after.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn text_lookup_destroy(lookup: *mut lookup_t) {
    if !lookup.is_null() {
        // SAFETY: the lookup came from Box::into_raw.
        drop(unsafe { Box::from_raw(lookup) });
    }
}

/// Sets `*out` to the value for `lookup_val` and returns `true`, or returns
/// `false` and leaves `*out` alone. The value lives until its key is
/// replaced or the lookup is destroyed.
///
/// Intentional difference: a NULL `lookup_val` or `out` returns `false`
/// (C dereferences them).
///
/// # Safety
///
/// `lookup` must be NULL or from [`text_lookup_create`]; `lookup_val` must
/// be NULL or a NUL-terminated string; `out` must be NULL or valid for
/// writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn text_lookup_getstr(
    lookup: *mut lookup_t,
    lookup_val: *const c_char,
    out: *mut *const c_char,
) -> bool {
    if lookup.is_null() || lookup_val.is_null() || out.is_null() {
        return false;
    }
    // SAFETY: per the contract.
    unsafe {
        let key = CStr::from_ptr(lookup_val).to_bytes();
        match (*lookup).get(key) {
            Some(v) => {
                *out = v.as_ptr();
                true
            }
            None => false,
        }
    }
}
