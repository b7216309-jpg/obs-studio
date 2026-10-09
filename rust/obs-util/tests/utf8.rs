//! Tier 1: safe-core tests. The `test_utf8_*` tests mirror
//! `test/cmocka/test_utf8.c` case for case. The C test's `wchar_t` is the
//! signed `int` of x86 Linux, so they use `i32`.
//!
//! The safe core takes slices, so C calls with `insize` 0 (NUL-terminated
//! input) map to the input up to the first NUL, and NULL input has no
//! equivalent; `utf8_parity.rs` covers both at the shim.

use obs_c_oracle as _; // links the test allocator
use obs_util::utf8::{UTF8_IGNORE_ERROR, UTF8_SKIP_BOM, utf8_to_wchar, wchar_to_utf8};

fn wide(s: &str) -> Vec<i32> {
    s.chars().map(|c| c as i32).collect()
}

#[test]
fn test_utf8_ascii_round_trip() {
    let mut out = [0i32; 8];

    assert_eq!(utf8_to_wchar::<i32>(b"hello", None, 0), 5);
    assert_eq!(utf8_to_wchar(b"hello", Some(&mut out), 0), 5);
    assert_eq!(out[..5], wide("hello")[..]);

    let mut back = [0u8; 8];
    assert_eq!(wchar_to_utf8(&out[..5], None, 0), 5);
    assert_eq!(wchar_to_utf8(&out[..5], Some(&mut back), 0), 5);
    assert_eq!(&back[..6], b"hello\0");
}

#[test]
fn test_utf8_to_wchar_multibyte() {
    let mut out = [0i32; 4];

    assert_eq!(utf8_to_wchar(b"\xc3\xa9", Some(&mut out), 0), 1);
    assert_eq!(out[0], 0xE9);

    assert_eq!(utf8_to_wchar(b"\xe2\x82\xac", Some(&mut out), 0), 1);
    assert_eq!(out[0], 0x20AC);

    assert_eq!(utf8_to_wchar(b"\xf0\x9f\x98\x80", Some(&mut out), 0), 1);
    assert_eq!(out[0], 0x1F600);
}

#[test]
fn test_utf8_wchar_to_utf8_multibyte() {
    let mut out = [0u8; 8];
    assert_eq!(wchar_to_utf8(&[0xE9i32], Some(&mut out), 0), 2);
    assert_eq!(&out[..2], b"\xc3\xa9");

    let mut out = [0u8; 8];
    assert_eq!(wchar_to_utf8(&[0x20ACi32], Some(&mut out), 0), 3);
    assert_eq!(&out[..3], b"\xe2\x82\xac");

    let mut out = [0u8; 8];
    assert_eq!(wchar_to_utf8(&[0x1F600i32], Some(&mut out), 0), 4);
    assert_eq!(&out[..4], b"\xf0\x9f\x98\x80");
}

#[test]
fn test_utf8_errors() {
    let mut out = [0i32; 8];
    let mut cout = [0u8; 8];

    // forbidden octets
    assert_eq!(utf8_to_wchar(b"\xc0\x80", Some(&mut out), 0), 0);
    assert_eq!(utf8_to_wchar(b"\xff", Some(&mut out), 0), 0);
    // truncated sequence
    assert_eq!(utf8_to_wchar(b"\xe2\x82", Some(&mut out), 0), 0);
    // bad continuation
    assert_eq!(utf8_to_wchar(b"\xe2\x41\x41", Some(&mut out), 0), 0);
    // surrogate
    assert_eq!(wchar_to_utf8(&[0xD800i32], Some(&mut cout), 0), 0);
}

#[test]
fn test_utf8_ignore_error() {
    let mut out = [0i32; 8];
    let mut cout = [0u8; 8];

    assert_eq!(
        utf8_to_wchar(b"a\xffb", Some(&mut out), UTF8_IGNORE_ERROR),
        2
    );
    assert_eq!(out[..2], wide("ab")[..]);

    let input = [0x61i32, 0xD800, 0x62];
    assert_eq!(wchar_to_utf8(&input, Some(&mut cout), UTF8_IGNORE_ERROR), 2);
    assert_eq!(&cout[..2], b"ab");
}

