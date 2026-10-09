//! C ABI shim for the `EXPORT` functions of `libobs/util/lexer.h`.
//!
//! The structs mirror the header (the inline helpers there keep operating on
//! them from C). The functions only convert pointers to slices and delegate
//! to [`crate::lexer`]; `error_data_add` and `error_data_buildstring` also
//! allocate through `bmalloc`, since C frees the results with `bfree`.

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr::{self, addr_of_mut};

use crate::ffi::darray::{bmalloc, darray, darray_push_back_array};
use crate::lexer as safe;

/// Mirrors `struct strref`.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct strref {
    pub array: *const c_char,
    pub len: usize,
}

/// Mirrors `struct base_token`. `type` is `enum base_token_type`.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct base_token {
    pub text: strref,
    pub r#type: c_int,
    pub passed_whitespace: bool,
}

/// Mirrors `struct error_item`.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub struct error_item {
    pub error: *mut c_char,
    pub file: *const c_char,
    pub row: u32,
    pub column: u32,
    pub level: c_int,
}

/// Mirrors `struct error_data`, whose `DARRAY(struct error_item)` has the
/// layout of `struct darray`.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct error_data {
    pub errors: darray,
}

/// Mirrors `struct lexer`.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct lexer {
    pub text: *mut c_char,
    pub offset: *const c_char,
}

/// `IGNORE_WHITESPACE` of `enum ignore_whitespace`.
const IGNORE_WHITESPACE: c_int = 1;

/// The bytes of a `strref`; NULL (or a NULL array) is empty.
///
/// # Safety
///
/// `s` must be null or point to a valid `strref` whose `array` is null or
/// readable for `len` bytes.
unsafe fn strref_bytes<'a>(s: *const strref) -> &'a [u8] {
    // SAFETY: per the contract.
    match unsafe { s.as_ref() } {
        Some(s) if !s.array.is_null() && s.len != 0 => unsafe {
            core::slice::from_raw_parts(s.array.cast::<u8>(), s.len)
        },
        _ => &[],
    }
}

/// The bytes of a C string up to its NUL; NULL is empty.
///
/// # Safety
///
/// `s` must be null or point to a NUL-terminated string.
unsafe fn cstr_bytes<'a>(s: *const c_char) -> &'a [u8] {
    if s.is_null() {
        &[]
    } else {
        // SAFETY: per the contract.
        unsafe { CStr::from_ptr(s) }.to_bytes()
    }
}

/// The bytes at `s` up to its NUL or `max` bytes, whichever comes first.
///
/// # Safety
///
/// `s` must be readable up to its NUL or for `max` bytes.
unsafe fn strn_bytes<'a>(s: *const c_char, max: usize) -> &'a [u8] {
    let mut len = 0;
    // SAFETY: only bytes before the NUL and within `max` are read.
    while len < max && unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: the `len` bytes were just read.
    unsafe { core::slice::from_raw_parts(s.cast::<u8>(), len) }
}

/// `bstrdup` from `util/bmem.h`.
///
/// # Safety
///
/// `s` must be null or point to a NUL-terminated string.
unsafe fn bstrdup(s: *const c_char) -> *mut c_char {
    if s.is_null() {
        return ptr::null_mut();
    }
    // SAFETY: `s` is a NUL-terminated string.
    let bytes = unsafe { CStr::from_ptr(s) }.to_bytes_with_nul();
    bmalloc_copy(bytes)
}

/// Copies `bytes` into a new `bmalloc` buffer.
fn bmalloc_copy(bytes: &[u8]) -> *mut c_char {
    // SAFETY: bmalloc returns a buffer of the requested size (it aborts on
    // failure), which cannot overlap `bytes`.
    unsafe {
        let dup = bmalloc(bytes.len()).cast::<u8>();
        ptr::copy_nonoverlapping(bytes.as_ptr(), dup, bytes.len());
        dup.cast()
    }
}

/// # Safety
///
/// `str1` must be null or point to a valid `strref`; `str2` must be null or
/// point to a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strref_cmp(str1: *const strref, str2: *const c_char) -> c_int {
    // SAFETY: per the contract.
    unsafe { safe::strref_cmp(strref_bytes(str1), cstr_bytes(str2)) }
}

/// # Safety
///
/// As [`strref_cmp`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strref_cmpi(str1: *const strref, str2: *const c_char) -> c_int {
    // SAFETY: per the contract.
    unsafe { safe::strref_cmpi(strref_bytes(str1), cstr_bytes(str2)) }
}

/// # Safety
///
/// `str1` and `str2` must each be null or point to a valid `strref`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strref_cmp_strref(str1: *const strref, str2: *const strref) -> c_int {
    // SAFETY: per the contract.
    unsafe { safe::strref_cmp_strref(strref_bytes(str1), strref_bytes(str2)) }
}

/// # Safety
///
/// As [`strref_cmp_strref`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strref_cmpi_strref(str1: *const strref, str2: *const strref) -> c_int {
    // SAFETY: per the contract.
    unsafe { safe::strref_cmpi_strref(strref_bytes(str1), strref_bytes(str2)) }
}

