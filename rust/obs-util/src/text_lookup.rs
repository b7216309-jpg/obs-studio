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

use std::ffi::CStr;
use std::path::Path;

/// An opaque `lookup_t`.
#[derive(Debug, Default)]
pub struct TextLookup {}

impl TextLookup {
    pub fn new() -> Self {
        Self::default()
    }

    /// `text_lookup_add`.
    pub fn add_file(&mut self, path: &Path) -> bool {
        let _ = path;
        todo!()
    }

    /// `text_lookup_add` on file contents already read.
    pub fn add_data(&mut self, data: &[u8]) -> bool {
        let _ = data;
        todo!()
    }

    /// `text_lookup_getstr`.
    pub fn get(&self, key: &[u8]) -> Option<&CStr> {
        let _ = key;
        todo!()
    }
}
