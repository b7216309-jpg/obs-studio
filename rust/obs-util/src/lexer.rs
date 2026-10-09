//! Safe core for `libobs/util/lexer.c`: `strref` comparisons, number
//! validation, the base tokenizer, text offsets and error-string building.
//!
//! Strings are byte slices without their NUL terminator. A NULL C string or
//! `strref` maps to an empty slice, which C treats the same way.
//!
//! Characters compare as C `char`, whose signedness is the platform's
//! (`c_char`): bytes from 0x80 sort below ASCII on x86 and above it on Arm
//! Linux, as in C. `toupper` is the C locale's, ASCII only.

use core::ffi::c_char;

fn ch(b: u8) -> c_char {
    b as c_char
}

fn upper(b: u8) -> c_char {
    ch(b.to_ascii_uppercase())
}

/// `strref_is_empty`: no bytes, or a NUL first byte.
fn strref_is_empty(s: &[u8]) -> bool {
    s.first().is_none_or(|&b| b == 0)
}

fn cmp_strref_cstr(s1: &[u8], s2: &[u8], map: fn(u8) -> c_char) -> i32 {
    if strref_is_empty(s1) {
        return if s2.is_empty() { 0 } else { -1 };
    }

    // C: do { ... } while (i++ < str1->len && *str2++);
    let mut i = 0;
    loop {
        let c1 = s1.get(i).map_or(0, |&b| map(b));
        let c2 = map(s2.get(i).copied().unwrap_or(0));
        if c1 < c2 {
            return -1;
        } else if c1 > c2 {
            return 1;
        }
        if i >= s1.len() || c2 == 0 {
            return 0;
        }
        i += 1;
    }
}

fn cmp_strref_strref(s1: &[u8], s2: &[u8], map: fn(u8) -> c_char) -> i32 {
    if strref_is_empty(s1) {
        return if strref_is_empty(s2) { 0 } else { -1 };
    }
    // Characterized, not endorsed: -1 whichever side is empty.
    if strref_is_empty(s2) {
        return -1;
    }

    let mut i = 0;
    while i <= s1.len() && i <= s2.len() {
        let c1 = s1.get(i).map_or(0, |&b| map(b));
        let c2 = s2.get(i).map_or(0, |&b| map(b));
        if c1 < c2 {
            return -1;
        } else if c1 > c2 {
            return 1;
        }
        i += 1;
    }
    0
}

/// `strref_cmp`: compares the segment `s1` with the C string `s2`.
pub fn strref_cmp(s1: &[u8], s2: &[u8]) -> i32 {
    cmp_strref_cstr(s1, s2, ch)
}

/// `strref_cmpi`: [`strref_cmp`] ignoring ASCII case.
pub fn strref_cmpi(s1: &[u8], s2: &[u8]) -> i32 {
    cmp_strref_cstr(s1, s2, upper)
}

/// `strref_cmp_strref`: compares two segments.
pub fn strref_cmp_strref(s1: &[u8], s2: &[u8]) -> i32 {
    cmp_strref_strref(s1, s2, ch)
}

/// `strref_cmpi_strref`: [`strref_cmp_strref`] ignoring ASCII case.
pub fn strref_cmpi_strref(s1: &[u8], s2: &[u8]) -> i32 {
    cmp_strref_strref(s1, s2, upper)
}

/// Shared walk of `valid_int_str` and `valid_float_str`: `s` is the string up
/// to its NUL, and `n` the C `n` (0 for the whole string). A leading sign is
/// skipped without counting toward `n`, as in C. `accept` sees each byte and
/// returns `false` to reject the string.
fn walk_number(s: &[u8], n: usize, mut accept: impl FnMut(u8) -> bool) -> bool {
    if s.is_empty() {
        return false;
    }
    let at = |i: usize| s.get(i).copied().unwrap_or(0);
    let mut n = if n == 0 { s.len() } else { n };
    let mut i = usize::from(matches!(s[0], b'-' | b'+'));

    // C: do { ... } while (*++str && --n);
    loop {
        if !accept(at(i)) {
            return false;
        }
        i += 1;
        if at(i) == 0 {
            return true;
        }
        n -= 1;
        if n == 0 {
            return true;
        }
    }
}

/// `valid_int_str`: an optional sign followed by digits.
///
/// `s` is the string up to its NUL terminator and `n` limits how many
/// characters after the sign are examined (0: all).
pub fn valid_int_str(s: &[u8], n: usize) -> bool {
    let mut found_num = false;
    walk_number(s, n, |b| {
        found_num = b.is_ascii_digit();
        found_num
    }) && found_num
}

