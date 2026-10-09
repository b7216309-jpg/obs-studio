//! C ABI shim for the `EXPORT` functions of `libobs/util/dstr.c` (the
//! printf, wide-character and conversion functions live in
//! `dstr-libc.c` and stay in C).
//!
//! [`CDstr`] is the [`DstrStore`] for a caller's `struct dstr`: it edits the
//! buffer in place through `bmalloc`/`brealloc`/`bfree`, so C and Rust can
//! keep sharing dstrs. Every source string is copied before the destination
//! changes, so a source that points into the destination reads as it was
//! before the call (C reads moved or freed memory there).

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;

use crate::dstr::{self as safe, DstrStore};
use crate::ffi::darray::{bfree, bmalloc};
use crate::ffi::lexer::strref;

unsafe extern "C" {
    fn brealloc(ptr: *mut c_void, size: usize) -> *mut c_void;
}

/// Mirrors `struct dstr`.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct dstr {
    pub array: *mut c_char,
    pub len: usize,
    pub capacity: usize,
}

/// A caller's `struct dstr` as a [`DstrStore`].
pub struct CDstr<'a>(&'a mut dstr);

impl CDstr<'_> {
    /// # Safety
    ///
    /// `d` must point to a valid `dstr` whose `array` is NULL or a `bmalloc`
    /// buffer of `capacity` bytes holding `len` bytes of string.
    unsafe fn new<'a>(d: *mut dstr) -> CDstr<'a> {
        // SAFETY: per the contract.
        CDstr(unsafe { &mut *d })
    }
}

impl DstrStore for CDstr<'_> {
    fn is_null(&self) -> bool {
        self.0.array.is_null()
    }
    fn len(&self) -> usize {
        self.0.len
    }
    fn capacity(&self) -> usize {
        self.0.capacity
    }
    fn bytes(&self) -> &[u8] {
        if self.0.array.is_null() {
            &[]
        } else {
            // SAFETY: a valid dstr holds `len` bytes of string at `array`.
            unsafe { core::slice::from_raw_parts(self.0.array.cast::<u8>(), self.0.len) }
        }
    }
    fn set_len(&mut self, len: usize) {
        self.0.len = len;
    }
    fn realloc(&mut self, capacity: usize) {
        // SAFETY: `array` is NULL or a bmalloc buffer; brealloc aborts on
        // failure.
        self.0.array = unsafe { brealloc(self.0.array.cast(), capacity) }.cast();
        self.0.capacity = capacity;
    }
    fn alloc(&mut self, capacity: usize) {
        // SAFETY: bmalloc aborts on failure.
        self.0.array = unsafe { bmalloc(capacity) }.cast();
        self.0.capacity = capacity;
    }
    fn free(&mut self) {
        // SAFETY: `array` is NULL or a bmalloc buffer owned by the dstr.
        unsafe { bfree(self.0.array.cast()) };
        self.init();
    }
    fn init(&mut self) {
        *self.0 = dstr {
            array: ptr::null_mut(),
            len: 0,
            capacity: 0,
        };
    }
    fn write(&mut self, at: usize, src: &[u8]) {
        assert!(
            at + src.len() <= self.0.capacity,
            "write past dstr capacity"
        );
        // SAFETY: the range is within the `capacity`-byte allocation, and
        // `src` is never part of it (sources are copied first).
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), self.0.array.cast::<u8>().add(at), src.len());
        }
    }
}

/// An owned copy of a C string (`None` for NULL).
///
/// # Safety
///
/// `s` must be NULL or a NUL-terminated string.
unsafe fn cstr(s: *const c_char) -> Option<Vec<u8>> {
    // SAFETY: per the contract.
    (!s.is_null()).then(|| unsafe { CStr::from_ptr(s) }.to_bytes().to_vec())
}

/// An owned copy of `n` bytes at `s` (`None` for NULL).
///
/// # Safety
///
/// `s` must be NULL or readable for `n` bytes.
unsafe fn raw(s: *const c_char, n: usize) -> Option<Vec<u8>> {
    // SAFETY: per the contract.
    (!s.is_null()).then(|| unsafe { core::slice::from_raw_parts(s.cast::<u8>(), n) }.to_vec())
}

