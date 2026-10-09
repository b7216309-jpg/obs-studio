//! Tier 3: the Rust C ABI shim (and through it the safe core) behaves
//! exactly like the original `util/lexer.c`, compiled as an oracle.
//!
//! Inputs are arbitrary bytes, weighted toward the characters the lexer
//! treats specially (signs, `.`, `e`, CR/LF, case pairs, NUL, high bytes).
//!
//! Intentional difference: `lexer_getstroffset` with a NULL `lex->text`
//! leaves the outputs untouched (C dereferences NULL), so it is not
//! compared.
//!
//! Reminder: the mutation check (break the core, see this file fail) is done
//! once per port, not on every change.

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use obs_c_oracle::lexer as c;
use obs_util::ffi::darray::bfree;
use obs_util::ffi::lexer as rs;
use proptest::prelude::*;

const SPECIAL: &[u8] = b"09aAzZeE+-. \t\r\n\0x_\x80\xc3\xe9\xff";

fn bytes(max: usize) -> impl Strategy<Value = Vec<u8>> {
    prop_oneof![
        prop::collection::vec(any::<u8>(), 0..max),
        prop::collection::vec(prop::sample::select(SPECIAL), 0..max),
    ]
}

/// Bytes without NUL, for C strings.
fn text(max: usize) -> impl Strategy<Value = Vec<u8>> {
    bytes(max).prop_map(|mut v| {
        v.retain(|&b| b != 0);
        v
    })
}

/// Strings shaped like numbers, `[sign] digits [. digits] [e [sign] digits]
/// [sign]`, with each part optional and an occasional stray character, so
/// every branch of the validators is reached.
fn number_like() -> impl Strategy<Value = Vec<u8>> {
    let sign = prop::option::of(prop::sample::select(&b"+-"[..]));
    let digits = prop::collection::vec(b'0'..=b'9', 0..3);
    let stray = prop::option::weighted(0.2, prop::sample::select(&b".e+-x\x80"[..]));
    (
        sign.clone(),
        digits.clone(),
        prop::option::of(digits.clone()),
        prop::option::of((sign.clone(), digits)),
        sign,
        stray,
        any::<prop::sample::Index>(),
    )
        .prop_map(|(s1, d1, frac, exp, s3, stray, at)| {
            let mut v: Vec<u8> = s1.into_iter().chain(d1).collect();
            if let Some(f) = frac {
                v.push(b'.');
                v.extend(f);
            }
            if let Some((s2, e)) = exp {
                v.push(b'e');
                v.extend(s2);
                v.extend(e);
            }
            v.extend(s3);
            if let Some(c) = stray {
                v.insert(at.index(v.len() + 1), c);
            }
            v
        })
}

fn nul_terminated(v: &[u8]) -> Vec<u8> {
    let mut v = v.to_vec();
    v.push(0);
    v
}

/// A segment `buf[..len]` of a larger buffer, or NULL.
fn segment() -> impl Strategy<Value = Option<(Vec<u8>, usize)>> {
    prop::option::weighted(
        0.95,
        bytes(12).prop_flat_map(|buf| {
            let n = buf.len();
            (Just(buf), 0..=n)
        }),
    )
}

fn as_strrefs(seg: &Option<(Vec<u8>, usize)>) -> (rs::strref, c::OracleStrref) {
    let (array, len) = match seg {
        Some((buf, len)) => (buf.as_ptr().cast::<c_char>(), *len),
        None => (ptr::null(), 0),
    };
    (rs::strref { array, len }, c::OracleStrref { array, len })
}

/// A NUL-terminated C string, or NULL (`None`).
fn cstring() -> impl Strategy<Value = Option<Vec<u8>>> {
    prop::option::weighted(0.95, text(12).prop_map(|v| nul_terminated(&v)))
}

