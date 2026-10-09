//! Safe core for `libobs/util/lexer.c`: `strref` comparisons, number
//! validation, the base tokenizer, text offsets and error-string building.
//!
//! Strings are byte slices without their NUL terminator. A NULL C string or
//! `strref` maps to an empty slice, which C treats the same way.
//!
//! Characters compare as C `char`, whose signedness is the platform's
//! (`c_char`): bytes from 0x80 sort below ASCII on x86 and above it on Arm
//! Linux, as in C. `toupper` is the C locale's, ASCII only.

/// `strref_cmp`: compares the segment `s1` with the C string `s2`.
pub fn strref_cmp(s1: &[u8], s2: &[u8]) -> i32 {
    let _ = (s1, s2);
    todo!()
}

/// `strref_cmpi`: [`strref_cmp`] ignoring ASCII case.
pub fn strref_cmpi(s1: &[u8], s2: &[u8]) -> i32 {
    let _ = (s1, s2);
    todo!()
}

/// `strref_cmp_strref`: compares two segments.
pub fn strref_cmp_strref(s1: &[u8], s2: &[u8]) -> i32 {
    let _ = (s1, s2);
    todo!()
}

/// `strref_cmpi_strref`: [`strref_cmp_strref`] ignoring ASCII case.
pub fn strref_cmpi_strref(s1: &[u8], s2: &[u8]) -> i32 {
    let _ = (s1, s2);
    todo!()
}

/// `valid_int_str`: an optional sign followed by digits.
pub fn valid_int_str(s: &[u8], n: usize) -> bool {
    let _ = (s, n);
    todo!()
}

/// `valid_float_str`.
pub fn valid_float_str(s: &[u8], n: usize) -> bool {
    let _ = (s, n);
    todo!()
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

/// `lexer_getbasetoken`.
pub fn get_base_token(text: &[u8], ignore_whitespace: bool) -> (usize, Option<BaseToken>) {
    let _ = (text, ignore_whitespace);
    todo!()
}

/// `lexer_getstroffset`.
pub fn get_str_offset(text: &[u8], pos: usize) -> (u32, u32) {
    let _ = (text, pos);
    todo!()
}

/// One `error_data_buildstring` line.
pub fn format_error_item(
    out: &mut Vec<u8>,
    file: Option<&[u8]>,
    row: u32,
    column: u32,
    error: Option<&[u8]>,
) {
    let _ = (out, file, row, column, error);
    todo!()
}