/// An owned copy of a dstr's string (`None` for a NULL array).
///
/// # Safety
///
/// `d` must point to a valid `dstr`.
unsafe fn dstr_bytes(d: *const dstr) -> Option<Vec<u8>> {
    // SAFETY: per the contract.
    unsafe { raw((*d).array, (*d).len) }
}

/// An owned copy of a strref's bytes (`None` for a NULL array).
///
/// # Safety
///
/// `s` must point to a valid `strref`.
unsafe fn strref_bytes(s: *const strref) -> Option<Vec<u8>> {
    // SAFETY: per the contract.
    unsafe { raw((*s).array, (*s).len) }
}

/// # Safety
///
/// `str1` and `str2` must be NULL or NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn astrcmpi(str1: *const c_char, str2: *const c_char) -> c_int {
    // SAFETY: per the contract.
    let (a, b) = unsafe { (cstr(str1), cstr(str2)) };
    safe::astrcmpi(a.as_deref().unwrap_or(&[]), b.as_deref().unwrap_or(&[]))
}

/// The bytes the `astrcmp*_n` loops may read: up to the NUL or `n`.
///
/// # Safety
///
/// `s` must be NULL, or readable up to its NUL or for `n` bytes.
unsafe fn cstr_n(s: *const c_char, n: usize) -> Vec<u8> {
    if s.is_null() {
        return Vec::new();
    }
    let mut len = 0;
    // SAFETY: only bytes before the NUL and within `n` are read.
    while len < n && unsafe { *s.add(len) } != 0 {
        len += 1;
    }
    // SAFETY: the `len` bytes were just read.
    unsafe { core::slice::from_raw_parts(s.cast::<u8>(), len) }.to_vec()
}

/// # Safety
///
/// `str1` and `str2` must be NULL, or readable up to their NUL or for `n`
/// bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn astrcmp_n(str1: *const c_char, str2: *const c_char, n: usize) -> c_int {
    // SAFETY: per the contract.
    let (a, b) = unsafe { (cstr_n(str1, n), cstr_n(str2, n)) };
    safe::astrcmp_n(&a, &b, n)
}

/// # Safety
///
/// As [`astrcmp_n`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn astrcmpi_n(str1: *const c_char, str2: *const c_char, n: usize) -> c_int {
    // SAFETY: per the contract.
    let (a, b) = unsafe { (cstr_n(str1, n), cstr_n(str2, n)) };
    safe::astrcmpi_n(&a, &b, n)
}

/// # Safety
///
/// `str` and `find` must be NULL or NUL-terminated strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn astrstri(str: *const c_char, find: *const c_char) -> *mut c_char {
    // SAFETY: per the contract.
    let (Some(s), Some(f)) = (unsafe { cstr(str) }, unsafe { cstr(find) }) else {
        return ptr::null_mut();
    };
    match safe::astrstri(&s, &f) {
        // SAFETY: the offset is within the string (at most its NUL).
        Some(i) => unsafe { str.add(i) }.cast_mut(),
        None => ptr::null_mut(),
    }
}

/// # Safety
///
/// `str` must be NULL or a writable NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strdepad(str: *mut c_char) -> *mut c_char {
    if !str.is_null() {
        // SAFETY: the string and its NUL are writable.
        let buf = unsafe {
            let len = CStr::from_ptr(str).to_bytes().len();
            core::slice::from_raw_parts_mut(str.cast::<u8>(), len + 1)
        };
        safe::strdepad(buf);
    }
    str
}

