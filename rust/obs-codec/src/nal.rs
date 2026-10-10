//! Safe core of `libobs/obs-nal.c`: the Annex B start-code search shared by
//! the AVC and HEVC helpers.
//!
//! The C code is FFmpeg's `ff_avc_find_startcode_internal`, which scans a
//! word at a time after an alignment prologue. The word scan reports the
//! earliest `00 00 01` it can see, in byte order, so its result is the same
//! as the plain byte scan here, whatever the buffer's alignment. What the C
//! code does decide is which positions are searched: a start code is only
//! found at `i` when `i + 3 < len`, so one in the last three bytes, with
//! nothing after it, is not reported.

/// `ff_avc_find_startcode_internal`: the first `i` with
/// `data[i..i + 3] == [0, 0, 1]` and `i + 3 < data.len()`, else
/// `data.len()`.
fn find_startcode_internal(data: &[u8]) -> usize {
    let searchable = data.len().saturating_sub(3);
    (0..searchable)
        .find(|&i| data[i] == 0 && data[i + 1] == 0 && data[i + 2] == 1)
        .unwrap_or(data.len())
}

/// `obs_nal_find_startcode`: the offset of the first start code in `data`,
/// or `data.len()` if there is none.
///
/// A 3-byte `00 00 01` preceded by a zero is reported at that zero, so a
/// 4-byte `00 00 00 01` code is found at its first byte. Only one zero is
/// folded in, and never one before the start of `data`.
#[must_use]
pub fn find_startcode(data: &[u8]) -> usize {
    let _ = (data, find_startcode_internal);
    todo!()
}
