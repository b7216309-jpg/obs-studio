//! Differential fuzz target: the Rust obs-av1 shims against the C oracle
//! (libobs/obs-av1.c with its OBU bounds fix). Any mismatch panics.
//!
//! Run from `rust/obs-codec/fuzz`:
//! `soldr cargo +nightly fuzz run av1_differential`

#![no_main]

use core::{ptr, slice};

use libfuzzer_sys::fuzz_target;
use obs_c_oracle::av1 as c;
use obs_codec::ffi::av1 as rs;
use obs_util::ffi::darray::bfree;

/// Takes ownership of a `bmalloc`ed buffer: its bytes, then `bfree`.
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

fuzz_target!(|input: &[u8]| {
    // The first byte picks the metadata type; the rest is the bitstream.
    let (metadata_type, data) = match input.split_first() {
        Some((&t, rest)) => (t, rest),
        None => (0, input),
    };
    let p = data.as_ptr();
    let n = data.len();
    // SAFETY: every call reads only `data` and writes only locals; each
    // returned buffer is taken (and freed) once.
    unsafe {
        assert_eq!(
            rs::obs_av1_keyframe(p, n),
            c::oracle_obs_av1_keyframe(p, n)
        );

        let (mut rp, mut rpn, mut rh, mut rhn) = (ptr::null_mut(), 0, ptr::null_mut(), 0);
        let (mut op, mut opn, mut oh, mut ohn) = (ptr::null_mut(), 0, ptr::null_mut(), 0);
        rs::obs_extract_av1_headers(p, n, &mut rp, &mut rpn, &mut rh, &mut rhn);
        c::oracle_obs_extract_av1_headers(p, n, &mut op, &mut opn, &mut oh, &mut ohn);
        assert_eq!((rpn, rhn), (opn, ohn));
        assert_eq!(take(rp, rpn), take(op, opn));
        assert_eq!(take(rh, rhn), take(oh, ohn));

        let (mut r, mut rn, mut o, mut on) = (ptr::null_mut(), 0, ptr::null_mut(), 0);
        rs::metadata_obu(p, n, &mut r, &mut rn, metadata_type);
        c::oracle_metadata_obu(p, n, &mut o, &mut on, metadata_type);
        assert_eq!(rn, on);
        assert_eq!(take(r, rn), take(o, on));

        let (mut r, mut rn, mut o, mut on) = (ptr::null_mut(), 0, ptr::null_mut(), 0);
        rs::metadata_obu_itu_t35(p, n, &mut r, &mut rn);
        c::oracle_metadata_obu_itu_t35(p, n, &mut o, &mut on);
        assert_eq!(rn, on);
        assert_eq!(take(r, rn), take(o, on));
    }
});