fn cptr(s: &Option<Vec<u8>>) -> *const c_char {
    s.as_ref().map_or(ptr::null(), |v| v.as_ptr().cast())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4096))]

    #[test]
    fn strref_cmp_matches_c(s1 in segment(), s2 in cstring()) {
        let (r, o) = as_strrefs(&s1);
        let p2 = cptr(&s2);
        // SAFETY: the strrefs cover live buffers and `p2` is NULL or a C string.
        unsafe {
            prop_assert_eq!(rs::strref_cmp(&r, p2), c::oracle_strref_cmp(&o, p2));
            prop_assert_eq!(rs::strref_cmpi(&r, p2), c::oracle_strref_cmpi(&o, p2));
        }
    }

    #[test]
    fn strref_cmp_strref_matches_c(s1 in segment(), s2 in segment()) {
        let (r1, o1) = as_strrefs(&s1);
        let (r2, o2) = as_strrefs(&s2);
        // SAFETY: the strrefs cover live buffers.
        unsafe {
            prop_assert_eq!(
                rs::strref_cmp_strref(&r1, &r2),
                c::oracle_strref_cmp_strref(&o1, &o2)
            );
            prop_assert_eq!(
                rs::strref_cmpi_strref(&r1, &r2),
                c::oracle_strref_cmpi_strref(&o1, &o2)
            );
        }
    }

    #[test]
    fn strref_cmp_equal_prefixes(common in text(8), a in bytes(4), b in text(4)) {
        // Shared prefixes reach the length and terminator logic more often;
        // `a` may hold NULs, which the strref compares as bytes.
        let x = [common.clone(), a].concat();
        let y = nul_terminated(&[common, b].concat());
        let (r, o) = as_strrefs(&Some((x.clone(), x.len())));
        // SAFETY: as above.
        unsafe {
            prop_assert_eq!(
                rs::strref_cmp(&r, y.as_ptr().cast()),
                c::oracle_strref_cmp(&o, y.as_ptr().cast())
            );
        }
    }

    #[test]
    fn valid_number_like_str_matches_c(s in number_like(), n in 0usize..16) {
        let buf = nul_terminated(&s);
        let p: *const c_char = buf.as_ptr().cast();
        // SAFETY: `p` is a NUL-terminated buffer.
        unsafe {
            prop_assert_eq!(rs::valid_int_str(p, n), c::oracle_valid_int_str(p, n));
            prop_assert_eq!(rs::valid_float_str(p, n), c::oracle_valid_float_str(p, n));
        }
    }

    #[test]
    fn valid_number_str_matches_c(s in bytes(12), n in 0usize..16, null in prop::bool::weighted(0.05)) {
        // Always NUL-terminated past the end, so C never reads past it.
        let buf = nul_terminated(&s);
        let p: *const c_char = if null { ptr::null() } else { buf.as_ptr().cast() };
        // SAFETY: `p` is NULL or a NUL-terminated buffer.
        unsafe {
            prop_assert_eq!(rs::valid_int_str(p, n), c::oracle_valid_int_str(p, n));
            prop_assert_eq!(rs::valid_float_str(p, n), c::oracle_valid_float_str(p, n));
        }
    }

    #[test]
    fn lexer_getbasetoken_matches_c(input in text(48), modes in prop::collection::vec(any::<bool>(), 1..64)) {
        let buf = nul_terminated(&input);
        let start = buf.as_ptr().cast::<c_char>();
        let mut r = rs::lexer { text: ptr::null_mut(), offset: start };
        let mut o = c::OracleLexer { text: ptr::null_mut(), offset: start };

        for ignore in modes {
            let iws = c_int::from(ignore);
            let mut rt = rs::base_token {
                text: rs::strref { array: ptr::null(), len: 7 },
                r#type: 9,
                passed_whitespace: true,
            };
            let mut ot = c::OracleBaseToken {
                text: c::OracleStrref { array: ptr::null(), len: 7 },
                kind: 9,
                passed_whitespace: true,
            };
            // SAFETY: both lexers point into `buf`, NUL-terminated.
            let (got_r, got_o) = unsafe {
                (
                    rs::lexer_getbasetoken(&mut r, &mut rt, iws),
                    c::oracle_lexer_getbasetoken(&mut o, &mut ot, iws),
                )
            };
            prop_assert_eq!(got_r, got_o);
            prop_assert_eq!(r.offset, o.offset);
            prop_assert_eq!(
                (rt.text.array, rt.text.len, rt.r#type, rt.passed_whitespace),
                (ot.text.array, ot.text.len, ot.kind, ot.passed_whitespace)
            );
        }
    }

    #[test]
    fn lexer_getstroffset_matches_c(input in text(48), pos in any::<prop::sample::Index>()) {
        let mut buf = nul_terminated(&input);
        let text = buf.as_mut_ptr().cast::<c_char>();
        let at = pos.index(buf.len()); // up to and including the NUL
        let r = rs::lexer { text, offset: text };
        let o = c::OracleLexer { text, offset: text };
        let (mut rr, mut rc, mut or, mut oc) = (0u32, 0u32, 0u32, 0u32);
        // SAFETY: `at` is within the buffer, at most the NUL.
        unsafe {
            let s = text.add(at);
            rs::lexer_getstroffset(&r, s, &mut rr, &mut rc);
            c::oracle_lexer_getstroffset(&o, s, &mut or, &mut oc);
        }
        prop_assert_eq!((rr, rc), (or, oc));
    }

    #[test]
    fn error_data_matches_c(
        adds in prop::collection::vec(
            (cstring(), any::<u32>(), any::<u32>(), cstring(), -2i32..3),
            0..12,
        ),
    ) {
        let mut r = rs::error_data { errors: obs_util::ffi::darray::darray {
            array: ptr::null_mut(), num: 0, capacity: 0,
        }};
        let mut o = c::OracleErrorData { errors: obs_c_oracle::darray::OracleDarray {
            array: ptr::null_mut(), num: 0, capacity: 0,
        }};

        for (file, row, col, msg, level) in &adds {
            // SAFETY: both containers are valid and the strings are NULL or C
            // strings that outlive the containers.
            unsafe {
                rs::error_data_add(&mut r, cptr(file), *row, *col, cptr(msg), *level);
                c::oracle_error_data_add(&mut o, cptr(file), *row, *col, cptr(msg), *level);
            }
        }
        // Adding to NULL is a no-op on both sides.
        // SAFETY: NULL `data` is checked first.
        unsafe {
            rs::error_data_add(ptr::null_mut(), ptr::null(), 0, 0, ptr::null(), 0);
            c::oracle_error_data_add(ptr::null_mut(), ptr::null(), 0, 0, ptr::null(), 0);
        }

        prop_assert_eq!(r.errors.num, o.errors.num);
        prop_assert_eq!(r.errors.capacity, o.errors.capacity);

        // SAFETY: both arrays hold `num` items; strings are NULL or bmalloc'd
        // C strings; every bmalloc'd pointer is freed exactly once.
        unsafe {
            let ri = items::<rs::error_item>(r.errors.array, r.errors.num);
            let oi = items::<c::OracleErrorItem>(o.errors.array, o.errors.num);
            let s = |p: *const c_char| (!p.is_null()).then(|| CStr::from_ptr(p).to_owned());
            for (a, b) in ri.iter().zip(oi) {
                prop_assert_eq!(
                    (s(a.error), a.file, a.row, a.column, a.level),
                    (s(b.error), b.file, b.row, b.column, b.level)
                );
                // The message is a copy, never the caller's pointer.
                prop_assert!(a.error.is_null() || a.error.cast_const() != a.file);
            }

            let rstr = rs::error_data_buildstring(&mut r);
            let ostr = c::oracle_error_data_buildstring(&mut o);
            prop_assert_eq!(s(rstr), s(ostr));

            for p in [rstr, ostr] {
                bfree(p.cast::<c_void>());
            }
            for item in ri {
                bfree(item.error.cast::<c_void>());
            }
            for item in oi {
                bfree(item.error.cast::<c_void>());
            }
            bfree(r.errors.array);
            bfree(o.errors.array);
        }
    }
}

