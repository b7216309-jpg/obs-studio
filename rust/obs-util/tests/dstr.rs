//! Tier 1: safe-core tests. The `test_*` tests mirror the cases of
//! `test/cmocka/test_dstr.c` that cover `dstr.c`; `test_dstr_case`,
//! `test_dstr_printf` and `test_dstr_inline_helpers` cover `dstr-libc.c`
//! and `dstr.h`, which stay in C.
//!
//! A `VecDstr` stands in for `struct dstr`. NULL strings are `None` (or an
//! empty slice where C treats them alike); pointer results are offsets.

use obs_c_oracle as _; // links the test allocator
use obs_util::dstr::*;

fn s(d: &VecDstr) -> Option<&[u8]> {
    d.c_str()
}

#[test]
fn test_astrcmpi() {
    assert_eq!(astrcmpi(b"abc", b"abc"), 0);
    assert_eq!(astrcmpi(b"abc", b"ABC"), 0);
    assert_eq!(astrcmpi(b"aBc", b"AbC"), 0);
    assert_eq!(astrcmpi(b"a", b"b"), -1);
    assert_eq!(astrcmpi(b"B", b"a"), 1);
    assert_eq!(astrcmpi(b"ab", b"abc"), -1);
    assert_eq!(astrcmpi(b"abc", b"ab"), 1);
    assert_eq!(astrcmpi(b"", b""), 0);

    // NULL is treated as the empty string
    assert_eq!(astrcmpi(b"", b"a"), -1);
    assert_eq!(astrcmpi(b"a", b""), 1);
}

#[test]
fn test_astrcmp_n() {
    // n == 0 always equal
    assert_eq!(astrcmp_n(b"a", b"b", 0), 0);

    assert_eq!(astrcmp_n(b"abcd", b"abce", 3), 0);
    assert_eq!(astrcmp_n(b"abcd", b"abce", 4), -1);
    assert_eq!(astrcmp_n(b"abce", b"abcd", 4), 1);
    assert_eq!(astrcmp_n(b"abc", b"abc", 100), 0);
    assert_eq!(astrcmp_n(b"ab", b"abc", 100), -1);

    // case sensitive
    assert_eq!(astrcmp_n(b"A", b"a", 1), -1);
    assert_eq!(astrcmp_n(b"a", b"A", 1), 1);

    // NULL is treated as the empty string
    assert_eq!(astrcmp_n(b"", b"", 5), 0);
    assert_eq!(astrcmp_n(b"", b"a", 1), -1);
    assert_eq!(astrcmp_n(b"a", b"", 1), 1);
}

#[test]
fn test_astrcmpi_n() {
    assert_eq!(astrcmpi_n(b"a", b"b", 0), 0);
    assert_eq!(astrcmpi_n(b"ABCD", b"abce", 3), 0);
    assert_eq!(astrcmpi_n(b"ABCD", b"abce", 4), -1);
    assert_eq!(astrcmpi_n(b"abce", b"ABCD", 4), 1);
    assert_eq!(astrcmpi_n(b"ABC", b"abc", 100), 0);
    assert_eq!(astrcmpi_n(b"AB", b"abc", 100), -1);
    assert_eq!(astrcmpi_n(b"", b"", 5), 0);
    assert_eq!(astrcmpi_n(b"", b"a", 1), -1);
    assert_eq!(astrcmpi_n(b"a", b"", 1), 1);
}

#[test]
fn test_astrstri() {
    let str = b"Hello World";
    assert_eq!(astrstri(str, b"WORLD"), Some(6));
    assert_eq!(astrstri(str, b"llo"), Some(2));
    assert_eq!(astrstri(str, b"hello world"), Some(0));
    assert_eq!(astrstri(str, b"d"), Some(10));
    assert_eq!(astrstri(str, b"xyz"), None);
    assert_eq!(astrstri(str, b"Hello World!"), None);
    assert_eq!(astrstri(b"", b"a"), None);
    // empty needle matches at the start
    assert_eq!(astrstri(str, b""), Some(0));
    // NULL arguments: shim only
}

