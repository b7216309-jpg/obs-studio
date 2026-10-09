//! Tier 1: safe-core tests. The `*_test` tests mirror
//! `test/cmocka/test_text_lookup.c` case for case; NULL arguments are
//! shim-only and covered in `text_lookup_parity.rs`.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use obs_c_oracle as _; // links the test allocator
use obs_util::text_lookup::TextLookup;

/// A temporary file holding `data`, removed on drop.
struct TempFile(PathBuf);

impl TempFile {
    fn new(data: &[u8]) -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("obs_tl_t1_{}_{n}.ini", std::process::id()));
        std::fs::write(&path, data).expect("write temp file");
        TempFile(path)
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn create(data: &[u8]) -> Option<TextLookup> {
    let f = TempFile::new(data);
    let mut l = TextLookup::new();
    l.add_file(&f.0).then_some(l)
}

fn get<'a>(l: &'a TextLookup, key: &str) -> Option<&'a [u8]> {
    l.get(key.as_bytes()).map(|v| v.to_bytes())
}

const INI1: &[u8] = b"# a comment line\n\
Key=\"Value\"\n\
Plain=hello\n\
Esc=\"a\\nb\\tc\"\n\
Quote=\"say \\\"hi\\\"\"\n\
Dup=\"first\"\n\
Dup=\"second\"\n\
Last=\"end\"";

const INI2: &[u8] = b"Key=\"Override\"\nExtra=\"more\"";

#[test]
fn create_and_getstr_test() {
    let l = create(INI1).expect("lookup");

    assert_eq!(get(&l, "Key"), Some(&b"Value"[..]));
    assert_eq!(get(&l, "Plain"), Some(&b"hello"[..]));
    assert_eq!(get(&l, "Esc"), Some(&b"a\nb\tc"[..]));
    assert_eq!(get(&l, "Quote"), Some(&b"say \"hi\""[..]));

    // later duplicate replaces the earlier one
    assert_eq!(get(&l, "Dup"), Some(&b"second"[..]));

    // final line without trailing newline is still read
    assert_eq!(get(&l, "Last"), Some(&b"end"[..]));

    assert_eq!(get(&l, "Missing"), None);
    assert_eq!(get(&l, "key"), None); // lookup is case sensitive
}

#[test]
fn add_overrides_test() {
    let f2 = TempFile::new(INI2);
    let mut l = create(INI1).expect("lookup");
    assert_eq!(get(&l, "Key"), Some(&b"Value"[..]));
    assert_eq!(get(&l, "Extra"), None);

    assert!(l.add_file(&f2.0));
    assert_eq!(get(&l, "Key"), Some(&b"Override"[..]));
    assert_eq!(get(&l, "Extra"), Some(&b"more"[..]));

    // keys absent from the second file survive
    assert_eq!(get(&l, "Plain"), Some(&b"hello"[..]));

    // adding a missing file fails and leaves existing entries intact
    let missing = std::env::temp_dir().join("obs_tl_t1_does_not_exist.ini");
    assert!(!l.add_file(&missing));
    assert_eq!(get(&l, "Key"), Some(&b"Override"[..]));
}

#[test]
fn missing_file_test() {
    let missing = std::env::temp_dir().join("obs_tl_t1_does_not_exist.ini");
    assert!(!TextLookup::new().add_file(&missing));
}

// null_lookup_test: shim only.

// Edge cases beyond the C test. All are characterized from the C code.

#[test]
fn empty_and_bom_only_files_fail() {
    assert!(create(b"").is_none());
    assert!(create(b"\xEF\xBB\xBF").is_none());
    let l = create(b"\xEF\xBB\xBFK=v").expect("lookup");
    assert_eq!(get(&l, "K"), Some(&b"v"[..]));
}

#[test]
fn text_stops_at_nul_and_cr_is_space() {
    let l = create(b"A=1\nB=2\0\nC=3").expect("lookup");
    assert_eq!(get(&l, "B"), Some(&b"2"[..]));
    assert_eq!(get(&l, "C"), None);

    // CRLF line endings: the CR becomes a space, which ends the value
    let l = create(b"A=one\r\nB=\"two\"\r\n").expect("lookup");
    assert_eq!(get(&l, "A"), Some(&b"one"[..]));
    assert_eq!(get(&l, "B"), Some(&b"two"[..]));

    // a file of only a NUL is read (no entries)
    assert!(create(b"\0abc").is_some());
}

#[test]
fn whitespace_is_significant() {
    // characterized, not endorsed: spaces are tokens
    let l = create(b"K = v\n  X=y\n").expect("lookup");
    assert_eq!(get(&l, "K"), Some(&b" "[..]));
    assert_eq!(get(&l, "X"), None);
    assert_eq!(get(&l, " "), Some(&b" "[..]));
}

#[test]
fn empty_string_value_ends_parsing() {
    // characterized, not endorsed
    let l = create(b"A=1\nB=\"\"\nC=3\n").expect("lookup");
    assert_eq!(get(&l, "A"), Some(&b"1"[..]));
    assert_eq!(get(&l, "B"), None);
    assert_eq!(get(&l, "C"), None);
}

#[test]
fn quote_at_line_end_ends_parsing() {
    // intentional difference: C's length wraps and it crashes
    let l = create(b"A=1\nB=\"\nC=3\n").expect("lookup");
    assert_eq!(get(&l, "A"), Some(&b"1"[..]));
    assert_eq!(get(&l, "C"), None);
    let l = create(b"A=1\nB=\"").expect("lookup");
    assert_eq!(get(&l, "A"), Some(&b"1"[..]));
}

#[test]
fn comments_and_misc() {
    let l = create(b"#only a comment\nA=x#tail\n\"Q K\"=\"v w\"\nB==c\nD\nE=f").expect("lookup");
    // a # ends a value
    assert_eq!(get(&l, "A"), Some(&b"x"[..]));
    // quoted names lose their quotes
    assert_eq!(get(&l, "Q K"), Some(&b"v w"[..]));
    // only the first = is skipped
    assert_eq!(get(&l, "B"), Some(&b"="[..]));
    // a name alone on its line is skipped
    assert_eq!(get(&l, "D"), None);
    assert_eq!(get(&l, "E"), Some(&b"f"[..]));
}

#[test]
fn unterminated_string_drops_trailing_escaped_quote() {
    // characterized, not endorsed
    let l = create(b"A=\"ab\\\"\nB=1").expect("lookup");
    assert_eq!(get(&l, "A"), Some(&b"ab\\"[..]));
    assert_eq!(get(&l, "B"), Some(&b"1"[..]));
}

#[test]
fn escapes_apply_in_order() {
    // "\\n" is a backslash then \n: the \n escape wins
    let l = create(b"A=\"x\\\\ny\"").expect("lookup");
    assert_eq!(get(&l, "A"), Some(&b"x\\\ny"[..]));
}

#[test]
fn values_stay_put_until_replaced() {
    let mut l = create(b"A=1\nB=2").expect("lookup");
    let a = l.get(b"A").unwrap().as_ptr();
    let extra = TempFile::new(b"B=3\nC=4\nD=5\nE=6\nF=7\nG=8\nH=9\nI=10\nJ=11");
    assert!(l.add_file(&extra.0));
    assert_eq!(l.get(b"A").unwrap().as_ptr(), a);
}
