//! Tier 1: safe-core tests. The `*_test` tests mirror
//! `test/cmocka/test_lexer.c` case for case.
//!
//! A NULL `strref` or C string is an empty slice in the safe core, and the
//! `struct lexer` inline helpers (`lexer_start`, `lexer_reset`) become a
//! byte string and an offset. The allocation checks (`bnum_allocs`) and the
//! `error_data` container itself are C-side; `lexer_parity.rs` covers them
//! at the shim.

use obs_c_oracle as _; // links the test allocator
use obs_util::lexer::{
    BaseToken, BaseTokenType, format_error_item, get_base_token, get_str_offset, strref_cmp,
    strref_cmp_strref, strref_cmpi, strref_cmpi_strref, valid_float_str, valid_int_str,
};

#[test]
fn strref_cmp_test() {
    let (abc, ab, empty): (&[u8], &[u8], &[u8]) = (b"abc", b"ab", b"");
    // a strref is a segment: only the first two chars of the array count
    let prefix = &b"abcdef"[..2];

    assert_eq!(strref_cmp(abc, b"abc"), 0);
    assert_eq!(strref_cmp(abc, b"abd"), -1);
    assert_eq!(strref_cmp(abc, b"abb"), 1);

    // strref shorter than the C string, and longer than it
    assert_eq!(strref_cmp(ab, b"abc"), -1);
    assert_eq!(strref_cmp(abc, b"ab"), 1);
    assert_eq!(strref_cmp(prefix, b"ab"), 0);
    assert_eq!(strref_cmp(prefix, b"abc"), -1);

    // case sensitive: 'A' (65) < 'a' (97)
    assert_eq!(strref_cmp(b"ABC", b"abc"), -1);
    assert_eq!(strref_cmp(abc, b"ABC"), 1);

    // empty strref (against "" and NULL alike)
    assert_eq!(strref_cmp(empty, b""), 0);
    assert_eq!(strref_cmp(empty, b"a"), -1);

    // non-empty strref against NULL behaves like against ""
    assert_eq!(strref_cmp(abc, b""), 1);
}

#[test]
fn strref_cmpi_test() {
    let (abc, upper, empty): (&[u8], &[u8], &[u8]) = (b"abc", b"ABC", b"");

    assert_eq!(strref_cmpi(abc, b"ABC"), 0);
    assert_eq!(strref_cmpi(upper, b"abc"), 0);
    assert_eq!(strref_cmpi(upper, b"abd"), -1);
    assert_eq!(strref_cmpi(upper, b"abb"), 1);
    assert_eq!(strref_cmpi(abc, b"ab"), 1);
    assert_eq!(strref_cmpi(abc, b"abcd"), -1);
    assert_eq!(strref_cmpi(abc, b""), 1);

    assert_eq!(strref_cmpi(empty, b""), 0);
    assert_eq!(strref_cmpi(empty, b"a"), -1);
}

#[test]
fn strref_cmp_strref_test() {
    let abc: &[u8] = b"abc";
    let abc2 = &b"abcxyz"[..3];
    let (abd, ab, upper, empty): (&[u8], &[u8], &[u8], &[u8]) = (b"abd", b"ab", b"ABC", b"");

    assert_eq!(strref_cmp_strref(abc, abc), 0);
    assert_eq!(strref_cmp_strref(abc, abc2), 0);
    assert_eq!(strref_cmp_strref(abc, abd), -1);
    assert_eq!(strref_cmp_strref(abd, abc), 1);
    assert_eq!(strref_cmp_strref(ab, abc), -1);
    assert_eq!(strref_cmp_strref(abc, ab), 1);
    assert_eq!(strref_cmp_strref(upper, abc), -1);
    assert_eq!(strref_cmp_strref(abc, upper), 1);

    // empty handling, including the asymmetric -1 when only the second is
    // empty (characterized, not endorsed)
    assert_eq!(strref_cmp_strref(empty, empty), 0);
    assert_eq!(strref_cmp_strref(empty, abc), -1);
    assert_eq!(strref_cmp_strref(abc, empty), -1);
}

#[test]
fn strref_cmpi_strref_test() {
    let abc: &[u8] = b"abc";
    let (upper, abd, ab, empty): (&[u8], &[u8], &[u8], &[u8]) = (b"ABC", b"ABD", b"AB", b"");

    assert_eq!(strref_cmpi_strref(abc, upper), 0);
    assert_eq!(strref_cmpi_strref(upper, abc), 0);
    assert_eq!(strref_cmpi_strref(abc, abd), -1);
    assert_eq!(strref_cmpi_strref(abd, abc), 1);
    assert_eq!(strref_cmpi_strref(ab, abc), -1);
    assert_eq!(strref_cmpi_strref(abc, ab), 1);

    assert_eq!(strref_cmpi_strref(empty, empty), 0);
    assert_eq!(strref_cmpi_strref(empty, abc), -1);
    assert_eq!(strref_cmpi_strref(abc, empty), -1);
}

