//! Safe core for `utf8_to_wchar` and `wchar_to_utf8` (`libobs/util/utf8.c`,
//! non-Windows implementation).
//!
//! The C code converts between UTF-8 and UCS-4 held in `wchar_t`, whose
//! signedness depends on the target (`int` on x86 Linux and macOS,
//! `unsigned int` on Arm Linux). Only `wchar_to_utf8` observes it: a negative
//! `wchar_t` is an error, while the same bits as an unsigned value encode as a
//! six-byte sequence. The core is generic over [`WChar`] so the shim picks the
//! platform's type and the logic lives here once.
//!
//! Characterized, not endorsed: like the C code, this accepts overlong forms
//! and code points above U+10FFFF (five- and six-byte sequences), and only
//! applies the surrogate and BOM checks to `utf8_to_wchar` when writing output.

/// `UTF8_IGNORE_ERROR`: skip invalid input instead of failing.
pub const UTF8_IGNORE_ERROR: i32 = 0x01;
/// `UTF8_SKIP_BOM`: drop U+FEFF from the output.
pub const UTF8_SKIP_BOM: i32 = 0x02;

/// A 32-bit `wchar_t`, signed or unsigned.
pub trait WChar: Copy {
    /// The code point this value holds, or `None` if it is negative.
    fn code(self) -> Option<u32>;
    /// Stores a decoded code point (at most `0x7FFF_FFFF`).
    fn from_code(code: u32) -> Self;
}

impl WChar for i32 {
    fn code(self) -> Option<u32> {
        u32::try_from(self).ok()
    }

    fn from_code(code: u32) -> Self {
        code as i32
    }
}

impl WChar for u32 {
    fn code(self) -> Option<u32> {
        Some(self)
    }

    fn from_code(code: u32) -> Self {
        code
    }
}

/// Decodes UTF-8 `input` into `out`, or only counts when `out` is `None`.
///
/// Returns the number of wide characters, or 0 on error (invalid input
/// without [`UTF8_IGNORE_ERROR`], or `out` too small). On error `out` keeps
/// whatever was written before it, as in C.
pub fn utf8_to_wchar<W: WChar>(input: &[u8], out: Option<&mut [W]>, flags: i32) -> usize {
    let _ = (input, out, flags);
    todo!()
}

/// Encodes wide characters from `input` as UTF-8 into `out`, or only counts
/// when `out` is `None`.
///
/// Returns the number of bytes, or 0 on error (invalid input without
/// [`UTF8_IGNORE_ERROR`], or `out` too small).
pub fn wchar_to_utf8<W: WChar>(input: &[W], out: Option<&mut [u8]>, flags: i32) -> usize {
    let _ = (input, out, flags);
    todo!()
}
