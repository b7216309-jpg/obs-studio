//! Safe core for `libobs/util/text-lookup.c`: `key=value` locale tables.
//!
//! [`TextLookup`] owns the table. Values are `CString`s whose heap buffers
//! never move while the entry lives, so a value pointer handed to C stays
//! valid until the key is replaced or the lookup is destroyed, as with the
//! C hash table.
//!
//! The file format is whatever `lookup_addfiledata` accepts, quirks
//! included (characterized, not endorsed): tokens come from the base lexer
//! with whitespace significant, so `Key = value` stores `" "`; a `#` starts
//! a comment up to the newline; an empty quoted value (`Key=""`) ends
//! parsing of the whole file; a later duplicate key replaces the earlier
//! one.

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::Path;

use crate::lexer::{BaseTokenType, get_base_token};

const BOM: &[u8] = b"\xEF\xBB\xBF";

/// An opaque `lookup_t`.
#[derive(Debug, Default)]
pub struct TextLookup {
    items: HashMap<Vec<u8>, CString>,
}

impl TextLookup {
    pub fn new() -> Self {
        Self::default()
    }

    /// `text_lookup_add`: reads `path` and adds its entries. Returns
    /// `false` (adding nothing) if the file cannot be read or holds no
    /// text after an optional UTF-8 BOM.
    pub fn add_file(&mut self, path: &Path) -> bool {
        std::fs::read(path).is_ok_and(|data| self.add_data(&data))
    }

    /// `text_lookup_add` on file contents already read: skips a UTF-8 BOM,
    /// stops at the first NUL (C handles the data as a C string), turns
    /// every CR into a space, and parses. Returns `false` if nothing is
    /// left after the BOM (`os_fread_utf8` returns no string then).
    pub fn add_data(&mut self, data: &[u8]) -> bool {
        let data = data.strip_prefix(BOM).unwrap_or(data);
        if data.is_empty() {
            return false;
        }
        let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
        let text: Vec<u8> = data[..end]
            .iter()
            .map(|&b| if b == b'\r' { b' ' } else { b })
            .collect();
        self.add_text(&text);
        true
    }

    /// `text_lookup_getstr`: the value for `key`.
    pub fn get(&self, key: &[u8]) -> Option<&CStr> {
        self.items.get(key).map(CString::as_c_str)
    }

    /// `lookup_addfiledata`.
    fn add_text(&mut self, text: &[u8]) {
        let mut lex = Lex { text, offset: 0 };

        'names: while let Some(name) = lex.get_token() {
            if lex.at(name.0) == b'\n' {
                continue;
            }

            let mut got_eq = false;
            let value = loop {
                let Some(value) = lex.get_token() else {
                    break 'names;
                };
                if lex.at(value.0) == b'\n' {
                    continue 'names;
                } else if !got_eq && lex.at(value.0) == b'=' {
                    got_eq = true;
                    continue;
                }
                break value;
            };

            let key = lex.text[name.0..name.0 + name.1].to_vec();
            let value = convert_string(&lex.text[value.0..value.0 + value.1]);
            self.items.insert(key, value);

            if !lex.goto_next_line() {
                break;
            }
        }
    }
}

/// `convert_string`: `\n`, `\t`, `\r` and `\"` escapes, replaced in that
/// order, each left to right.
fn convert_string(s: &[u8]) -> CString {
    let mut out = s.to_vec();
    for (find, rep) in [
        (&b"\\n"[..], b'\n'),
        (b"\\t", b'\t'),
        (b"\\r", b'\r'),
        (b"\\\"", b'"'),
    ] {
        let mut next = Vec::with_capacity(out.len());
        let mut i = 0;
        while i < out.len() {
            if out[i..].starts_with(find) {
                next.push(rep);
                i += find.len();
            } else {
                next.push(out[i]);
                i += 1;
            }
        }
        out = next;
    }
    CString::new(out).expect("file text holds no NUL")
}

/// The lexer state of `lookup_addfiledata`; tokens are `(start, len)`.
struct Lex<'a> {
    text: &'a [u8],
    offset: usize,
}

impl Lex<'_> {
    fn at(&self, i: usize) -> u8 {
        self.text.get(i).copied().unwrap_or(0)
    }

    /// `lexer_getbasetoken` with whitespace significant.
    fn base_token(&mut self) -> Option<(usize, usize, BaseTokenType)> {
        let (advance, token) = get_base_token(&self.text[self.offset..], false);
        let base = self.offset;
        self.offset += advance;
        token.map(|t| (base + t.start, t.len, t.kind))
    }

    /// `lookup_gettoken`: the next name, value, `=`, whitespace or quoted
    /// string, skipping comments. `None` when nothing (or an empty string)
    /// is left.
    fn get_token(&mut self) -> Option<(usize, usize)> {
        let mut str: Option<(usize, usize)> = None;

        while let Some((start, len, kind)) = self.base_token() {
            let ch = self.at(start);
            match &mut str {
                None => {
                    if ch == b'#' {
                        // comments are designated with a #, and end at LF
                        while !matches!(self.at(self.offset), b'\n' | 0) {
                            self.offset += 1;
                        }
                    } else if kind == BaseTokenType::Whitespace {
                        str = Some((start, len));
                        break;
                    } else {
                        str = Some((start, len));
                        if ch == b'"' {
                            str = Some(self.string_token(start));
                            break;
                        } else if ch == b'=' {
                            break;
                        }
                    }
                }
                Some(s) => {
                    if kind == BaseTokenType::Whitespace || ch == b'=' {
                        self.offset -= len;
                        break;
                    }
                    if ch == b'#' {
                        self.offset -= 1;
                        break;
                    }
                    s.1 += len;
                }
            }
        }

        str.filter(|s| s.1 != 0)
    }

    /// `lookup_getstringtoken` for the quote at `quote`: up to the closing
    /// unescaped quote, the newline or the end, without the quotes.
    ///
    /// Characterized, not endorsed: a string cut off by a newline or the
    /// end loses its last character if that is an (escaped) quote.
    /// Intentional difference: a quote right before a newline or the end
    /// makes C's length wrap around to SIZE_MAX (and crash in
    /// `bstrdup_n`); here the string is empty, which ends parsing.
    fn string_token(&mut self, quote: usize) -> (usize, usize) {
        let mut temp = self.offset;
        let mut was_backslash = false;

        while !matches!(self.at(temp), 0 | b'\n') {
            if !was_backslash {
                if self.at(temp) == b'\\' {
                    was_backslash = true;
                } else if self.at(temp) == b'"' {
                    temp += 1;
                    break;
                }
            } else {
                was_backslash = false;
            }
            temp += 1;
        }

        // C: len = 1 + (temp - offset), minus the opening quote, minus a
        // closing one.
        let mut len = temp - self.offset;
        if self.at(temp - 1) == b'"' {
            len = len.saturating_sub(1);
        }
        self.offset = temp;
        (quote + 1, len)
    }

    /// `lookup_goto_nextline`: skips to just past the next newline token.
    fn goto_next_line(&mut self) -> bool {
        loop {
            match self.get_token() {
                None => return false,
                Some(t) if self.at(t.0) == b'\n' => return true,
                Some(_) => {}
            }
        }
    }
}