/// `valid_float_str`: an optional sign, digits, an optional `.`, and an
/// optional `e` exponent. Characterized, not endorsed: a sign right after
/// `e` is rejected, and a sign after exponent digits is accepted.
///
/// `s` and `n` as in [`valid_int_str`].
pub fn valid_float_str(s: &[u8], n: usize) -> bool {
    let (mut found_num, mut found_exp, mut found_dec) = (false, false, false);
    walk_number(s, n, |b| match b {
        b'.' => {
            let ok = !found_dec && !found_exp && found_num;
            found_dec = true;
            ok
        }
        b'e' => {
            let ok = !found_exp && found_num;
            found_exp = true;
            found_num = false;
            ok
        }
        b'-' | b'+' => found_exp && found_num,
        b'0'..=b'9' => {
            found_num = true;
            true
        }
        _ => false,
    }) && found_num
}

/// `enum base_token_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BaseTokenType {
    None = 0,
    Alpha = 1,
    Digit = 2,
    Whitespace = 3,
    Other = 4,
}

/// A token found by [`get_base_token`], as a range of its input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BaseToken {
    pub start: usize,
    pub len: usize,
    pub kind: BaseTokenType,
}

fn is_whitespace(b: u8) -> bool {
    matches!(b, b' ' | b'\r' | b'\t' | b'\n')
}

fn is_newline(b: u8) -> bool {
    matches!(b, b'\r' | b'\n')
}

fn is_newline_pair(a: u8, b: u8) -> bool {
    (a == b'\r' && b == b'\n') || (a == b'\n' && b == b'\r')
}

fn char_token_type(b: u8) -> BaseTokenType {
    if is_whitespace(b) {
        BaseTokenType::Whitespace
    } else if b.is_ascii_digit() {
        BaseTokenType::Digit
    } else if b.is_ascii_alphabetic() {
        BaseTokenType::Alpha
    } else {
        BaseTokenType::Other
    }
}

/// `lexer_getbasetoken`: reads the next token from `text` (the lexer's text
/// from its current offset up to the NUL).
///
/// Returns how far the offset advances, which is past any skipped
/// whitespace even when no token is found, and the token. A token is a run
/// of letters, a run of digits, or one other character; a CRLF or LFCR pair
/// is one whitespace token.
pub fn get_base_token(text: &[u8], ignore_whitespace: bool) -> (usize, Option<BaseToken>) {
    let mut offset = 0;
    let mut token: Option<(usize, BaseTokenType)> = None;

    while let Some(&b) = text.get(offset) {
        offset += 1;
        let new_type = char_token_type(b);

        match token {
            None => {
                if new_type == BaseTokenType::Whitespace && ignore_whitespace {
                    continue;
                }
                token = Some((offset - 1, new_type));
                if new_type != BaseTokenType::Digit && new_type != BaseTokenType::Alpha {
                    let next = text.get(offset).copied().unwrap_or(0);
                    if is_newline(b) && is_newline_pair(b, next) {
                        offset += 1;
                    }
                    break;
                }
            }
            Some((_, kind)) if kind != new_type => {
                offset -= 1;
                break;
            }
            Some(_) => {}
        }
    }

    let token = token.map(|(start, kind)| BaseToken {
        start,
        len: offset - start,
        kind,
    });
    (offset, token)
}

/// `lexer_getstroffset`: the 1-based row and column of `text[pos]`.
///
/// CRLF and LFCR count as one newline. `text` must extend one byte past
/// `pos` when `text[pos - 1]` is a newline (C peeks at it); a missing byte
/// reads as NUL.
pub fn get_str_offset(text: &[u8], pos: usize) -> (u32, u32) {
    let (mut row, mut col) = (1u32, 1u32);
    let mut i = 0;

    while i < pos {
        let b = text.get(i).copied().unwrap_or(0);
        if is_newline(b) {
            // newline_size() - 1: skip the second char of a pair.
            if is_newline_pair(b, text.get(i + 1).copied().unwrap_or(0)) {
                i += 1;
            }
            col = 1;
            row = row.wrapping_add(1);
        } else {
            col = col.wrapping_add(1);
        }
        i += 1;
    }

    (row, col)
}

/// Appends one `error_data_buildstring` line, `"%s (%u, %u): %s\n"`. `None`
/// prints as `(null)`, like glibc, macOS and the MSVC CRT.
pub fn format_error_item(
    out: &mut Vec<u8>,
    file: Option<&[u8]>,
    row: u32,
    column: u32,
    error: Option<&[u8]>,
) {
    out.extend_from_slice(file.unwrap_or(b"(null)"));
    out.extend_from_slice(format!(" ({row}, {column}): ").as_bytes());
    out.extend_from_slice(error.unwrap_or(b"(null)"));
    out.push(b'\n');
}