#[test]
fn test_utf8_skip_bom() {
    let mut out = [0i32; 8];
    assert_eq!(
        utf8_to_wchar(b"\xef\xbb\xbfx", Some(&mut out), UTF8_SKIP_BOM),
        1
    );
    assert_eq!(out[0], 'x' as i32);

    let mut out = [0i32; 8];
    assert_eq!(utf8_to_wchar(b"\xef\xbb\xbfx", Some(&mut out), 0), 2);
    assert_eq!(out[0], 0xFEFF);
    assert_eq!(out[1], 'x' as i32);
}

#[test]
fn test_utf8_invalid_arguments() {
    let mut out = [0i32; 8];
    let mut cout = [0u8; 8];

    // utf8_to_wchar(NULL, ...) and wchar_to_utf8(NULL, ...): shim only.
    assert_eq!(utf8_to_wchar(b"abc", Some(&mut out[..0]), 0), 0);
    // output buffer too small
    assert_eq!(utf8_to_wchar(b"abc", Some(&mut out[..2]), 0), 0);

    assert_eq!(wchar_to_utf8(&wide("abc"), Some(&mut cout[..0]), 0), 0);
    assert_eq!(wchar_to_utf8(&wide("abc"), Some(&mut cout[..2]), 0), 0);
}

#[test]
fn test_utf8_embedded_nul() {
    let mut out = [0i32; 8];

    // insize 0 stops at NUL: the core gets the input up to the NUL.
    assert_eq!(utf8_to_wchar::<i32>(b"a", None, 0), 1);
    // insize > 0 translates NUL as a regular symbol
    assert_eq!(utf8_to_wchar(b"a\0b", Some(&mut out), 0), 3);
    assert_eq!(out[0], 'a' as i32);
    assert_eq!(out[1], 0);
    assert_eq!(out[2], 'b' as i32);
}

// Edge cases beyond the C test. All are characterized from the C code.

#[test]
fn empty_input_is_zero() {
    assert_eq!(utf8_to_wchar::<i32>(b"", None, 0), 0);
    assert_eq!(wchar_to_utf8::<i32>(&[], None, 0), 0);
}

#[test]
fn counting_skips_surrogate_and_bom_checks() {
    // Without an output buffer, decoded surrogates and BOMs are counted, not
    // rejected (the documented CAVEAT in utf8.c).
    assert_eq!(utf8_to_wchar::<i32>(b"\xed\xa0\x80", None, 0), 1);
    assert_eq!(
        utf8_to_wchar::<i32>(b"\xef\xbb\xbf", None, UTF8_SKIP_BOM),
        1
    );

    let mut out = [0i32; 4];
    assert_eq!(utf8_to_wchar(b"\xed\xa0\x80", Some(&mut out), 0), 0);
    assert_eq!(
        utf8_to_wchar(b"\xef\xbb\xbf", Some(&mut out), UTF8_SKIP_BOM),
        0
    );
}

#[test]
fn ignored_surrogate_stays_in_buffer_past_count() {
    let mut out = [0i32; 4];
    assert_eq!(
        utf8_to_wchar(b"a\xed\xa0\x80", Some(&mut out), UTF8_IGNORE_ERROR),
        1
    );
    assert_eq!(out[..2], [0x61, 0xD800]);
}

#[test]
fn full_buffer_fails_even_when_ignoring_errors() {
    let mut out = [0i32; 1];
    // The space check runs before the surrogate is decoded and dropped.
    assert_eq!(
        utf8_to_wchar(b"a\xed\xa0\x80", Some(&mut out), UTF8_IGNORE_ERROR),
        0
    );
    // A sequence rejected before decoding does not need space.
    assert_eq!(
        utf8_to_wchar(b"a\xff", Some(&mut out), UTF8_IGNORE_ERROR),
        1
    );
}

#[test]
fn error_keeps_partial_output() {
    let mut out = [0i32; 4];
    assert_eq!(utf8_to_wchar(b"ab\xff", Some(&mut out), 0), 0);
    assert_eq!(out[..2], [0x61, 0x62]);

    let mut cout = [0u8; 4];
    assert_eq!(wchar_to_utf8(&[0x61i32, 0xD800], Some(&mut cout), 0), 0);
    assert_eq!(cout[0], b'a');
}