fn depad(text: &[u8]) -> Vec<u8> {
    let mut buf = text.to_vec();
    buf.push(0);
    strdepad(&mut buf);
    buf.truncate(buf.iter().position(|&b| b == 0).unwrap());
    buf
}

#[test]
fn test_strdepad() {
    assert_eq!(depad(b"  \t hi there \r\n"), b"hi there");
    assert_eq!(depad(b"nopad"), b"nopad");
    assert_eq!(depad(b"   lead"), b"lead");
    assert_eq!(depad(b"trail \t\n"), b"trail");
    assert_eq!(depad(b" \t\r\n "), b"");
    assert_eq!(depad(b""), b"");
    // NULL: shim only
}

#[test]
fn test_strlist_split() {
    fn v(s: &[u8], inc: bool) -> Vec<&[u8]> {
        strlist_split(s, b',', inc)
    }

    assert_eq!(v(b"a,b,,c", true), [&b"a"[..], b"b", b"", b"c"]);
    assert_eq!(v(b"a,b,,c", false), [&b"a"[..], b"b", b"c"]);

    // trailing separator
    assert_eq!(v(b"a,b,", true), [&b"a"[..], b"b", b""]);
    assert_eq!(v(b"a,b,", false), [&b"a"[..], b"b"]);

    // leading separator
    assert_eq!(v(b",a", true), [&b""[..], b"a"]);

    // no separator present
    assert_eq!(v(b"abc", false), [&b"abc"[..]]);

    // empty input
    assert_eq!(v(b"", true), [&b""[..]]);
    assert!(v(b"", false).is_empty());

    // NULL input: shim only
}

#[test]
fn test_dstr_copy() {
    let mut d = VecDstr::default();

    dstr_copy(&mut d, Some(b"hello"));
    assert_eq!(s(&d), Some(&b"hello"[..]));
    assert_eq!(d.len, 5);
    assert!(d.capacity > d.len);

    // copy shorter over longer
    dstr_copy(&mut d, Some(b"hi"));
    assert_eq!(s(&d), Some(&b"hi"[..]));
    assert_eq!(d.len, 2);
    assert!(d.capacity > d.len);

    // copy longer
    dstr_copy(&mut d, Some(b"a considerably longer string"));
    assert_eq!(s(&d), Some(&b"a considerably longer string"[..]));
    assert_eq!(d.len, 28);
    assert!(d.capacity > d.len);

    // NULL and empty free the destination
    dstr_copy(&mut d, None);
    assert_eq!((s(&d), d.len, d.capacity), (None, 0, 0));

    dstr_copy(&mut d, Some(b"x"));
    assert_eq!(s(&d), Some(&b"x"[..]));
    dstr_copy(&mut d, Some(b""));
    assert_eq!((s(&d), d.len, d.capacity), (None, 0, 0));
}

#[test]
fn test_dstr_ncopy() {
    let mut d = VecDstr::default();

    dstr_ncopy(&mut d, b"hello world", 5);
    assert_eq!((s(&d), d.len, d.capacity), (Some(&b"hello"[..]), 5, 6));

    // previous contents are replaced
    dstr_ncopy(&mut d, b"abcdef", 2);
    assert_eq!((s(&d), d.len, d.capacity), (Some(&b"ab"[..]), 2, 3));

    // zero length frees
    dstr_ncopy(&mut d, b"abc", 0);
    assert_eq!((s(&d), d.len, d.capacity), (None, 0, 0));

    let src = b"abcdef";
    dstr_ncopy_dstr(&mut d, src, 3);
    assert_eq!((s(&d), d.len, d.capacity), (Some(&b"abc"[..]), 3, 4));

    // length is clamped to the source length
    dstr_ncopy_dstr(&mut d, src, 100);
    assert_eq!((s(&d), d.len, d.capacity), (Some(&b"abcdef"[..]), 6, 7));

    dstr_ncopy_dstr(&mut d, src, 0);
    assert_eq!((s(&d), d.len), (None, 0));
}

