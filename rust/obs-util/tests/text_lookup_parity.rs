//! Tier 3: the Rust C ABI shim (and through it the safe core) behaves
//! exactly like the original `util/text-lookup.c`, compiled as an oracle
//! on the lexer and dstr oracles (with test-only `os_fopen`/`os_fread_utf8`
//! and `uthash` stand-ins).
//!
//! Random locale-like files are written to temp files and loaded by both;
//! `text_lookup_create` must agree on success, and `text_lookup_getstr` on
//! every candidate key (each word of the files, plus whitespace and
//! punctuation keys), before and after a second file is added.
//!
//! Intentional difference, excluded here: a quote right before a newline or
//! the end of the text makes C's string length wrap around (and crash);
//! generated files never have one.
//!
//! Reminder: the mutation check (break the core, see this file fail) is done
//! once per port, not on every change.

use core::ffi::{CStr, c_char, c_void};
use core::ptr;
use std::collections::BTreeSet;
use std::ffi::CString;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use obs_c_oracle::text_lookup as c;
use obs_util::ffi::text_lookup as rs;
use proptest::prelude::*;

struct TempFile(PathBuf);

impl TempFile {
    fn new(data: &[u8]) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("obs_tl_t3_{}_{n}.ini", std::process::id()));
        std::fs::write(&path, data).expect("write temp file");
        TempFile(path)
    }

    fn c_path(&self) -> CString {
        CString::new(self.0.to_str().expect("ASCII temp path")).unwrap()
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

const PIECES: &[&[u8]] = &[
    b"Key",
    b"A",
    b"b_1",
    b"Name.Sub",
    b"x",
    b"=",
    b"==",
    b"\"",
    b"\"v w\"",
    b"\\n",
    b"\\t",
    b"\\r",
    b"\\\"",
    b"\\\\",
    b"\\",
    b" ",
    b"\t",
    b"\n",
    b"\r\n",
    b"\r",
    b"#",
    b"# c\n",
    b"1",
    b"\xc3\xa9",
];

/// Locale-like text: mostly `key=value` lines, with every special character
/// mixed in. Never a quote right before a newline or the end.
fn file() -> impl Strategy<Value = Vec<u8>> {
    let piece = prop_oneof![
        4 => prop::sample::select(PIECES).prop_map(<[u8]>::to_vec),
        2 => (prop::sample::select(&PIECES[..5]), prop::sample::select(PIECES))
            .prop_map(|(k, v)| [k, b"=", v, b"\n"].concat()),
        1 => any::<u8>().prop_map(|b| vec![b]),
    ];
    (any::<bool>(), prop::collection::vec(piece, 0..30)).prop_map(|(bom, pieces)| {
        let mut v = if bom {
            b"\xEF\xBB\xBF".to_vec()
        } else {
            Vec::new()
        };
        v.extend(pieces.concat());
        // C crashes on a quote right before LF, NUL or the end (CR becomes
        // a space, so it is fine).
        let mut out = Vec::with_capacity(v.len());
        for (i, &b) in v.iter().enumerate() {
            out.push(b);
            if b == b'"' && matches!(v.get(i + 1), None | Some(b'\n') | Some(0)) {
                out.push(b'q');
            }
        }
        out
    })
}

/// Every word of `data`, plus keys the quirks can produce.
fn candidate_keys(files: &[&[u8]]) -> BTreeSet<Vec<u8>> {
    let mut keys: BTreeSet<Vec<u8>> = [&b" "[..], b"\t", b"\n", b"=", b"#", b"\"", b"", b"Missing"]
        .iter()
        .map(|k| k.to_vec())
        .collect();
    for data in files {
        for word in data.split(|&b| matches!(b, b'\n' | b'=' | b' ' | b'\t' | b'#' | b'\r' | 0)) {
            if !word.is_empty() {
                keys.insert(word.to_vec());
                keys.insert(word.iter().copied().filter(|&b| b != b'"').collect());
            }
        }
    }
    keys
}

fn get(
    f: unsafe extern "C" fn(*mut c_void, *const c_char, *mut *const c_char) -> bool,
    lookup: *mut c_void,
    key: &CStr,
) -> Option<Vec<u8>> {
    let mut out: *const c_char = ptr::null();
    // SAFETY: `lookup` is NULL or live, `key` a C string, `out` writable.
    unsafe { f(lookup, key.as_ptr(), &mut out).then(|| CStr::from_ptr(out).to_bytes().to_vec()) }
}

fn compare(r: *mut rs::lookup_t, o: *mut c_void, files: &[&[u8]]) -> Result<(), TestCaseError> {
    for key in candidate_keys(files) {
        let Ok(key) = CString::new(key) else { continue };
        let ours = {
            let mut out: *const c_char = ptr::null();
            // SAFETY: `r` is NULL or live, `key` a C string, `out` writable.
            unsafe {
                rs::text_lookup_getstr(r, key.as_ptr(), &mut out)
                    .then(|| CStr::from_ptr(out).to_bytes().to_vec())
            }
        };
        let theirs = get(c::oracle_text_lookup_getstr, o, &key);
        prop_assert_eq!(ours, theirs, "key {:?}", key);
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1024))]

    #[test]
    fn text_lookup_matches_c(first in file(), second in file()) {
        let (f1, f2) = (TempFile::new(&first), TempFile::new(&second));
        let (p1, p2) = (f1.c_path(), f2.c_path());
        // SAFETY: valid C paths; lookups are destroyed once.
        unsafe {
            let r = rs::text_lookup_create(p1.as_ptr());
            let o = c::oracle_text_lookup_create(p1.as_ptr());
            prop_assert_eq!(r.is_null(), o.is_null());
            compare(r, o, &[&first])?;

            if !r.is_null() {
                prop_assert_eq!(
                    rs::text_lookup_add(r, p2.as_ptr()),
                    c::oracle_text_lookup_add(o, p2.as_ptr())
                );
                compare(r, o, &[&first, &second])?;
            }
            rs::text_lookup_destroy(r);
            c::oracle_text_lookup_destroy(o);
        }
    }
}

#[test]
fn null_and_missing() {
    let missing = CString::new(
        std::env::temp_dir()
            .join("obs_tl_t3_missing.ini")
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let mut out: *const c_char = ptr::null();
    // SAFETY: NULL lookups and paths are handled first by both.
    unsafe {
        assert!(rs::text_lookup_create(ptr::null()).is_null());
        assert!(c::oracle_text_lookup_create(ptr::null()).is_null());
        assert!(rs::text_lookup_create(missing.as_ptr()).is_null());
        assert!(c::oracle_text_lookup_create(missing.as_ptr()).is_null());
        assert!(!rs::text_lookup_getstr(
            ptr::null_mut(),
            c"Key".as_ptr(),
            &mut out
        ));
        assert!(!c::oracle_text_lookup_getstr(
            ptr::null_mut(),
            c"Key".as_ptr(),
            &mut out
        ));
        assert!(out.is_null());
        rs::text_lookup_destroy(ptr::null_mut());
        c::oracle_text_lookup_destroy(ptr::null_mut());
    }
}