#[test]
fn overlong_and_out_of_range_sequences_decode() {
    let mut out = [0i32; 4];
    assert_eq!(utf8_to_wchar(b"\xe0\x80\x80", Some(&mut out), 0), 1);
    assert_eq!(out[0], 0);
    assert_eq!(utf8_to_wchar(b"\xf4\x90\x80\x80", Some(&mut out), 0), 1);
    assert_eq!(out[0], 0x11_0000);
    assert_eq!(utf8_to_wchar(b"\xf8\x88\x80\x80\x80", Some(&mut out), 0), 1);
    assert_eq!(out[0], 0x20_0000);
    assert_eq!(
        utf8_to_wchar(b"\xfd\xbf\xbf\xbf\xbf\xbf", Some(&mut out), 0),
        1
    );
    assert_eq!(out[0], 0x7FFF_FFFF);
}

#[test]
fn invalid_lead_bytes() {
    // A lone continuation byte, 0xfe, and the forbidden 0xc1 and 0xf5.
    for bad in [&b"\x80"[..], b"\xfe", b"\xc1\x80", b"\xf5\x80\x80\x80"] {
        assert_eq!(utf8_to_wchar::<i32>(bad, None, 0), 0, "{bad:x?}");
    }
    // Ignoring errors also disables the forbidden-octet check, so the
    // overlong 0xc1 0x80 decodes to U+0040.
    let mut out = [0i32; 2];
    assert_eq!(
        utf8_to_wchar(b"\xc1\x80", Some(&mut out), UTF8_IGNORE_ERROR),
        1
    );
    assert_eq!(out[0], 0x40);
    // A truncated sequence skips one byte, then resyncs.
    assert_eq!(
        utf8_to_wchar::<i32>(b"\xe2\x41", None, UTF8_IGNORE_ERROR),
        1
    );
}

#[test]
fn encode_five_and_six_byte_forms() {
    let mut out = [0u8; 8];
    assert_eq!(wchar_to_utf8(&[0x20_0000i32], Some(&mut out), 0), 5);
    assert_eq!(&out[..5], b"\xf8\x88\x80\x80\x80");
    assert_eq!(wchar_to_utf8(&[0x7FFF_FFFFi32], Some(&mut out), 0), 6);
    assert_eq!(&out[..6], b"\xfd\xbf\xbf\xbf\xbf\xbf");
}

#[test]
fn encode_negative_wchar_depends_on_signedness() {
    let mut out = [0u8; 8];
    // Signed wchar_t: negative is an error, or skipped.
    assert_eq!(wchar_to_utf8(&[i32::MIN], Some(&mut out), 0), 0);
    assert_eq!(
        wchar_to_utf8(&[i32::MIN, 0x61], Some(&mut out), UTF8_IGNORE_ERROR),
        1
    );
    // Unsigned wchar_t: six bytes, and bit 31 is dropped.
    assert_eq!(wchar_to_utf8(&[0x8000_0000u32], Some(&mut out), 0), 6);
    assert_eq!(&out[..6], b"\xfc\x80\x80\x80\x80\x80");
    assert_eq!(wchar_to_utf8(&[u32::MAX], Some(&mut out), 0), 6);
    assert_eq!(&out[..6], b"\xfd\xbf\xbf\xbf\xbf\xbf");
}

#[test]
fn encode_skip_bom_and_space() {
    let mut out = [0u8; 8];
    assert_eq!(
        wchar_to_utf8(&[0xFEFFi32, 0x61], Some(&mut out), UTF8_SKIP_BOM),
        1
    );
    assert_eq!(out[0], b'a');
    assert_eq!(wchar_to_utf8(&[0xFEFFi32], None, 0), 3);
    // Exactly enough space, then one byte short.
    assert_eq!(wchar_to_utf8(&[0x20ACi32], Some(&mut out[..3]), 0), 3);
    assert_eq!(wchar_to_utf8(&[0x20ACi32], Some(&mut out[..2]), 0), 0);
    // Counting ignores space but still rejects surrogates.
    assert_eq!(wchar_to_utf8(&[0xDFFFi32], None, 0), 0);
}

#[test]
fn round_trips_valid_strings() {
    let s = "héllo, wörld € 😀 \u{FFFF} \u{10FFFF}";
    let mut w = [0i32; 64];
    let n = utf8_to_wchar(s.as_bytes(), Some(&mut w), 0);
    assert_eq!(n, s.chars().count());
    let mut back = [0u8; 64];
    let m = wchar_to_utf8(&w[..n], Some(&mut back), 0);
    assert_eq!(&back[..m], s.as_bytes());
}
