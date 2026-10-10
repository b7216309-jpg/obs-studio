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
    let out = find_startcode_internal(data);
    if 0 < out && out < data.len() && data[out - 1] == 0 {
        out - 1
    } else {
        out
    }
}

/// One NAL unit of an Annex B buffer, as offsets into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nal {
    /// Where the search for this unit started: its start code, or for the
    /// first unit, the first start code in the buffer.
    pub code_start: usize,
    /// The NAL header, just after the start code.
    pub start: usize,
    /// The next start code, or the end of the buffer.
    pub end: usize,
}

impl Nal {
    /// The first header byte.
    #[must_use]
    pub fn header(&self, data: &[u8]) -> u8 {
        data[self.start]
    }
}

/// The NAL units of `data`, in order.
///
/// This is the loop every function in `obs-avc.c` and `obs-hevc.c`
/// repeats:
///
/// ```c
/// nal_start = obs_nal_find_startcode(data, end);
/// while (true) {
///     while (nal_start < end && !*(nal_start++))
///         ;
///     if (nal_start == end)
///         break;
///     nal_end = obs_nal_find_startcode(nal_start, end);
///     ...
///     nal_start = nal_end;
/// }
/// ```
///
/// The inner loop skips zeros and then one more byte (the `01` of the start
/// code), so a unit starts after the first nonzero byte, and the walk ends
/// when that lands on the end of the buffer.
pub fn nal_units(data: &[u8]) -> impl Iterator<Item = Nal> + '_ {
    let mut pos = find_startcode(data);
    core::iter::from_fn(move || {
        let code_start = pos;
        while pos < data.len() {
            let byte = data[pos];
            pos += 1;
            if byte != 0 {
                break;
            }
        }
        if pos == data.len() {
            return None;
        }
        let start = pos;
        let end = start + find_startcode(&data[start..]);
        pos = end;
        Some(Nal {
            code_start,
            start,
            end,
        })
    })
}

/// How a codec rates one NAL unit, from its first header byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnitRating {
    /// The unit makes the packet a keyframe.
    pub keyframe: bool,
    /// The unit's priority; a packet takes the highest.
    pub priority: i32,
}

/// What `obs_parse_avc_packet` / `obs_parse_hevc_packet` derive from an
/// Annex B packet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LengthPrefixed {
    /// The units, each as a big-endian `u32` size and its bytes (header
    /// included, start code dropped).
    pub payload: Vec<u8>,
    /// `keyframe` after the walk: set by any keyframe unit, else unchanged.
    pub keyframe: bool,
    /// `priority` after the walk: raised to the highest unit priority.
    pub priority: i32,
}

/// `serialize_avc_data` / `serialize_hevc_data`: converts Annex B `data` to
/// length-prefixed units, starting from the source packet's `keyframe` and
/// `priority`.
#[must_use]
pub fn to_length_prefixed(
    data: &[u8],
    keyframe: bool,
    priority: i32,
    rate: impl Fn(u8) -> UnitRating,
) -> LengthPrefixed {
    let mut out = LengthPrefixed {
        payload: Vec::with_capacity(data.len() + 16),
        keyframe,
        priority,
    };
    for nal in nal_units(data) {
        let r = rate(nal.header(data));
        if r.keyframe {
            out.keyframe = true;
        }
        // C: return priority > new_priority ? priority : new_priority;
        out.priority = out.priority.max(r.priority);
        let unit = &data[nal.start..nal.end];
        // C: s_wb32(s, (uint32_t)nal_size)
        out.payload
            .extend_from_slice(&(unit.len() as u32).to_be_bytes());
        out.payload.extend_from_slice(unit);
    }
    out
}

/// `obs_parse_*_packet_priority`: `priority` raised to the highest unit
/// priority in `data`.
#[must_use]
pub fn packet_priority(data: &[u8], priority: i32, rate: impl Fn(u8) -> UnitRating) -> i32 {
    nal_units(data).fold(priority, |p, nal| p.max(rate(nal.header(data)).priority))
}

/// Where `obs_extract_*_headers` puts a unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bucket {
    Header,
    Sei,
    Packet,
}

/// `obs_extract_*_headers`: `data` split by unit, each unit with its start
/// code.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SplitHeaders {
    pub packet: Vec<u8>,
    pub header: Vec<u8>,
    pub sei: Vec<u8>,
}

/// `obs_extract_*_headers`, with `bucket` choosing from the first header
/// byte.
#[must_use]
pub fn split_headers(data: &[u8], bucket: impl Fn(u8) -> Bucket) -> SplitHeaders {
    let mut out = SplitHeaders::default();
    for nal in nal_units(data) {
        let unit = &data[nal.code_start..nal.end];
        match bucket(nal.header(data)) {
            Bucket::Header => out.header.extend_from_slice(unit),
            Bucket::Sei => out.sei.extend_from_slice(unit),
            Bucket::Packet => out.packet.extend_from_slice(unit),
        }
    }
    out
}
