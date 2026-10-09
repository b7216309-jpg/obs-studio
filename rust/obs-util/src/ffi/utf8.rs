//! C ABI shim for `utf8_to_wchar` and `wchar_to_utf8` in
//! `libobs/util/utf8.h`.
//!
//! Non-Windows only: on Windows `utf8.c` wraps `MultiByteToWideChar` and
//! `WideCharToMultiByte` on 16-bit `wchar_t` and stays in C. libobs does not
//! export these symbols, so they are listed as local in
//! `libobs/cmake/rust-exports.map` and `rust-unexports-macos.txt` instead of
//! `rust-exports.txt`.
//!
//! The functions only convert pointers to slices and delegate to
//! [`crate::utf8`].

use core::ffi::{CStr, c_char, c_int};

use libc::wchar_t;

use crate::utf8;

/// Decodes UTF-8 into `wchar_t`. See `utf8_to_wchar` in `util/utf8.c`.
///
/// # Safety
///
/// `input` must be null, point to `insize` readable bytes, or, when `insize`
/// is 0, point to a NUL-terminated string. `out` must be null or point to
/// `outsize` writable `wchar_t`s.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn utf8_to_wchar(
    input: *const c_char,
    insize: usize,
    out: *mut wchar_t,
    outsize: usize,
    flags: c_int,
) -> usize {
    if input.is_null() {
        return 0;
    }
    let input: &[u8] = if insize == 0 {
        // SAFETY: with `insize` 0 the caller passes a NUL-terminated string.
        unsafe { CStr::from_ptr(input) }.to_bytes()
    } else {
        // SAFETY: the caller guarantees `insize` readable bytes at `input`.
        unsafe { core::slice::from_raw_parts(input.cast::<u8>(), insize) }
    };
    let out = if out.is_null() {
        None
    } else {
        // SAFETY: the caller guarantees `outsize` writable `wchar_t`s at `out`.
        Some(unsafe { core::slice::from_raw_parts_mut(out, outsize) })
    };
    utf8::utf8_to_wchar(input, out, flags)
}

/// Encodes `wchar_t` as UTF-8. See `wchar_to_utf8` in `util/utf8.c`.
///
/// # Safety
///
/// `input` must be null, point to `insize` readable `wchar_t`s, or, when
/// `insize` is 0, point to a NUL-terminated wide string. `out` must be null
/// or point to `outsize` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wchar_to_utf8(
    input: *const wchar_t,
    insize: usize,
    out: *mut c_char,
    outsize: usize,
    flags: c_int,
) -> usize {
    if input.is_null() {
        return 0;
    }
    let len = if insize == 0 {
        let mut len = 0;
        // SAFETY: with `insize` 0 the caller passes a NUL-terminated wide
        // string, so every element up to the terminator is readable.
        while unsafe { *input.add(len) } != 0 {
            len += 1;
        }
        len
    } else {
        insize
    };
    // SAFETY: `len` elements at `input` are readable (see above).
    let input = unsafe { core::slice::from_raw_parts(input, len) };
    let out = if out.is_null() {
        None
    } else {
        // SAFETY: the caller guarantees `outsize` writable bytes at `out`.
        Some(unsafe { core::slice::from_raw_parts_mut(out.cast::<u8>(), outsize) })
    };
    utf8::wchar_to_utf8(input, out, flags)
}