#[test]
fn test_dstr_cat() {
    let mut d = VecDstr::default();

    // NULL / empty are no-ops and do not allocate (dstr_cat is dstr_ncat
    // with strlen)
    dstr_ncat(&mut d, None, 0);
    dstr_ncat(&mut d, Some(b""), 0);
    assert_eq!((s(&d), d.len), (None, 0));

    dstr_ncat(&mut d, Some(b"foo"), 3);
    assert_eq!((s(&d), d.len), (Some(&b"foo"[..]), 3));
    assert!(d.capacity > d.len);

    dstr_ncat(&mut d, Some(b"bar"), 3);
    assert_eq!((s(&d), d.len), (Some(&b"foobar"[..]), 6));
    assert!(d.capacity > d.len);

    // dstr_cat_ch is dstr_insert_ch at len
    dstr_insert_ch(&mut d, 6, b'!');
    assert_eq!((s(&d), d.len), (Some(&b"foobar!"[..]), 7));
    assert!(d.capacity > d.len);

    dstr_ncat(&mut d, Some(b"123456"), 3);
    assert_eq!((s(&d), d.len), (Some(&b"foobar!123"[..]), 10));
    assert!(d.capacity > d.len);

    // zero length / NULL / empty are no-ops
    dstr_ncat(&mut d, Some(b"zzz"), 0);
    dstr_ncat(&mut d, None, 3);
    dstr_ncat(&mut d, Some(b""), 3);
    assert_eq!((s(&d), d.len), (Some(&b"foobar!123"[..]), 10));

    dstr_cat_dstr(&mut d, b"-tail");
    assert_eq!((s(&d), d.len), (Some(&b"foobar!123-tail"[..]), 15));
    assert!(d.capacity > d.len);

    // empty source is a no-op
    dstr_cat_dstr(&mut d, b"");
    assert_eq!((s(&d), d.len), (Some(&b"foobar!123-tail"[..]), 15));

    dstr_ncat_dstr(&mut d, Some(b"abcdef"), 2);
    assert_eq!((s(&d), d.len), (Some(&b"foobar!123-tailab"[..]), 17));

    // length is clamped to the source length
    dstr_ncat_dstr(&mut d, Some(b"abcdef"), 100);
    assert_eq!((s(&d), d.len), (Some(&b"foobar!123-tailababcdef"[..]), 23));
    assert!(d.capacity > d.len);
}

#[test]
fn test_dstr_insert() {
    let mut d = VecDstr::default();

    // inserting at len (0 here) is a cat
    dstr_insert(&mut d, 0, Some(b"ad"));
    assert_eq!((s(&d), d.len), (Some(&b"ad"[..]), 2));

    dstr_insert(&mut d, 1, Some(b"bc"));
    assert_eq!((s(&d), d.len), (Some(&b"abcd"[..]), 4));
    assert!(d.capacity > d.len);

    dstr_insert(&mut d, 4, Some(b"ef"));
    assert_eq!((s(&d), d.len), (Some(&b"abcdef"[..]), 6));

    dstr_insert(&mut d, 0, Some(b"X"));
    assert_eq!((s(&d), d.len), (Some(&b"Xabcdef"[..]), 7));
    assert!(d.capacity > d.len);

    // NULL / empty are no-ops
    dstr_insert(&mut d, 2, None);
    dstr_insert(&mut d, 2, Some(b""));
    assert_eq!((s(&d), d.len), (Some(&b"Xabcdef"[..]), 7));

    dstr_insert_dstr(&mut d, 3, b"--");
    assert_eq!((s(&d), d.len), (Some(&b"Xab--cdef"[..]), 9));
    assert!(d.capacity > d.len);

    dstr_insert_dstr(&mut d, 9, b"--");
    assert_eq!((s(&d), d.len), (Some(&b"Xab--cdef--"[..]), 11));

    dstr_insert_dstr(&mut d, 1, b"");
    assert_eq!((s(&d), d.len), (Some(&b"Xab--cdef--"[..]), 11));
}

