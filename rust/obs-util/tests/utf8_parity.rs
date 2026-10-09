//! Tier 3: the safe core, and on non-Windows the C ABI shim, behave exactly
//! like the original non-Windows `util/utf8.c`, compiled as an oracle with
//! `wchar_t` as `int32_t` and as `uint32_t` (`oracle/utf8_*.c`).
//!
//! Inputs are arbitrary byte strings, including invalid UTF-8, and arbitrary
//! 32-bit values. Every call compares the return value and the whole output
//! buffer, since on error C leaves partial output behind.
//!
//! No intentional differences from C.
//!
//! Reminder: the mutation check (break the core, see this file fail) is done
//! once per port, not on every change.

use core::ffi::c_char;

use obs_c_oracle::utf8 as c;
use obs_util::utf8::{WChar, utf8_to_wchar, wchar_to_utf8};
use proptest::prelude::*;

/// Fills output buffers so untouched elements are visible.
const SENTINEL_W: u32 = 0xA5A5_A5A5;
const SENTINEL_B: u8 = 0xA5;

/// Bytes that exercise every branch of the decoder.
const EDGE_BYTES: &[u8] = &[
    0x00, 0x41, 0x7f, 0x80, 0x8f, 0x90, 0xa0, 0xbb, 0xbf, 0xc0, 0xc1, 0xc2, 0xc3, 0xdf, 0xe0, 0xe2,
    0xed, 0xef, 0xf0, 0xf4, 0xf5, 0xf7, 0xf8, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

/// Code points around every encoding-length and validity boundary.
const EDGE_CODES: &[u32] = &[
    0,
    0x61,
    0x7f,
    0x80,
    0x7ff,
    0x800,
    0xd7ff,
    0xd800,
    0xdbff,
    0xdc00,
    0xdfff,
    0xe000,
    0xfeff,
    0xffff,
    0x1_0000,
    0x10_ffff,
    0x11_0000,
    0x1f_ffff,
    0x20_0000,
    0x3ff_ffff,
    0x400_0000,
    0x7fff_ffff,
    0x8000_0000,
    0xc000_0000,
    0xffff_ffff,
];

/// Any lead byte followed by 0 to 6 continuation bytes: well-formed,
/// truncated, overlong, out-of-range and forbidden sequences alike.
fn sequence() -> impl Strategy<Value = Vec<u8>> {
    (any::<u8>(), prop::collection::vec(0x80u8..=0xbf, 0..=6)).prop_map(|(lead, mut rest)| {
        rest.insert(0, lead);
        rest
    })
}

/// Mostly well-formed text with some invalid sequences mixed in, so errors
/// do not always end the comparison at the first byte.
fn mixed_text() -> impl Strategy<Value = Vec<u8>> {
    let token = prop_oneof![
        4 => any::<char>().prop_map(|c| c.to_string().into_bytes()),
        2 => sequence(),
        1 => prop::sample::select(EDGE_BYTES).prop_map(|b| vec![b]),
    ];
    prop::collection::vec(token, 0..24).prop_map(|t| t.concat())
}

fn utf8_input() -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        1 => prop::collection::vec(any::<u8>(), 0..64),
        1 => prop::collection::vec(prop::sample::select(EDGE_BYTES), 0..64),
        1 => any::<String>().prop_map(String::into_bytes),
        3 => sequence(),
        3 => mixed_text(),
    ]
}

fn wide_input() -> impl Strategy<Value = Vec<u32>> {
    prop_oneof![
        prop::collection::vec(any::<u32>(), 0..32),
        prop::collection::vec(prop::sample::select(EDGE_CODES), 0..32),
        any::<String>().prop_map(|s| s.chars().map(u32::from).collect()),
    ]
}

/// `None`: count only (C `out` NULL). `Some(n)`: an `n`-element buffer.
fn out_len() -> impl Strategy<Value = Option<usize>> {
    prop_oneof![Just(None), (0usize..80).prop_map(Some)]
}

