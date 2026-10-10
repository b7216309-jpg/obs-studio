//! Safe core of `libobs/obs-hevc.c`: H.265 (HEVC) Annex B helpers.
//!
//! The NAL walk, packet conversion, priority scan and header split are the
//! ones in [`crate::nal`], shared with the AVC port. HEVC only differs in
//! how it reads a NAL header: the unit type is bits 6..1 of the first of its
//! two header bytes.

use crate::nal::{self, Bucket, LengthPrefixed, SplitHeaders, UnitRating, nal_units};

pub const NAL_BLA_W_LP: u8 = 16;
pub const NAL_RSV_IRAP_VCL23: u8 = 23;
pub const NAL_VPS: u8 = 32;
pub const NAL_SPS: u8 = 33;
pub const NAL_PPS: u8 = 34;
pub const NAL_SEI_PREFIX: u8 = 39;
pub const NAL_SEI_SUFFIX: u8 = 40;

/// `OBS_NAL_PRIORITY_*` from `obs-nal.h`.
pub const PRIORITY_DISPOSABLE: i32 = 0;
pub const PRIORITY_HIGH: i32 = 2;
pub const PRIORITY_HIGHEST: i32 = 3;

/// `nal_unit_type`: `(header & 0x7F) >> 1`.
fn kind(header: u8) -> u8 {
    (header & 0x7f) >> 1
}

/// `obs_hevc_keyframe`: decided by the first unit whose type is 23 or
/// below, which is a keyframe for the IRAP types 16..=23. Units above 23
/// (parameter sets, SEI, but also the reserved VCL types 24..=31) are
/// skipped; no such unit is `false`.
#[must_use]
pub fn keyframe(data: &[u8]) -> bool {
    let _ = (kind(0), nal_units(data), Bucket::Header);
    let _ = nal::packet_priority(data, 0, rate);
    todo!()
}

/// `compute_hevc_keyframe_priority`: IRAP pictures (16..=23) are keyframes
/// with the highest priority; the sub-layer reference types (TRAIL_R,
/// TSA_R, STSA_R, RADL_R, RASL_R) are high; everything else is disposable.
fn rate(header: u8) -> UnitRating {
    match kind(header) {
        16..=23 => UnitRating {
            keyframe: true,
            priority: PRIORITY_HIGHEST,
        },
        1 | 3 | 5 | 7 | 9 => UnitRating {
            keyframe: false,
            priority: PRIORITY_HIGH,
        },
        _ => UnitRating {
            keyframe: false,
            priority: PRIORITY_DISPOSABLE,
        },
    }
}

/// `obs_parse_hevc_packet_priority`: `priority` raised to the highest unit
/// priority in `data`.
#[must_use]
pub fn packet_priority(data: &[u8], priority: i32) -> i32 {
    let _ = (data, priority);
    todo!()
}

/// `serialize_hevc_data`: converts Annex B `data` to length-prefixed units,
/// starting from the source packet's `keyframe` and `priority`.
#[must_use]
pub fn to_hvcc(data: &[u8], keyframe: bool, priority: i32) -> LengthPrefixed {
    let _ = (data, keyframe, priority);
    todo!()
}

/// `obs_extract_hevc_headers`: VPS, SPS and PPS units go to `header`,
/// prefix and suffix SEI to `sei`, everything else to `packet`, each with
/// its start code.
#[must_use]
pub fn extract_headers(data: &[u8]) -> SplitHeaders {
    let _ = data;
    todo!()
}
