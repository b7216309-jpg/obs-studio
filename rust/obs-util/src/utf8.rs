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
pub fn utf8_to_wchar<W: WChar>(input: &[u8], mut out: Option<&mut [W]>, flags: i32) -> usize {
    let ignore = flags & UTF8_IGNORE_ERROR != 0;
    let skip_bom = flags & UTF8_SKIP_BOM != 0;
    let mut total = 0;
    let mut written = 0;
    let mut p = 0;

    while p < input.len() {
        let lead = input[p];
        if utf8_forbidden(lead) && !ignore {
            return 0;
        }

        let Some((n, high)) = sequence_header(lead) else {
            if !ignore {
                return 0;
            }
            p += 1;
            continue;
        };

        // The sequence must fit in the input and continue with 10xxxxxx.
        let rest = input.get(p + 1..p + n);
        if !rest.is_some_and(|rest| rest.iter().all(|&b| b & 0xc0 == 0x80)) {
            if !ignore {
                return 0;
            }
            p += 1;
            continue;
        }

        total += 1;
        p += n;

        let Some(out) = out.as_deref_mut() else {
            continue;
        };
        let Some(slot) = out.get_mut(written) else {
            return 0; // no space left
        };

        let code = input[p - n + 1..p]
            .iter()
            .fold(high, |code, &b| (code << 6) | u32::from(b & 0x3f));
        // Written before the checks: a dropped character stays in the
        // buffer past the returned count, as in C.
        *slot = W::from_code(code);

        if is_surrogate(code) {
            if !ignore {
                return 0;
            }
            total -= 1;
        } else if code == BOM && skip_bom {
            total -= 1;
        } else {
            written += 1;
        }
    }

    total
}

/// Encodes wide characters from `input` as UTF-8 into `out`, or only counts
/// when `out` is `None`.
///
/// Returns the number of bytes, or 0 on error (invalid input without
/// [`UTF8_IGNORE_ERROR`], or `out` too small).
pub fn wchar_to_utf8<W: WChar>(input: &[W], mut out: Option<&mut [u8]>, flags: i32) -> usize {
    let ignore = flags & UTF8_IGNORE_ERROR != 0;
    let skip_bom = flags & UTF8_SKIP_BOM != 0;
    let mut total = 0;
    let mut pos = 0;

    for &w in input {
        // A negative wchar_t is neither a surrogate nor the BOM, so checking
        // it first matches C's order.
        let Some(code) = w.code() else {
            if !ignore {
                return 0;
            }
            continue;
        };
        if is_surrogate(code) {
            if !ignore {
                return 0;
            }
            continue;
        }
        if code == BOM && skip_bom {
            continue;
        }

        let n = match code {
            0..=0x7f => 1,
            0x80..=0x7ff => 2,
            0x800..=0xffff => 3,
            0x1_0000..=0x1f_ffff => 4,
            0x20_0000..=0x3ff_ffff => 5,
            _ => 6,
        };
        total += n;

        let Some(out) = out.as_deref_mut() else {
            continue;
        };
        let Some(seq) = out.get_mut(pos..pos + n) else {
            return 0; // no space left
        };

        if n == 1 {
            seq[0] = code as u8;
        } else {
            for (i, byte) in seq[1..].iter_mut().rev().enumerate() {
                *byte = 0x80 | ((code >> (6 * i)) & 0x3f) as u8;
            }
            // Six-byte sequences keep only bit 30; bit 31 (reachable with an
            // unsigned wchar_t) is dropped, as in C.
            let (prefix, mask) = LEAD[n - 2];
            seq[0] = prefix | ((code >> (6 * (n - 1))) & mask) as u8;
        }
        pos += n;
    }

    total
}

/// Lead-byte prefix and payload mask for sequences of 2 to 6 bytes.
const LEAD: [(u8, u32); 5] = [
    (0xc0, 0x1f),
    (0xe0, 0x0f),
    (0xf0, 0x07),
    (0xf8, 0x03),
    (0xfc, 0x01),
];

const BOM: u32 = 0xfeff;

/// `utf8_forbidden` in C: lead octets RFC 3629 never allows.
fn utf8_forbidden(octet: u8) -> bool {
    matches!(octet, 0xc0 | 0xc1 | 0xf5 | 0xff)
}

/// `wchar_forbidden` in C: UTF-16 surrogates.
fn is_surrogate(code: u32) -> bool {
    (0xd800..=0xdfff).contains(&code)
}

/// Sequence length and payload bits of a lead byte, or `None` for a
/// continuation byte or 0xfe/0xff.
fn sequence_header(lead: u8) -> Option<(usize, u32)> {
    let b = u32::from(lead);
    match lead {
        0x00..=0x7f => Some((1, b)),
        _ if lead & 0xe0 == 0xc0 => Some((2, b & 0x1f)),
        _ if lead & 0xf0 == 0xe0 => Some((3, b & 0x0f)),
        _ if lead & 0xf8 == 0xf0 => Some((4, b & 0x07)),
        _ if lead & 0xfc == 0xf8 => Some((5, b & 0x03)),
        _ if lead & 0xfe == 0xfc => Some((6, b & 0x01)),
        _ => None,
    }
}
