//! Safe core for `libobs/util/dstr.c`: byte-string compares, `strdepad`,
//! `strlist_split`, and the `struct dstr` editing functions.
//!
//! The `dstr` functions are generic over [`DstrStore`], the C struct's
//! state (`array`, `len`, `capacity`) plus the allocator calls C makes on
//! it, so the shim can edit the caller's buffer in place while
//! [`VecDstr`] backs the Rust tests. Growth follows the `dstr.h` inline
//! `dstr_ensure_capacity` exactly, so `capacity` matches C.
//!
//! Sources are byte slices read before the destination changes. Where C
//! would read past a source (or through a NULL one), the missing bytes
//! read as NUL. Strings compare as the platform's C `char`; `toupper` is
//! the C locale's, ASCII only.

/// The state of a `struct dstr` and the allocator calls C makes on it.
// `is_empty` would read like `dstr_is_empty`, which also checks the first
// byte; the core uses its own `dstr_is_empty` instead.
#[allow(clippy::len_without_is_empty)]
pub trait DstrStore {
    /// `array == NULL`.
    fn is_null(&self) -> bool;
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    /// The string, `array[0..len]`; empty when `array` is NULL.
    fn bytes(&self) -> &[u8];
    fn set_len(&mut self, len: usize);
    /// `array = brealloc(array, capacity)`, keeping the old contents.
    fn realloc(&mut self, capacity: usize);
    /// `array = bmalloc(capacity)` on a NULL array (C's `bmemdup`).
    fn alloc(&mut self, capacity: usize);
    /// `dstr_free`: `bfree(array)`, then NULL / 0 / 0.
    fn free(&mut self);
    /// `dstr_init`: NULL / 0 / 0 without freeing.
    fn init(&mut self);
    /// Copies `src` to `array[at..at + src.len()]` (within `capacity`).
    fn write(&mut self, at: usize, src: &[u8]);
}

/// A [`DstrStore`] on a `Vec`, for Rust callers and tests.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VecDstr {
    /// The allocation, `capacity` bytes, or `None` for a NULL array.
    pub buf: Option<Vec<u8>>,
    pub len: usize,
    pub capacity: usize,
}

impl VecDstr {
    /// A dstr holding `s`, as `dstr_copy` leaves it.
    pub fn from(s: &[u8]) -> Self {
        let mut d = Self::default();
        dstr_copy(&mut d, Some(s));
        d
    }

    /// The string up to its NUL, as C sees `array` (`None` when NULL).
    pub fn c_str(&self) -> Option<&[u8]> {
        self.buf.as_deref().map(|b| &b[..cstrlen(b, 0)])
    }
}

impl DstrStore for VecDstr {
    fn is_null(&self) -> bool {
        self.buf.is_none()
    }
    fn len(&self) -> usize {
        self.len
    }
    fn capacity(&self) -> usize {
        self.capacity
    }
    fn bytes(&self) -> &[u8] {
        self.buf
            .as_deref()
            .map_or(&[], |b| &b[..self.len.min(b.len())])
    }
    fn set_len(&mut self, len: usize) {
        self.len = len;
    }
    fn realloc(&mut self, capacity: usize) {
        self.buf.get_or_insert_with(Vec::new).resize(capacity, 0);
        self.capacity = capacity;
    }
    fn alloc(&mut self, capacity: usize) {
        self.buf = Some(vec![0; capacity]);
        self.capacity = capacity;
    }
    fn free(&mut self) {
        self.init();
    }
    fn init(&mut self) {
        *self = Self::default();
    }
    fn write(&mut self, at: usize, src: &[u8]) {
        self.buf.as_mut().expect("allocated")[at..at + src.len()].copy_from_slice(src);
    }
}

/// `strlen(&s[from..])`, stopping at the end of `s`.
fn cstrlen(s: &[u8], from: usize) -> usize {
    s.get(from..)
        .map_or(0, |t| t.iter().position(|&b| b == 0).unwrap_or(t.len()))
}

/// `astrcmpi`. Strings are the bytes before their NUL; NULL is empty.
pub fn astrcmpi(s1: &[u8], s2: &[u8]) -> i32 {
    let _ = (s1, s2);
    todo!()
}

/// `astrcmp_n`.
pub fn astrcmp_n(s1: &[u8], s2: &[u8], n: usize) -> i32 {
    let _ = (s1, s2, n);
    todo!()
}

/// `astrcmpi_n`.
pub fn astrcmpi_n(s1: &[u8], s2: &[u8], n: usize) -> i32 {
    let _ = (s1, s2, n);
    todo!()
}

/// `astrstri`: the offset of the first case-insensitive match of `find`
/// in `s`. An empty `find` matches at 0.
pub fn astrstri(s: &[u8], find: &[u8]) -> Option<usize> {
    let _ = (s, find);
    todo!()
}

/// `strdepad` on a NUL-terminated buffer, in place: removes leading and
/// trailing spaces, tabs, CRs and LFs.
pub fn strdepad(buf: &mut [u8]) {
    let _ = buf;
    todo!()
}

/// `strlist_split`: the pieces of `s` between `split_ch`, skipping empty
/// ones unless `include_empty`.
///
/// Intentional difference: with `split_ch` NUL, C reads past the
/// terminator; here the whole string is one piece.
pub fn strlist_split(s: &[u8], split_ch: u8, include_empty: bool) -> Vec<&[u8]> {
    let _ = (s, split_ch, include_empty);
    todo!()
}

