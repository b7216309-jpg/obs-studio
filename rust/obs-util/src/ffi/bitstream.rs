//! C ABI shim for `libobs/util/bitstream.h`.
//!
//! The header is the source of truth: `bitstream_reader` keeps its exact
//! layout, including `uint8_t pos` (callers stack-allocate the struct and
//! read `pos` directly). The functions only convert to and from the safe
//! core in [`crate::bitstream`].

use core::ffi::c_int;
use core::slice;

use crate::bitstream::Reader;

/// Mirrors `struct bitstream_reader` in `libobs/util/bitstream.h`.
#[repr(C)]
#[allow(non_camel_case_types, non_snake_case)]
pub struct bitstream_reader {
    pub pos: u8,
    pub subPos: u8,
    pub buf: *mut u8,
    pub len: usize,
}

/// Runs `f` on a core reader built from `r`, then writes the position back.
///
/// # Safety
///
/// `r` must point to a valid `bitstream_reader` whose `buf` is valid for
/// reads of `len` bytes (or null).
unsafe fn with_reader<T>(r: *mut bitstream_reader, f: impl FnOnce(&mut Reader<'_, u8>) -> T) -> T {
    // SAFETY: the caller guarantees `r` is valid and not aliased for the call.
    let r = unsafe { &mut *r };
    let buf = if r.buf.is_null() {
        &[][..]
    } else {
        // SAFETY: the caller guarantees `buf` is valid for `len` bytes, the
        // same contract the C implementation relies on.
        unsafe { slice::from_raw_parts(r.buf, r.len) }
    };
    let mut reader = Reader {
        buf,
        pos: r.pos,
        sub_pos: r.subPos,
    };
    let out = f(&mut reader);
    r.pos = reader.pos;
    r.subPos = reader.sub_pos;
    out
}

/// # Safety
///
/// `r` must point to writable memory for a `bitstream_reader`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitstream_reader_init(
    r: *mut bitstream_reader,
    data: *mut u8,
    len: usize,
) {
    // SAFETY: the caller guarantees `r` is writable.
    unsafe {
        r.write(bitstream_reader {
            pos: 0,
            subPos: 0x80,
            buf: data,
            len,
        });
    }
}

/// # Safety
///
/// See [`with_reader`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitstream_reader_read_bits(r: *mut bitstream_reader, bits: c_int) -> u8 {
    // A negative count reads nothing, as in the C loop.
    let bits = u32::try_from(bits).unwrap_or(0);
    // SAFETY: forwarded from the caller.
    unsafe { with_reader(r, |reader| reader.read_bits(bits)) }
}

/// # Safety
///
/// See [`with_reader`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitstream_reader_r8(r: *mut bitstream_reader) -> u8 {
    // SAFETY: forwarded from the caller.
    unsafe { with_reader(r, |reader| reader.r8()) }
}

/// # Safety
///
/// See [`with_reader`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn bitstream_reader_r16(r: *mut bitstream_reader) -> u16 {
    // SAFETY: forwarded from the caller.
    unsafe { with_reader(r, |reader| reader.r16()) }
}