/// The items of a darray; its array is NULL while empty.
///
/// # Safety
///
/// `array` must be NULL with `num` 0, or hold `num` items of `T`.
unsafe fn items<'a, T>(array: *mut c_void, num: usize) -> &'a [T] {
    if array.is_null() {
        &[]
    } else {
        // SAFETY: per the contract.
        unsafe { core::slice::from_raw_parts(array.cast::<T>(), num) }
    }
}

#[test]
fn null_offset_and_null_str() {
    let mut r = rs::lexer {
        text: ptr::null_mut(),
        offset: ptr::null(),
    };
    let mut o = c::OracleLexer {
        text: ptr::null_mut(),
        offset: ptr::null(),
    };
    let mut rt = rs::base_token {
        text: rs::strref {
            array: ptr::null(),
            len: 0,
        },
        r#type: 0,
        passed_whitespace: false,
    };
    let mut ot = c::OracleBaseToken {
        text: c::OracleStrref {
            array: ptr::null(),
            len: 0,
        },
        kind: 0,
        passed_whitespace: false,
    };
    // SAFETY: a NULL offset is checked before anything is read.
    unsafe {
        assert!(!rs::lexer_getbasetoken(&mut r, &mut rt, 0));
        assert!(!c::oracle_lexer_getbasetoken(&mut o, &mut ot, 0));
    }

    let mut buf = *b"ab\0";
    let text = buf.as_mut_ptr().cast::<c_char>();
    let r = rs::lexer { text, offset: text };
    let o = c::OracleLexer { text, offset: text };
    let (mut rr, mut rc, mut or, mut oc) = (99u32, 98u32, 99u32, 98u32);
    // SAFETY: a NULL `str` is checked before anything is read.
    unsafe {
        rs::lexer_getstroffset(&r, ptr::null(), &mut rr, &mut rc);
        c::oracle_lexer_getstroffset(&o, ptr::null(), &mut or, &mut oc);
    }
    assert_eq!((rr, rc), (99, 98));
    assert_eq!((or, oc), (99, 98));
}