/// Calls `f(ptr, insize)` with the C equivalent of the slice `v`. An empty
/// slice has no valid pointer of its own, so it becomes an empty
/// NUL-terminated string with `insize` 0, which C treats the same.
fn with_c_input<T: Copy + Default, R>(v: &[T], f: impl FnOnce(*const T, usize) -> R) -> R {
    if v.is_empty() {
        let nul = [T::default()];
        f(nul.as_ptr(), 0)
    } else {
        f(v.as_ptr(), v.len())
    }
}

/// Runs the core and an oracle on the same input; returns both results and
/// output buffers.
fn decode_both<W: WChar + PartialEq + core::fmt::Debug>(
    input: &[u8],
    out: Option<usize>,
    flags: i32,
    sentinel: W,
    oracle: unsafe extern "C" fn(*const c_char, usize, *mut W, usize, i32) -> usize,
) -> ((usize, Vec<W>), (usize, Vec<W>)) {
    let n = out.unwrap_or(0);
    let mut ours = vec![sentinel; n];
    let mut theirs = vec![sentinel; n];
    let r_ours = utf8_to_wchar(input, out.map(|_| &mut ours[..]), flags);
    let r_theirs = with_c_input(input, |p, len| {
        let dst = if out.is_some() {
            theirs.as_mut_ptr()
        } else {
            core::ptr::null_mut()
        };
        // SAFETY: `p` holds `len` bytes (or is NUL-terminated when `len` is
        // 0) and `dst` is null or holds `n` elements; both outlive the call.
        unsafe { oracle(p.cast(), len, dst, n, flags) }
    });
    ((r_ours, ours), (r_theirs, theirs))
}

fn encode_both<W: WChar + Default>(
    input: &[W],
    out: Option<usize>,
    flags: i32,
    oracle: unsafe extern "C" fn(*const W, usize, *mut c_char, usize, i32) -> usize,
) -> ((usize, Vec<u8>), (usize, Vec<u8>)) {
    let n = out.unwrap_or(0);
    let mut ours = vec![SENTINEL_B; n];
    let mut theirs = vec![SENTINEL_B; n];
    let r_ours = wchar_to_utf8(input, out.map(|_| &mut ours[..]), flags);
    let r_theirs = with_c_input(input, |p, len| {
        let dst = if out.is_some() {
            theirs.as_mut_ptr()
        } else {
            core::ptr::null_mut()
        };
        // SAFETY: as in `decode_both`.
        unsafe { oracle(p, len, dst.cast(), n, flags) }
    });
    ((r_ours, ours), (r_theirs, theirs))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn utf8_to_wchar_matches_c_s32(input in utf8_input(), out in out_len(), flags in 0i32..4) {
        let (ours, theirs) =
            decode_both(&input, out, flags, SENTINEL_W as i32, c::oracle_s32_utf8_to_wchar);
        prop_assert_eq!(ours, theirs);
    }

    #[test]
    fn utf8_to_wchar_matches_c_u32(input in utf8_input(), out in out_len(), flags in 0i32..4) {
        let (ours, theirs) =
            decode_both(&input, out, flags, SENTINEL_W, c::oracle_u32_utf8_to_wchar);
        prop_assert_eq!(ours, theirs);
    }

    #[test]
    fn wchar_to_utf8_matches_c_s32(input in wide_input(), out in out_len(), flags in 0i32..4) {
        let input: Vec<i32> = input.iter().map(|&w| w as i32).collect();
        let (ours, theirs) = encode_both(&input, out, flags, c::oracle_s32_wchar_to_utf8);
        prop_assert_eq!(ours, theirs);
    }

    #[test]
    fn wchar_to_utf8_matches_c_u32(input in wide_input(), out in out_len(), flags in 0i32..4) {
        let (ours, theirs) = encode_both(&input, out, flags, c::oracle_u32_wchar_to_utf8);
        prop_assert_eq!(ours, theirs);
    }
}

#[test]
fn wchar_to_utf8_translates_nul() {
    let input = [0x61u32, 0, 0x62];
    let (ours, theirs) = encode_both(&input, Some(8), 0, c::oracle_u32_wchar_to_utf8);
    assert_eq!(ours, theirs);
    assert_eq!(ours.0, 3);
}