/// Returns one `bmalloc` block: a NULL-terminated table of pointers, then
/// the NUL-terminated pieces. NULL `str` returns NULL.
///
/// # Safety
///
/// `str` must be NULL or a NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlist_split(
    str: *const c_char,
    split_ch: c_char,
    include_empty: bool,
) -> *mut *mut c_char {
    // SAFETY: per the contract.
    let Some(s) = (unsafe { cstr(str) }) else {
        return ptr::null_mut();
    };
    #[allow(clippy::unnecessary_cast)] // c_char is u8 on some targets
    let pieces = safe::strlist_split(&s, split_ch as u8, include_empty);

    let table_size = (pieces.len() + 1) * size_of::<*mut c_char>();
    let total = table_size + pieces.iter().map(|p| p.len() + 1).sum::<usize>();
    // SAFETY: the block holds the table and every piece with its NUL; the
    // table is pointer-aligned (bmalloc aligns to at least 16 bytes).
    unsafe {
        let out = bmalloc(total).cast::<u8>();
        let table = out.cast::<*mut c_char>();
        let mut offset = out.add(table_size);
        for (i, p) in pieces.iter().enumerate() {
            *table.add(i) = offset.cast();
            ptr::copy_nonoverlapping(p.as_ptr(), offset, p.len());
            *offset.add(p.len()) = 0;
            offset = offset.add(p.len() + 1);
        }
        *table.add(pieces.len()) = ptr::null_mut();
        table
    }
}

/// # Safety
///
/// `strlist` must be NULL or a list from [`strlist_split`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn strlist_free(strlist: *mut *mut c_char) {
    // SAFETY: the list is one bmalloc block.
    unsafe { bfree(strlist.cast()) };
}

/// # Safety
///
/// `dst` must be valid for writes; `src` must point to a valid `strref`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_init_copy_strref(dst: *mut dstr, src: *const strref) {
    // SAFETY: per the contract; `init` never reads the old fields.
    unsafe {
        let s = strref_bytes(src).unwrap_or_default();
        safe::dstr_init_copy_strref(&mut CDstr(&mut *dst), &s);
    }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`; `array` must be NULL or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_copy(dst: *mut dstr, array: *const c_char) {
    // SAFETY: per the contract.
    unsafe {
        let s = cstr(array);
        safe::dstr_copy(&mut CDstr::new(dst), s.as_deref());
    }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`; `src` to a valid `strref`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_copy_strref(dst: *mut dstr, src: *const strref) {
    // SAFETY: per the contract.
    unsafe {
        let s = strref_bytes(src).unwrap_or_default();
        safe::dstr_copy_strref(&mut CDstr::new(dst), &s);
    }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`; `array` must be readable for `len`
/// bytes (or NULL with `len` 0).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_ncopy(dst: *mut dstr, array: *const c_char, len: usize) {
    // SAFETY: per the contract.
    unsafe {
        let s = raw(array, len).unwrap_or_default();
        safe::dstr_ncopy(&mut CDstr::new(dst), &s, len);
    }
}

/// # Safety
///
/// `dst` and `str` must point to valid `dstr`s (possibly the same).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_ncopy_dstr(dst: *mut dstr, str: *const dstr, len: usize) {
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = dstr_bytes(str).unwrap_or_default();
        safe::dstr_ncopy_dstr(&mut CDstr::new(dst), &s, len);
    }
}

/// # Safety
///
/// As [`dstr_ncopy_dstr`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_cat_dstr(dst: *mut dstr, str: *const dstr) {
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = dstr_bytes(str).unwrap_or_default();
        safe::dstr_cat_dstr(&mut CDstr::new(dst), &s);
    }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`; `str` to a valid `strref`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_cat_strref(dst: *mut dstr, str: *const strref) {
    // SAFETY: per the contract.
    unsafe { dstr_ncat(dst, (*str).array, (*str).len) }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`; `array` must be NULL, or point to a
/// NUL, or be readable for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_ncat(dst: *mut dstr, array: *const c_char, len: usize) {
    // SAFETY: per the contract: a NUL first byte is read alone.
    unsafe {
        let s = if array.is_null() || *array == 0 {
            None
        } else {
            raw(array, len)
        };
        safe::dstr_ncat(&mut CDstr::new(dst), s.as_deref(), len);
    }
}

/// # Safety
///
/// As [`dstr_ncopy_dstr`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_ncat_dstr(dst: *mut dstr, str: *const dstr, len: usize) {
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = dstr_bytes(str);
        safe::dstr_ncat_dstr(&mut CDstr::new(dst), s.as_deref(), len);
    }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`; `array` must be NULL or a
/// NUL-terminated string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_insert(dst: *mut dstr, idx: usize, array: *const c_char) {
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = cstr(array);
        safe::dstr_insert(&mut CDstr::new(dst), idx, s.as_deref());
    }
}