#[test]
fn test_dstr_insert_ch() {
    let mut d = VecDstr::default();

    // idx == len is a cat_ch, including on an empty dstr
    dstr_insert_ch(&mut d, 0, b'a');
    assert_eq!((s(&d), d.len), (Some(&b"a"[..]), 1));

    dstr_insert_ch(&mut d, 1, b'c');
    assert_eq!((s(&d), d.len), (Some(&b"ac"[..]), 2));

    // (the C test reserves room here to avoid C's out-of-bounds write; the
    // port never writes past the string's NUL, so no reserve is needed)

    dstr_insert_ch(&mut d, 1, b'b');
    assert_eq!((s(&d), d.len), (Some(&b"abc"[..]), 3));
    assert!(d.capacity > d.len);

    dstr_insert_ch(&mut d, 0, b'X');
    assert_eq!((s(&d), d.len), (Some(&b"Xabc"[..]), 4));
    assert!(d.capacity > d.len);

    dstr_insert_ch(&mut d, 4, b'Z');
    assert_eq!((s(&d), d.len), (Some(&b"XabcZ"[..]), 5));
}

#[test]
fn test_dstr_remove() {
    let mut d = VecDstr::from(b"abcdef");

    dstr_remove(&mut d, 1, 0);
    assert_eq!((s(&d), d.len), (Some(&b"abcdef"[..]), 6));

    // middle
    dstr_remove(&mut d, 1, 2);
    assert_eq!((s(&d), d.len), (Some(&b"adef"[..]), 4));
    assert!(d.capacity > d.len);

    // tail
    dstr_remove(&mut d, 2, 2);
    assert_eq!((s(&d), d.len), (Some(&b"ad"[..]), 2));
    assert!(d.capacity > d.len);

    // head
    dstr_remove(&mut d, 0, 1);
    assert_eq!((s(&d), d.len), (Some(&b"d"[..]), 1));

    // removing everything frees
    dstr_remove(&mut d, 0, 1);
    assert_eq!((s(&d), d.len, d.capacity), (None, 0, 0));
}

#[test]
fn test_dstr_replace() {
    let mut d = VecDstr::default();

    // empty dstr is a no-op
    dstr_replace(&mut d, b"a", Some(b"b"));
    assert_eq!((s(&d), d.len), (None, 0));

    // replacement shorter than find
    d = VecDstr::from(b"foo bar foo");
    dstr_replace(&mut d, b"foo", Some(b"x"));
    assert_eq!((s(&d), d.len), (Some(&b"x bar x"[..]), 7));
    assert!(d.capacity > d.len);

    // empty replacement
    d = VecDstr::from(b"foo bar foo");
    dstr_replace(&mut d, b"foo", Some(b""));
    assert_eq!((s(&d), d.len), (Some(&b" bar "[..]), 5));

    // NULL replacement behaves like empty
    d = VecDstr::from(b"foo bar foo");
    dstr_replace(&mut d, b"foo", None);
    assert_eq!((s(&d), d.len), (Some(&b" bar "[..]), 5));

    // replacement longer than find
    d = VecDstr::from(b"a-b-c");
    dstr_replace(&mut d, b"-", Some(b"--"));
    assert_eq!((s(&d), d.len), (Some(&b"a--b--c"[..]), 7));
    assert!(d.capacity > d.len);

    d = VecDstr::from(b"-start and end-");
    dstr_replace(&mut d, b"-", Some(b"<longer>"));
    assert_eq!(
        (s(&d), d.len),
        (Some(&b"<longer>start and end<longer>"[..]), 29)
    );
    assert!(d.capacity > d.len);

    // replacement same length as find
    d = VecDstr::from(b"abcabc");
    dstr_replace(&mut d, b"abc", Some(b"xyz"));
    assert_eq!((s(&d), d.len), (Some(&b"xyzxyz"[..]), 6));

    // no match, every branch
    d = VecDstr::from(b"hello");
    dstr_replace(&mut d, b"zzz", Some(b"y"));
    assert_eq!((s(&d), d.len), (Some(&b"hello"[..]), 5));
    dstr_replace(&mut d, b"zzz", Some(b"yyyyyy"));
    assert_eq!((s(&d), d.len), (Some(&b"hello"[..]), 5));
    dstr_replace(&mut d, b"zzz", Some(b"yyy"));
    assert_eq!((s(&d), d.len), (Some(&b"hello"[..]), 5));

    // an empty find string loops forever in C; the port does nothing
    dstr_replace(&mut d, b"", Some(b"y"));
    assert_eq!((s(&d), d.len), (Some(&b"hello"[..]), 5));
}