/// Tier 2 at the C boundary: NULL input, `insize` 0 (NUL-terminated input),
/// and the platform `wchar_t`, against the matching oracle.
#[cfg(not(windows))]
mod shim {
    use super::*;
    use obs_util::ffi::utf8 as rs;

    type W = libc::wchar_t;

    #[allow(clippy::unnecessary_cast)]
    fn signed() -> bool {
        (0 as W).wrapping_sub(1) < (0 as W)
    }

    unsafe fn oracle_decode(i: *const c_char, n: usize, o: *mut W, on: usize, f: i32) -> usize {
        // SAFETY: forwarded caller contract; `W` is 32 bits on non-Windows.
        unsafe {
            if signed() {
                c::oracle_s32_utf8_to_wchar(i, n, o.cast(), on, f)
            } else {
                c::oracle_u32_utf8_to_wchar(i, n, o.cast(), on, f)
            }
        }
    }

    unsafe fn oracle_encode(i: *const W, n: usize, o: *mut c_char, on: usize, f: i32) -> usize {
        // SAFETY: forwarded caller contract; `W` is 32 bits on non-Windows.
        unsafe {
            if signed() {
                c::oracle_s32_wchar_to_utf8(i.cast(), n, o, on, f)
            } else {
                c::oracle_u32_wchar_to_utf8(i.cast(), n, o, on, f)
            }
        }
    }

    #[test]
    fn null_input_is_zero() {
        let mut out = [0 as W; 4];
        let mut cout = [0 as c_char; 4];
        // SAFETY: NULL input is checked before anything is read.
        unsafe {
            assert_eq!(
                rs::utf8_to_wchar(core::ptr::null(), 0, out.as_mut_ptr(), 4, 0),
                0
            );
            assert_eq!(
                rs::wchar_to_utf8(core::ptr::null(), 0, cout.as_mut_ptr(), 4, 0),
                0
            );
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1024))]

        #[test]
        fn shim_decode_matches_c(
            input in utf8_input(),
            nul_terminated in any::<bool>(),
            out in out_len(),
            flags in 0i32..4,
        ) {
            let mut input = input;
            let insize = if nul_terminated { input.push(0); 0 } else { input.len() };
            if insize == 0 && !nul_terminated {
                input.push(0); // empty slice: pass "" like `with_c_input`
            }
            let n = out.unwrap_or(0);
            let mut ours = vec![SENTINEL_W as W; n];
            let mut theirs = vec![SENTINEL_W as W; n];
            let p = input.as_ptr().cast::<c_char>();
            let (o1, o2) = match out {
                Some(_) => (ours.as_mut_ptr(), theirs.as_mut_ptr()),
                None => (core::ptr::null_mut(), core::ptr::null_mut()),
            };
            // SAFETY: `p` holds `insize` bytes or is NUL-terminated; the
            // outputs are null or hold `n` elements.
            let (r1, r2) = unsafe {
                (rs::utf8_to_wchar(p, insize, o1, n, flags), oracle_decode(p, insize, o2, n, flags))
            };
            prop_assert_eq!((r1, ours), (r2, theirs));
        }

        #[test]
        fn shim_encode_matches_c(
            input in wide_input(),
            nul_terminated in any::<bool>(),
            out in out_len(),
            flags in 0i32..4,
        ) {
            let mut input: Vec<W> = input.into_iter().map(|w| w as W).collect();
            let insize = if nul_terminated { input.push(0); 0 } else { input.len() };
            if insize == 0 && !nul_terminated {
                input.push(0);
            }
            let n = out.unwrap_or(0);
            let mut ours = vec![SENTINEL_B as c_char; n];
            let mut theirs = vec![SENTINEL_B as c_char; n];
            let (o1, o2) = match out {
                Some(_) => (ours.as_mut_ptr(), theirs.as_mut_ptr()),
                None => (core::ptr::null_mut(), core::ptr::null_mut()),
            };
            // SAFETY: as in `shim_decode_matches_c`.
            let (r1, r2) = unsafe {
                (
                    rs::wchar_to_utf8(input.as_ptr(), insize, o1, n, flags),
                    oracle_encode(input.as_ptr(), insize, o2, n, flags),
                )
            };
            prop_assert_eq!((r1, ours), (r2, theirs));
        }
    }
}
