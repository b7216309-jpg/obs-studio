//! C ABI shims for the exported functions in `libobs/obs-av1.h`.
//!
//! The shims convert to the safe core in [`crate::av1`] and copy its results
//! into `bmalloc` memory, which the callers `bfree`.

use super::packet::{bmalloc_concat, bytes};
use crate::av1::{self, METADATA_TYPE_ITUT_T35};

/// Returns true if the first frame (header) OBU in the packet is a shown
/// key frame.
///
/// # Safety
///
/// A non-NULL `data` must be readable for `size` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obs_av1_keyframe(data: *const u8, size: usize) -> bool {
    // SAFETY: forwarded from the caller.
    av1::keyframe(unsafe { bytes(data, size) })
}

/// Copies the packet, and its sequence header and metadata OBUs, to new
/// `bmalloc` buffers (NULL with size 0 when empty).
///
/// # Safety
///
/// `packet` is readable for `size` bytes; every output pointer is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obs_extract_av1_headers(
    packet: *const u8,
    size: usize,
    new_packet_data: *mut *mut u8,
    new_packet_size: *mut usize,
    header_data: *mut *mut u8,
    header_size: *mut usize,
) {
    // SAFETY: the caller guarantees `size` readable bytes.
    let split = av1::extract_headers(unsafe { bytes(packet, size) });
    // SAFETY: the caller guarantees every output pointer is writable.
    unsafe {
        *new_packet_data = bmalloc_concat(&[&split.packet]);
        *new_packet_size = split.packet.len();
        *header_data = bmalloc_concat(&[&split.header]);
        *header_size = split.header.len();
    }
}

/// Builds a metadata OBU of `metadata_type` around `source_buffer`.
///
/// # Safety
///
/// `source_buffer` is readable for `source_bufsize` bytes; `out_buffer`
/// and `outbuf_size` are writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn metadata_obu(
    source_buffer: *const u8,
    source_bufsize: usize,
    out_buffer: *mut *mut u8,
    outbuf_size: *mut usize,
    metadata_type: u8,
) {
    // SAFETY: the caller guarantees `source_bufsize` readable bytes.
    let obu = av1::metadata_obu(
        unsafe { bytes(source_buffer, source_bufsize) },
        metadata_type,
    );
    // SAFETY: the caller guarantees both outputs are writable.
    unsafe {
        *outbuf_size = obu.len();
        *out_buffer = bmalloc_concat(&[&obu]);
    }
}

/// Builds an ITU-T T.35 metadata OBU around `itut_t35_buffer`.
///
/// # Safety
///
/// As [`metadata_obu`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn metadata_obu_itu_t35(
    itut_t35_buffer: *const u8,
    itut_bufsize: usize,
    out_buffer: *mut *mut u8,
    outbuf_size: *mut usize,
) {
    // SAFETY: forwarded from the caller.
    unsafe {
        metadata_obu(
            itut_t35_buffer,
            itut_bufsize,
            out_buffer,
            outbuf_size,
            METADATA_TYPE_ITUT_T35,
        );
    }
}