#[test]
fn valid_int_str_test() {
    assert!(valid_int_str(b"123", 0));
    assert!(valid_int_str(b"0", 0));
    assert!(valid_int_str(b"-5", 0));
    assert!(valid_int_str(b"+7", 0));
    assert!(!valid_int_str(b"1.5", 0));
    assert!(!valid_int_str(b"1e3", 0));
    assert!(!valid_int_str(b"abc", 0));
    assert!(!valid_int_str(b"12a", 0));
    // "" and NULL
    assert!(!valid_int_str(b"", 0));

    // sign only
    assert!(!valid_int_str(b"-", 0));

    // explicit n limits how many chars are examined
    assert!(valid_int_str(b"12a", 2));
    assert!(valid_int_str(b"12a", 1));
    assert!(!valid_int_str(b"12a", 3));

    // the sign is skipped without counting toward n
    assert!(!valid_int_str(b"-12x", 3));
    assert!(valid_int_str(b"-12x", 2));
}

#[test]
fn valid_float_str_test() {
    assert!(valid_float_str(b"123", 0));
    assert!(valid_float_str(b"1.5", 0));
    assert!(valid_float_str(b"1e3", 0));
    assert!(valid_float_str(b"-5", 0));
    assert!(valid_float_str(b"+1.5e3", 0));
    assert!(valid_float_str(b"1.", 0));
    assert!(!valid_float_str(b"abc", 0));
    // "" and NULL
    assert!(!valid_float_str(b"", 0));
    assert!(!valid_float_str(b".5", 0));
    assert!(!valid_float_str(b"1.2.3", 0));
    assert!(!valid_float_str(b"e3", 0));
    assert!(!valid_float_str(b"1e3e4", 0));
    assert!(!valid_float_str(b"1e", 0));

    // quirk (characterized, not endorsed): a sign right after the exponent
    // marker is rejected
    assert!(!valid_float_str(b"1e-3", 0));
    assert!(!valid_float_str(b"1e+3", 0));
    // quirk (characterized, not endorsed): a trailing sign after exponent
    // digits is accepted
    assert!(valid_float_str(b"1e3-", 0));

    // explicit n
    assert!(valid_float_str(b"1.5", 1));
    assert!(valid_float_str(b"1.x", 2));
    assert!(!valid_float_str(b"1.x", 3));
}

/// A `struct lexer`: the text and the current offset into it.
struct Lexer<'a> {
    text: &'a [u8],
    offset: usize,
}

impl<'a> Lexer<'a> {
    fn new(text: &'a [u8]) -> Self {
        Lexer { text, offset: 0 }
    }

    fn next(&mut self, ignore_whitespace: bool) -> Option<(BaseTokenType, &'a [u8])> {
        let (advance, token) = get_base_token(&self.text[self.offset..], ignore_whitespace);
        let base = self.offset;
        self.offset += advance;
        token.map(|BaseToken { start, len, kind }| {
            (kind, &self.text[base + start..base + start + len])
        })
    }

    fn expect(&mut self, ignore_whitespace: bool, kind: BaseTokenType, text: impl AsRef<[u8]>) {
        assert_eq!(self.next(ignore_whitespace), Some((kind, text.as_ref())));
    }

    fn expect_end(&mut self, ignore_whitespace: bool) {
        assert_eq!(self.next(ignore_whitespace), None);
    }
}

const IGNORE: bool = true;
const PARSE: bool = false;

#[test]
fn getbasetoken_ignore_ws_test() {
    use BaseTokenType::*;

    let mut lex = Lexer::new(b"abc 12 +\n x");
    lex.expect(IGNORE, Alpha, "abc");
    lex.expect(IGNORE, Digit, "12");
    lex.expect(IGNORE, Other, "+");
    lex.expect(IGNORE, Alpha, "x");
    lex.expect_end(IGNORE);
    // stays at the end
    lex.expect_end(IGNORE);

    // lexer_reset rewinds to the start
    lex.offset = 0;
    lex.expect(IGNORE, Alpha, "abc");
}

#[test]
fn getbasetoken_parse_ws_test() {
    use BaseTokenType::*;

    let mut lex = Lexer::new(b"abc 12 +\n x");
    lex.expect(PARSE, Alpha, "abc");
    lex.expect(PARSE, Whitespace, " ");
    lex.expect(PARSE, Digit, "12");
    lex.expect(PARSE, Whitespace, " ");
    lex.expect(PARSE, Other, "+");
    lex.expect(PARSE, Whitespace, "\n");
    lex.expect(PARSE, Whitespace, " ");
    lex.expect(PARSE, Alpha, "x");
    lex.expect_end(PARSE);

    // CRLF and LFCR are single two-char whitespace tokens; lone CR is one char
    let mut lex = Lexer::new(b"a\r\nb\n\rc\rd");
    lex.expect(PARSE, Alpha, "a");
    lex.expect(PARSE, Whitespace, "\r\n");
    lex.expect(PARSE, Alpha, "b");
    lex.expect(PARSE, Whitespace, "\n\r");
    lex.expect(PARSE, Alpha, "c");
    lex.expect(PARSE, Whitespace, "\r");
    lex.expect(PARSE, Alpha, "d");
    lex.expect_end(PARSE);

    // whitespace-only text yields nothing when whitespace is ignored
    let mut lex = Lexer::new(b" \t\n ");
    lex.expect_end(IGNORE);

    // a lexer that was never started has no offset: shim only
}