/// # Safety
///
/// As [`dstr_ncopy_dstr`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_insert_dstr(dst: *mut dstr, idx: usize, str: *const dstr) {
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = dstr_bytes(str).unwrap_or_default();
        safe::dstr_insert_dstr(&mut CDstr::new(dst), idx, &s);
    }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_insert_ch(dst: *mut dstr, idx: usize, ch: c_char) {
    // SAFETY: per the contract.
    #[allow(clippy::unnecessary_cast)] // c_char is u8 on some targets
    unsafe {
        safe::dstr_insert_ch(&mut CDstr::new(dst), idx, ch as u8)
    }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_remove(dst: *mut dstr, idx: usize, count: usize) {
    // SAFETY: per the contract.
    unsafe { safe::dstr_remove(&mut CDstr::new(dst), idx, count) }
}

/// # Safety
///
/// `dst` must point to a valid `dstr`; the strings must be NULL or
/// NUL-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_safe_printf(
    dst: *mut dstr,
    format: *const c_char,
    val1: *const c_char,
    val2: *const c_char,
    val3: *const c_char,
    val4: *const c_char,
) {
    // SAFETY: per the contract; all strings are copied first.
    unsafe {
        let f = cstr(format);
        let v = [cstr(val1), cstr(val2), cstr(val3), cstr(val4)];
        let vals = [
            v[0].as_deref(),
            v[1].as_deref(),
            v[2].as_deref(),
            v[3].as_deref(),
        ];
        safe::dstr_safe_printf(&mut CDstr::new(dst), f.as_deref(), vals);
    }
}

/// Intentional differences: a NULL `find` does nothing (C dereferences it),
/// and an empty one too (C loops forever).
///
/// # Safety
///
/// `str` must point to a valid `dstr`; `find` and `replace` must be NULL or
/// NUL-terminated.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_replace(str: *mut dstr, find: *const c_char, replace: *const c_char) {
    // SAFETY: per the contract; both strings are copied first.
    unsafe {
        let Some(f) = cstr(find) else {
            return;
        };
        let r = cstr(replace);
        safe::dstr_replace(&mut CDstr::new(str), &f, r.as_deref());
    }
}

/// # Safety
///
/// `str` must point to a valid `dstr`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_depad(str: *mut dstr) {
    // SAFETY: per the contract.
    unsafe { safe::dstr_depad(&mut CDstr::new(str)) }
}

/// # Safety
///
/// `dst` and `str` must point to valid `dstr`s (possibly the same).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_left(dst: *mut dstr, str: *const dstr, pos: usize) {
    let same = ptr::eq(dst, str);
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = if same {
            Vec::new()
        } else {
            dstr_bytes(str).unwrap_or_default()
        };
        safe::dstr_left(&mut CDstr::new(dst), &s, same, pos);
    }
}

/// # Safety
///
/// As [`dstr_left`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_mid(dst: *mut dstr, str: *const dstr, start: usize, count: usize) {
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = dstr_bytes(str).unwrap_or_default();
        safe::dstr_mid(&mut CDstr::new(dst), &s, start, count);
    }
}

/// # Safety
///
/// As [`dstr_left`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dstr_right(dst: *mut dstr, str: *const dstr, pos: usize) {
    // SAFETY: per the contract; the source is copied first.
    unsafe {
        let s = dstr_bytes(str).unwrap_or_default();
        safe::dstr_right(&mut CDstr::new(dst), &s, pos);
    }
}
