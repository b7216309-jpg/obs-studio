//! Safe core for `libobs/util/cf-tokenizer.c`: the C-family lexer
//! (`cf_lexer_lex`) and `cf_literal_to_str`.
//!
//! The lexer turns source text into a reformatted copy, with spliced lines
//! (backslash-newline) merged and comments replaced by one space, and into
//! tokens that point both into that copy (`str`) and into the original text
//! (`unmerged_str`). Tokens here are offsets; the shim turns them into the
//! C pointers the preprocessor and parsers walk.
//!
//! Text is read as if NUL bytes followed it. That matches C wherever C
//! stays in bounds. C reads past the terminator when a comment or string
//! ends in a line splice at the very end of the text (and `\x`/octal escapes
//! in `cf_literal_to_str` can skip past it); there the Rust port reads NUL
//! where C reads whatever follows.

/// `enum cf_token_type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CfTokenType {
    None = 0,
    Name = 1,
    Num = 2,
    SpaceTab = 3,
    Newline = 4,
    String = 5,
    Other = 6,
}

/// A `struct cf_token` as offsets: `str` into [`CfLexed::reformatted`],
/// `unmerged` into the input text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CfToken {
    pub str_start: usize,
    pub str_len: usize,
    pub unmerged_start: usize,
    pub unmerged_len: usize,
    pub kind: CfTokenType,
}

/// The result of [`cf_lex`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CfLexed {
    /// The reformatted text, without its NUL terminator.
    pub reformatted: Vec<u8>,
    /// The tokens, ending with a `CfTokenType::None` terminator whose `str`
    /// is the end of `reformatted` and whose `unmerged` is `offset`.
    pub tokens: Vec<CfToken>,
    /// The base lexer's final offset into the input.
    pub offset: usize,
    /// An unterminated block comment was found.
    pub unexpected_eof: bool,
}

/// `cf_lexer_lex` on non-empty `text` (the input up to its NUL).
pub fn cf_lex(text: &[u8]) -> CfLexed {
    let _ = text;
    todo!()
}

/// `cf_literal_to_str`.
pub fn cf_literal_to_str(literal: &[u8], count: usize, ulong_bits: u32) -> Option<Vec<u8>> {
    let _ = (literal, count, ulong_bits);
    todo!()
}