#[test]
fn getstroffset_test() {
    let t = b"ab\ncd\r\nef\n\rg";

    assert_eq!(get_str_offset(t, 0), (1, 1));
    assert_eq!(get_str_offset(t, 1), (1, 2));
    // 'c' directly after "\n"
    assert_eq!(get_str_offset(t, 3), (2, 1));
    // 'd'
    assert_eq!(get_str_offset(t, 4), (2, 2));
    // the '\r' of the CRLF pair is still on row 2, after "cd"
    assert_eq!(get_str_offset(t, 5), (2, 3));
    // 'e' after "\r\n" counts as a single newline
    assert_eq!(get_str_offset(t, 7), (3, 1));
    // 'f'
    assert_eq!(get_str_offset(t, 8), (3, 2));
    // 'g' after "\n\r" counts as a single newline
    assert_eq!(get_str_offset(t, 11), (4, 1));
    // end of text
    assert_eq!(get_str_offset(t, 12), (4, 2));

    // NULL pointer leaves the outputs untouched: shim only
}

#[test]
fn error_data_test() {
    // error_data_init, _add, _item, _has_errors and _type_count are the
    // darray container: shim only. This is the buildstring formatting.
    let mut out = Vec::new();
    format_error_item(&mut out, Some(b"f.txt"), 1, 2, Some(b"bad"));
    format_error_item(&mut out, Some(b"g.txt"), 10, 20, Some(b"worse"));
    assert_eq!(out, b"f.txt (1, 2): bad\ng.txt (10, 20): worse\n");
}

// Edge cases beyond the C test.

#[test]
fn strref_with_nul_first_byte_is_empty() {
    assert_eq!(strref_cmp(b"\0abc", b""), 0);
    assert_eq!(strref_cmp_strref(b"\0x", b""), 0);
    // a NUL later is compared like any byte
    assert_eq!(strref_cmp(b"a\0", b"a"), 0);
    assert_eq!(strref_cmp_strref(b"a\0b", b"a\0c"), -1);
}

#[test]
fn high_bytes_compare_as_platform_char() {
    let signed = (0x80u8 as core::ffi::c_char) < 0;
    let expected = if signed { -1 } else { 1 };
    assert_eq!(strref_cmp(b"\x80", b"a"), expected);
    assert_eq!(strref_cmpi_strref(b"\xe9", b"z"), expected);
}

#[test]
fn getbasetoken_edges() {
    use BaseTokenType::*;

    assert_eq!(get_base_token(b"", PARSE), (0, Option::None));
    // skipped whitespace still advances the offset
    assert_eq!(get_base_token(b"  ", IGNORE), (2, Option::None));
    // runs of OTHER and of WHITESPACE are one char each
    let mut lex = Lexer::new(b"++\t\t");
    lex.expect(PARSE, Other, "+");
    lex.expect(PARSE, Other, "+");
    lex.expect(PARSE, Whitespace, "\t");
    lex.expect(PARSE, Whitespace, "\t");
    lex.expect_end(PARSE);
    // letters and digits do not mix; high bytes are OTHER
    let mut lex = Lexer::new(b"ab12\xc3\xa9");
    lex.expect(PARSE, Alpha, "ab");
    lex.expect(PARSE, Digit, "12");
    lex.expect(PARSE, Other, b"\xc3");
}

#[test]
fn getstroffset_edges() {
    // the position falls inside a CRLF pair: C jumps over the pair
    assert_eq!(get_str_offset(b"a\r\nb", 2), (2, 1));
    // a lone CR at the end peeks at the terminator
    assert_eq!(get_str_offset(b"a\r", 2), (2, 1));
    assert_eq!(get_str_offset(b"", 0), (1, 1));
}

#[test]
fn format_error_item_null_strings() {
    let mut out = Vec::new();
    format_error_item(&mut out, None, 0, u32::MAX, None);
    assert_eq!(out, b"(null) (0, 4294967295): (null)\n");
}

#[test]
fn numbers_edges() {
    // only a sign, with n set
    assert!(!valid_int_str(b"+", 5));
    // n larger than the string
    assert!(valid_int_str(b"42", 100));
    // high bytes are never digits
    assert!(!valid_int_str(b"\xb9", 0));
    assert!(!valid_float_str(b"1\xb9", 0));
    // "1e3+5" is accepted too: a sign after exponent digits
    assert!(valid_float_str(b"1e3+5", 0));
    // a dot after the exponent is rejected
    assert!(!valid_float_str(b"1e3.5", 0));
}