#[test]
fn test_dstr_depad() {
    let mut d = VecDstr::default();

    // no array is a no-op
    dstr_depad(&mut d);
    assert_eq!((s(&d), d.len), (None, 0));

    d = VecDstr::from(b"  \thi there \r\n");
    dstr_depad(&mut d);
    assert_eq!((s(&d), d.len), (Some(&b"hi there"[..]), 8));
    assert!(d.capacity > d.len);

    d = VecDstr::from(b"tight");
    dstr_depad(&mut d);
    assert_eq!((s(&d), d.len), (Some(&b"tight"[..]), 5));

    // all whitespace frees
    d = VecDstr::from(b" \t\r\n ");
    dstr_depad(&mut d);
    assert_eq!((s(&d), d.len, d.capacity), (None, 0, 0));
}

#[test]
fn test_dstr_left_mid_right() {
    let src = VecDstr::from(b"hello world");
    let mut d = VecDstr::default();

    dstr_left(&mut d, src.c_str().unwrap(), false, 5);
    assert_eq!((s(&d), d.len), (Some(&b"hello"[..]), 5));
    assert!(d.capacity > d.len);

    // in-place (dst == src)
    let mut same = src.clone();
    dstr_left(&mut same, b"", true, 8);
    assert_eq!((s(&same), same.len), (Some(&b"hello wo"[..]), 8));
    assert!(same.capacity > same.len);

    let src = b"hello world";
    dstr_mid(&mut d, src, 6, 5);
    assert_eq!((s(&d), d.len), (Some(&b"world"[..]), 5));
    assert!(d.capacity > d.len);

    dstr_mid(&mut d, src, 2, 3);
    assert_eq!((s(&d), d.len), (Some(&b"llo"[..]), 3));

    // zero count frees dst
    dstr_mid(&mut d, src, 2, 0);
    assert_eq!((s(&d), d.len), (None, 0));

    dstr_right(&mut d, src, 6);
    assert_eq!((s(&d), d.len), (Some(&b"world"[..]), 5));
    assert!(d.capacity > d.len);

    dstr_right(&mut d, src, 0);
    assert_eq!((s(&d), d.len), (Some(&b"hello world"[..]), 11));

    // pos == len leaves dst empty
    dstr_right(&mut d, src, 11);
    assert_eq!((s(&d), d.len), (None, 0));
}

#[test]
fn test_dstr_safe_printf() {
    let mut d = VecDstr::default();

    dstr_safe_printf(
        &mut d,
        Some(b"$1 and $2 and $3 and $4"),
        [Some(b"a"), Some(b"bb"), Some(b"ccc"), Some(b"dddd")],
    );
    assert_eq!(
        (s(&d), d.len),
        (Some(&b"a and bb and ccc and dddd"[..]), 25)
    );
    assert!(d.capacity > d.len);

    // NULL values leave their placeholder untouched
    dstr_safe_printf(&mut d, Some(b"x=$1 y=$2"), [Some(b"1"), None, None, None]);
    assert_eq!((s(&d), d.len), (Some(&b"x=1 y=$2"[..]), 8));

    // repeated placeholder
    dstr_safe_printf(&mut d, Some(b"$1$1"), [Some(b"ab"), None, None, None]);
    assert_eq!((s(&d), d.len), (Some(&b"abab"[..]), 4));

    // empty value removes the placeholder
    dstr_safe_printf(&mut d, Some(b"[$1]"), [Some(b""), None, None, None]);
    assert_eq!((s(&d), d.len), (Some(&b"[]"[..]), 2));

    // NULL format frees
    dstr_safe_printf(
        &mut d,
        None,
        [Some(b"a"), Some(b"b"), Some(b"c"), Some(b"d")],
    );
    assert_eq!((s(&d), d.len), (None, 0));
}

