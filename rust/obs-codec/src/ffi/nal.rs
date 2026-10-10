//! C ABI shim for the exported function in `libobs/obs-nal.h`.

use core::slice;

use crate::nal::find_startcode;

/// Returns a pointer to the first start code in `[p, end)`, or `end` if
/// there is none.
///
/// When `end` is not after `p` (an empty or reversed range) this returns
/// `end`, as the C code does.
///
/// # Safety
///
/// When `p < end`, `[p, end)` must be one readable allocation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn obs_nal_find_startcode(p: *const u8, end: *const u8) -> *const u8 {
    if end <= p {
        return end;
    }
    // SAFETY: `p < end` and the caller guarantees `[p, end)` is readable.
    let len = unsafe { end.offset_from(p) } as usize;
    // SAFETY: as above.
    let data = unsafe { slice::from_raw_parts(p, len) };
    // SAFETY: the offset is at most `len`, so the result is within
    // `[p, end]`.
    unsafe { p.add(find_startcode(data)) }
}