/// The bytes `valid_*_str` may read: up to the NUL, and with `n` set at most
/// a sign plus `n` characters.
///
/// # Safety
///
/// `s` must be non-null and readable up to its NUL, or (with `n` set) for
/// `n + 1` bytes.
unsafe fn number_bytes<'a>(s: *const c_char, n: usize) -> &'a [u8] {
    let max = if n == 0 {
        usize::MAX
    } else {
        n.saturating_add(1)
    };
    // SAFETY: per the contract.
    unsafe { strn_bytes(s, max) }
}

/// # Safety
///
/// `str` must be null, or readable up to its NUL or (with `n` set) for a
/// sign plus `n` characters.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn valid_int_str(str: *const c_char, n: usize) -> bool {
    // SAFETY: per the contract.
    !str.is_null() && safe::valid_int_str(unsafe { number_bytes(str, n) }, n)
}

/// # Safety
///
/// As [`valid_int_str`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn valid_float_str(str: *const c_char, n: usize) -> bool {
    // SAFETY: per the contract.
    !str.is_null() && safe::valid_float_str(unsafe { number_bytes(str, n) }, n)
}

/// # Safety
///
/// `data` must be null or point to a valid `error_data` whose array came
/// from `bmalloc`; `msg` must be null or a NUL-terminated string. `file` is
/// stored, not read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn error_data_add(
    data: *mut error_data,
    file: *const c_char,
    row: u32,
    column: u32,
    msg: *const c_char,
    level: c_int,
) {
    if data.is_null() {
        return;
    }
    let item = error_item {
        // SAFETY: per the contract.
        error: unsafe { bstrdup(msg) },
        file,
        row,
        column,
        level,
    };
    // SAFETY: `data` is valid, and `item` is one readable element.
    unsafe {
        darray_push_back_array(
            size_of::<error_item>(),
            addr_of_mut!((*data).errors),
            ptr::from_ref(&item).cast::<c_void>(),
            1,
        );
    }
}

/// Returns a `bmalloc` string with one line per error, or NULL when there
/// are none.
///
/// # Safety
///
/// `ed` must point to a valid `error_data` whose items' `file` and `error`
/// are null or NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn error_data_buildstring(ed: *mut error_data) -> *mut c_char {
    // SAFETY: `ed` is valid, so its array holds `num` items.
    let items: &[error_item] = unsafe {
        let errors = &(*ed).errors;
        if errors.num == 0 {
            return ptr::null_mut();
        }
        core::slice::from_raw_parts(errors.array.cast::<error_item>(), errors.num)
    };

    let mut out = Vec::new();
    for item in items {
        let opt = |s: *const c_char| {
            // SAFETY: per the contract.
            (!s.is_null()).then(|| unsafe { CStr::from_ptr(s) }.to_bytes())
        };
        safe::format_error_item(
            &mut out,
            opt(item.file),
            item.row,
            item.column,
            opt(item.error),
        );
    }
    out.push(0);
    bmalloc_copy(&out)
}

/// # Safety
///
/// `lex` must point to a valid `lexer` whose `offset` is null or points into
/// a NUL-terminated string; `token` must be valid for writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lexer_getbasetoken(
    lex: *mut lexer,
    token: *mut base_token,
    iws: c_int,
) -> bool {
    // SAFETY: `lex` is valid.
    let offset = unsafe { (*lex).offset };
    if offset.is_null() {
        return false;
    }
    // SAFETY: `offset` points into a NUL-terminated string.
    let text = unsafe { CStr::from_ptr(offset) }.to_bytes();
    let (advance, found) = safe::get_base_token(text, iws == IGNORE_WHITESPACE);

    // SAFETY: `advance <= text.len()`, so the new offset is at most the NUL,
    // and `token` is valid for writes. Only `text` and `type` are written,
    // as in C.
    unsafe {
        (*lex).offset = offset.add(advance);
        if let Some(t) = found {
            addr_of_mut!((*token).text).write(strref {
                array: offset.add(t.start),
                len: t.len,
            });
            addr_of_mut!((*token).r#type).write(t.kind as c_int);
        }
    }
    found.is_some()
}

/// # Safety
///
/// `lex` must point to a valid `lexer`; `str` must be null or point into its
/// text at or before the NUL; `row` and `col` must be valid for writes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lexer_getstroffset(
    lex: *const lexer,
    str: *const c_char,
    row: *mut u32,
    col: *mut u32,
) {
    // SAFETY: `lex` is valid.
    let text = unsafe { (*lex).text };
    // Intentional difference: C dereferences a NULL text; leave the outputs
    // untouched instead, as for a NULL `str`.
    if str.is_null() || text.is_null() {
        return;
    }

    let pos = (str as usize).saturating_sub(text as usize);
    // C peeks one byte past `pos` after a newline; read it only then, as C
    // does, since it may be the NUL.
    // SAFETY: `str` is within the text, so `pos` bytes are readable, and so
    // is `text[pos]` (at worst the NUL).
    let bytes = unsafe {
        let peek = pos > 0 && matches!(*text.cast::<u8>().add(pos - 1), b'\r' | b'\n');
        core::slice::from_raw_parts(text.cast::<u8>(), pos + usize::from(peek))
    };
    let (r, c) = safe::get_str_offset(bytes, pos);
    // SAFETY: `row` and `col` are valid for writes.
    unsafe {
        *row = r;
        *col = c;
    }
}
