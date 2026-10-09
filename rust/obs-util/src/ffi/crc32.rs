//! C ABI shim for `calc_crc32` in `libobs/util/crc32.h`.
//!
//! The function only converts to the safe core in [`crate::crc32`].

use core::ffi::c_void;

use crate::crc32;

/// Updates the CRC-32 `crc` with `size` bytes at `buf`.
///
/// # Safety
///
/// `buf` must point to `size` readable bytes. It may be null or dangling
/// when `size` is 0.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn calc_crc32(crc: u32, buf: *const c_void, size: usize) -> u32 {
    let bytes: &[u8] = if size == 0 {
        &[]
    } else {
        // SAFETY: `size` is non-zero, so the caller guarantees `buf` points
        // to `size` readable bytes.
        unsafe { core::slice::from_raw_parts(buf.cast::<u8>(), size) }
    };
    crc32::calc_crc32(crc, bytes)
}