#[test]
fn test_dstr_strref() {
    let mut d = VecDstr::default();

    dstr_copy_strref(&mut d, &b"hello world"[..5]);
    assert_eq!((s(&d), d.len, d.capacity), (Some(&b"hello"[..]), 5, 6));

    // replaces previous contents
    dstr_copy_strref(&mut d, &b"abcdef"[..3]);
    assert_eq!((s(&d), d.len), (Some(&b"abc"[..]), 3));

    dstr_cat_strref(&mut d, Some(b"abcdef"), 3);
    assert_eq!((s(&d), d.len), (Some(&b"abcabc"[..]), 6));
    assert!(d.capacity > d.len);

    // empty strref
    dstr_cat_strref(&mut d, Some(b"xyz"), 0);
    assert_eq!((s(&d), d.len), (Some(&b"abcabc"[..]), 6));

    dstr_copy_strref(&mut d, b"");
    assert_eq!((s(&d), d.len), (None, 0));

    dstr_init_copy_strref(&mut d, &b"init-copy"[..4]);
    assert_eq!((s(&d), d.len, d.capacity), (Some(&b"init"[..]), 4, 5));
}

// Edge cases beyond the C test.

#[test]
fn ensure_capacity_doubles() {
    let mut d = VecDstr::default();
    dstr_ensure_capacity(&mut d, 5);
    assert_eq!(d.capacity, 5);
    dstr_ensure_capacity(&mut d, 6);
    assert_eq!(d.capacity, 10);
    dstr_ensure_capacity(&mut d, 25);
    assert_eq!(d.capacity, 25);
    dstr_ensure_capacity(&mut d, 3);
    assert_eq!(d.capacity, 25);
}

#[test]
fn remove_whole_length_ignores_idx() {
    // characterized, not endorsed
    let mut d = VecDstr::from(b"abc");
    dstr_remove(&mut d, 2, 3);
    assert_eq!((s(&d), d.len), (None, 0));
}

#[test]
fn out_of_range_edits_do_nothing() {
    let mut d = VecDstr::from(b"abc");
    let before = d.clone();
    dstr_insert(&mut d, 4, Some(b"x"));
    dstr_insert_dstr(&mut d, 9, b"x");
    dstr_insert_ch(&mut d, 4, b'x');
    dstr_remove(&mut d, 2, 2);
    assert_eq!(d, before);
}

#[test]
fn replace_restarts_after_each_replacement() {
    let mut d = VecDstr::from(b"aaaa");
    dstr_replace(&mut d, b"aa", Some(b"a"));
    assert_eq!((s(&d), d.len), (Some(&b"aa"[..]), 2));

    let mut d = VecDstr::from(b"aaa");
    dstr_replace(&mut d, b"a", Some(b"aa"));
    assert_eq!((s(&d), d.len), (Some(&b"aaaaaa"[..]), 6));
}

#[test]
fn ncat_copies_embedded_nul() {
    let mut d = VecDstr::from(b"a");
    dstr_ncat(&mut d, Some(b"b\0c"), 3);
    assert_eq!(d.len, 4);
    assert_eq!(&d.buf.as_ref().unwrap()[..5], b"ab\0c\0");
}

#[test]
fn compares_use_platform_char() {
    let signed = (0x80u8 as core::ffi::c_char) < 0;
    let expected = if signed { -1 } else { 1 };
    assert_eq!(astrcmpi(b"\xe9", b"a"), expected);
    assert_eq!(astrcmp_n(b"\x80", b"a", 1), expected);
}

#[test]
fn strlist_split_on_nul_is_whole_string() {
    // intentional difference: C reads past the terminator
    assert_eq!(strlist_split(b"a,b", 0, false), [&b"a,b"[..]]);
    assert!(strlist_split(b"", 0, false).is_empty());
}