/// `dstr_ensure_capacity` from `dstr.h`. A NULL array is always allocated
/// (C would write through it when `capacity` is already large enough).
pub fn dstr_ensure_capacity(d: &mut impl DstrStore, new_size: usize) {
    let _ = (d, new_size);
    todo!()
}

/// `dstr_copy`. `src` is the C string (`None` for NULL); NULL or empty
/// frees `d`.
pub fn dstr_copy(d: &mut impl DstrStore, src: Option<&[u8]>) {
    let _ = (d, src);
    todo!()
}

/// `dstr_ncopy`: `len` bytes of `src` (past its end read as NUL), in a new
/// allocation of exactly `len + 1`.
pub fn dstr_ncopy(d: &mut impl DstrStore, src: &[u8], len: usize) {
    let _ = (d, src, len);
    todo!()
}

/// `dstr_ncopy_dstr`: `src` is the source dstr's string.
pub fn dstr_ncopy_dstr(d: &mut impl DstrStore, src: &[u8], len: usize) {
    let _ = (d, src, len);
    todo!()
}

/// `dstr_copy_strref`.
pub fn dstr_copy_strref(d: &mut impl DstrStore, src: &[u8]) {
    let _ = (d, src);
    todo!()
}

/// `dstr_init_copy_strref`: forgets (does not free) what `d` held.
pub fn dstr_init_copy_strref(d: &mut impl DstrStore, src: &[u8]) {
    let _ = (d, src);
    todo!()
}

/// `dstr_cat_dstr`: `src` is the source dstr's string.
pub fn dstr_cat_dstr(d: &mut impl DstrStore, src: &[u8]) {
    let _ = (d, src);
    todo!()
}

/// `dstr_ncat`: appends `len` bytes of `src` (NUL bytes included, past its
/// end read as NUL). NULL (`None`), a NUL first byte, or `len` 0 do
/// nothing.
pub fn dstr_ncat(d: &mut impl DstrStore, src: Option<&[u8]>, len: usize) {
    let _ = (d, src, len);
    todo!()
}

/// `dstr_cat_strref`.
pub fn dstr_cat_strref(d: &mut impl DstrStore, src: Option<&[u8]>, len: usize) {
    let _ = (d, src, len);
    todo!()
}

/// `dstr_ncat_dstr`: `src` is the source dstr's string (`None` for a NULL
/// array).
pub fn dstr_ncat_dstr(d: &mut impl DstrStore, src: Option<&[u8]>, len: usize) {
    let _ = (d, src, len);
    todo!()
}

/// `dstr_insert`. `src` is the C string (`None` for NULL).
///
/// Intentional difference: `idx > len` does nothing (C moves memory from
/// before or past the buffer).
pub fn dstr_insert(d: &mut impl DstrStore, idx: usize, src: Option<&[u8]>) {
    let _ = (d, idx, src);
    todo!()
}

/// `dstr_insert_dstr`: `src` is the source dstr's string. `idx > len` does
/// nothing, as for [`dstr_insert`].
pub fn dstr_insert_dstr(d: &mut impl DstrStore, idx: usize, src: &[u8]) {
    let _ = (d, idx, src);
    todo!()
}

/// `dstr_insert_ch`. Intentional difference: C moves one byte more than
/// needed, writing `array[len + 2]`, past the buffer when `capacity` is
/// exactly `len + 2`; that byte is past the string's NUL and is not
/// written here. `idx > len` does nothing.
pub fn dstr_insert_ch(d: &mut impl DstrStore, idx: usize, c: u8) {
    let _ = (d, idx, c);
    todo!()
}

/// `dstr_remove`. Characterized, not endorsed: removing `count == len`
/// frees the dstr whatever `idx` is. Intentional difference:
/// `idx + count > len` does nothing (C moves memory past the string).
pub fn dstr_remove(d: &mut impl DstrStore, idx: usize, count: usize) {
    let _ = (d, idx, count);
    todo!()
}

/// `dstr_replace`: replaces every `find` with `replace` (`None` is empty)
/// in the C string, left to right.
///
/// Intentional difference: an empty `find` does nothing (C loops forever).
pub fn dstr_replace(d: &mut impl DstrStore, find: &[u8], replace: Option<&[u8]>) {
    let _ = (d, find, replace);
    todo!()
}

/// `dstr_safe_printf`: copies `format`, then replaces `$1`..`$4` with each
/// value that is not `None`.
pub fn dstr_safe_printf(d: &mut impl DstrStore, format: Option<&[u8]>, values: [Option<&[u8]>; 4]) {
    let _ = (d, format, values);
    todo!()
}

/// `dstr_depad`: [`strdepad`] in place; an all-padding string frees `d`.
pub fn dstr_depad(d: &mut impl DstrStore) {
    let _ = d;
    todo!()
}

/// `dstr_left`: the first `pos` bytes of `src` (the source dstr's string;
/// past its end read as NUL). `same` is `dst == str`, which truncates in
/// place.
pub fn dstr_left(d: &mut impl DstrStore, src: &[u8], same: bool, pos: usize) {
    let _ = (d, src, same, pos);
    todo!()
}

/// `dstr_mid`: `count` bytes of `src` from `start` (past its end read as
/// NUL), in a new allocation of exactly `count + 1`.
pub fn dstr_mid(d: &mut impl DstrStore, src: &[u8], start: usize, count: usize) {
    let _ = (d, src, start, count);
    todo!()
}

/// `dstr_right`: `src` from `pos` on. Intentional difference: `pos > len`
/// leaves `d` empty (C copies a wrapped-around length).
pub fn dstr_right(d: &mut impl DstrStore, src: &[u8], pos: usize) {
    let _ = (d, src, pos);
    todo!()
}
