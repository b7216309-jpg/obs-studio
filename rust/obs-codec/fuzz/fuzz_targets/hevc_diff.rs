//! cargo-fuzz differential target: the Rust `obs-hevc.c` port against the
//! original C compiled as an oracle (`obs-c-oracle`). Any difference in a
//! return value, an output field or an output buffer aborts.
//!
//! Input: byte 0 picks the source packet's `keyframe` (bit 0) and
//! `priority` (bits 1..3, minus 2); the rest is the Annex B bitstream.
//!
//! Excluded inputs (C UB or crash, documented in rust/README.md):
//! - NULL data / a range within 3 bytes of address 0: the C start-code
//!   search computes `end - 3` and reads address 0. The bitstream is always
//!   copied into a real heap allocation, so this cannot occur here.
//!
//! Run: `soldr cargo +nightly fuzz run hevc_diff` from `rust/obs-codec`.
#![no_main]

use core::ffi::c_long;
use core::mem::size_of;
use core::{ptr, slice};

use libfuzzer_sys::fuzz_target;
use obs_c_oracle::hevc::{self as c, OracleEncoderPacket};
use obs_codec::ffi::hevc as rs;
use obs_codec::ffi::packet::encoder_packet;
use obs_util::ffi::darray::bfree;

fn take(p: *mut u8, len: usize) -> Option<Vec<u8>> {
    if p.is_null() {
        return None;
    }
    // SAFETY: `p` holds `len` bytes from bmalloc and is freed once.
    unsafe {
        let v = slice::from_raw_parts(p, len).to_vec();
        bfree(p.cast());
        Some(v)
    }
}

fn take_packet(data: *mut u8, size: usize) -> Vec<u8> {
    // SAFETY: `data` points `sizeof(long)` bytes into a bmalloc block.
    let block = unsafe { data.sub(size_of::<c_long>()) };
    take(block, size_of::<c_long>() + size).expect("parsed packet has data")
}

fn packet(data: &[u8], keyframe: bool, priority: i32) -> encoder_packet {
    encoder_packet {
        data: data.as_ptr().cast_mut(),
        size: data.len(),
        pts: 11,
        dts: 7,
        timebase_num: 1,
        timebase_den: 60,
        type_: 1,
        keyframe,
        dts_usec: 1234,
        sys_dts_usec: 5678,
        priority,
        drop_priority: -7,
        track_idx: 2,
        encoder: ptr::dangling_mut(),
    }
}

fn to_oracle(p: &encoder_packet) -> OracleEncoderPacket {
    OracleEncoderPacket {
        data: p.data,
        size: p.size,
        pts: p.pts,
        dts: p.dts,
        timebase_num: p.timebase_num,
        timebase_den: p.timebase_den,
        type_: p.type_,
        keyframe: p.keyframe,
        dts_usec: p.dts_usec,
        sys_dts_usec: p.sys_dts_usec,
        priority: p.priority,
        drop_priority: p.drop_priority,
        track_idx: p.track_idx,
        encoder: p.encoder,
    }
}

fuzz_target!(|input: &[u8]| {
    let Some((&flags, body)) = input.split_first() else {
        return;
    };
    let keyframe = flags & 1 != 0;
    let priority = i32::from((flags >> 1) & 7) - 2;

    // Keep even an empty body inside a real allocation (see the exclusion).
    let mut backing = body.to_vec();
    backing.push(0xee);
    let data = &backing[..body.len()];
    let p = data.as_ptr();

    // SAFETY: every call reads only `data` and writes only locals; each
    // returned buffer is taken (and freed) once.
    unsafe {
        assert_eq!(
            rs::obs_hevc_keyframe(p, data.len()),
            c::oracle_obs_hevc_keyframe(p, data.len())
        );

        let src = packet(data, keyframe, priority);
        let osrc = to_oracle(&src);
        assert_eq!(
            rs::obs_parse_hevc_packet_priority(&src),
            c::oracle_obs_parse_hevc_packet_priority(&osrc)
        );

        let mut rd = packet(&[], false, 0);
        let mut cd = to_oracle(&rd);
        rs::obs_parse_hevc_packet(&mut rd, &src);
        c::oracle_obs_parse_hevc_packet(&mut cd, &osrc);
        assert_eq!(
            (rd.size, rd.keyframe, rd.priority, rd.drop_priority),
            (cd.size, cd.keyframe, cd.priority, cd.drop_priority)
        );
        assert_eq!(
            (rd.pts, rd.dts, rd.dts_usec, rd.sys_dts_usec, rd.track_idx),
            (cd.pts, cd.dts, cd.dts_usec, cd.sys_dts_usec, cd.track_idx)
        );
        assert_eq!(take_packet(rd.data, rd.size), take_packet(cd.data, cd.size));

        let mut r = [ptr::null_mut::<u8>(); 3];
        let mut rn = [0usize; 3];
        let mut o = [ptr::null_mut::<u8>(); 3];
        let mut on = [0usize; 3];
        rs::obs_extract_hevc_headers(
            p,
            data.len(),
            &mut r[0],
            &mut rn[0],
            &mut r[1],
            &mut rn[1],
            &mut r[2],
            &mut rn[2],
        );
        c::oracle_obs_extract_hevc_headers(
            p,
            data.len(),
            &mut o[0],
            &mut on[0],
            &mut o[1],
            &mut on[1],
            &mut o[2],
            &mut on[2],
        );
        assert_eq!(rn, on);
        for i in 0..3 {
            assert_eq!(take(r[i], rn[i]), take(o[i], on[i]));
        }
    }
});
