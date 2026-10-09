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

use core::ffi::c_char;

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

fn ch(b: u8) -> c_char {
    b as c_char
}

fn upper(b: u8) -> c_char {
    ch(b.to_ascii_uppercase())
}

fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// `strlen(&s[from..])`, stopping at the end of `s`.
fn cstrlen(s: &[u8], from: usize) -> usize {
    s.get(from..)
        .map_or(0, |t| t.iter().position(|&b| b == 0).unwrap_or(t.len()))
}

/// The first `n` bytes of `s`, padded with NUL where `s` is shorter.
fn take_padded(s: &[u8], n: usize) -> Vec<u8> {
    let mut v = s[..n.min(s.len())].to_vec();
    v.resize(n, 0);
    v
}

/// Shared loop of the `astrcmp*` functions: `limit` is `n` (0 means no
/// limit, used by `astrcmpi`), `map` the character transform.
fn astrcmp_with(s1: &[u8], s2: &[u8], limit: Option<usize>, map: fn(u8) -> c_char) -> i32 {
    let mut n = match limit {
        Some(0) => return 0,
        Some(n) => n,
        None => usize::MAX,
    };
    let mut i = 0;
    loop {
        let (c1, c2) = (map(at(s1, i)), map(at(s2, i)));
        if c1 < c2 {
            return -1;
        } else if c1 > c2 {
            return 1;
        }
        // C: while (*str1++ && *str2++ [&& --n])
        if at(s1, i) == 0 || at(s2, i) == 0 {
            return 0;
        }
        if limit.is_some() {
            n -= 1;
            if n == 0 {
                return 0;
            }
        }
        i += 1;
    }
}

/// `astrcmpi`. Strings are the bytes before their NUL; NULL is empty.
pub fn astrcmpi(s1: &[u8], s2: &[u8]) -> i32 {
    astrcmp_with(s1, s2, None, upper)
}

/// `astrcmp_n`.
pub fn astrcmp_n(s1: &[u8], s2: &[u8], n: usize) -> i32 {
    astrcmp_with(s1, s2, Some(n), ch)
}

/// `astrcmpi_n`.
pub fn astrcmpi_n(s1: &[u8], s2: &[u8], n: usize) -> i32 {
    astrcmp_with(s1, s2, Some(n), upper)
}

/// `astrstri`: the offset of the first case-insensitive match of `find`
/// in `s`. An empty `find` matches at 0.
pub fn astrstri(s: &[u8], find: &[u8]) -> Option<usize> {
    (0..=s.len()).find(|&i| astrcmpi_n(&s[i..], find, find.len()) == 0)
}

fn is_padding(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// `strdepad` on a NUL-terminated buffer, in place: removes leading and
/// trailing spaces, tabs, CRs and LFs.
pub fn strdepad(buf: &mut [u8]) {
    if at(buf, 0) == 0 {
        return;
    }
    let lead = buf.iter().take_while(|&&b| is_padding(b)).count();
    let len = cstrlen(buf, lead);
    if lead != 0 {
        buf.copy_within(lead..lead + len + 1, 0);
    }
    // buf[0] is not padding once len > 0, so this stops before index 0.
    let mut t = len;
    while t > 0 && is_padding(buf[t - 1]) {
        buf[t - 1] = 0;
        t -= 1;
    }
}

/// `strlist_split`: the pieces of `s` between `split_ch`, skipping empty
/// ones unless `include_empty`.
///
/// Intentional difference: with `split_ch` NUL, C reads past the
/// terminator; here the whole string is one piece.
pub fn strlist_split(s: &[u8], split_ch: u8, include_empty: bool) -> Vec<&[u8]> {
    let keep = |p: &&[u8]| !p.is_empty() || include_empty;
    if split_ch == 0 {
        return Some(s).filter(keep).into_iter().collect();
    }
    s.split(|&b| b == split_ch).filter(keep).collect()
}

/// `dstr_ensure_capacity` from `dstr.h`. A NULL array is always allocated
/// (C would write through it when `capacity` is already large enough).
pub fn dstr_ensure_capacity(d: &mut impl DstrStore, new_size: usize) {
    if new_size <= d.capacity() && !d.is_null() {
        return;
    }
    let cap = d.capacity();
    let new_cap = if cap == 0 { new_size } else { cap * 2 }.max(new_size);
    d.realloc(new_cap);
}

fn dstr_is_empty(d: &impl DstrStore) -> bool {
    d.is_null() || d.len() == 0 || d.bytes()[0] == 0
}

fn free_if_array(d: &mut impl DstrStore) {
    if !d.is_null() {
        d.free();
    }
}

/// Writes `s` and a NUL at `at`.
fn write_cstr(d: &mut impl DstrStore, at: usize, s: &[u8]) {
    d.write(at, s);
    d.write(at + s.len(), &[0]);
}

/// `dstr_copy`. `src` is the C string (`None` for NULL); NULL or empty
/// frees `d`.
pub fn dstr_copy(d: &mut impl DstrStore, src: Option<&[u8]>) {
    let Some(src) = src.filter(|s| !s.is_empty()) else {
        d.free();
        return;
    };
    dstr_ensure_capacity(d, src.len() + 1);
    write_cstr(d, 0, src);
    d.set_len(src.len());
}

/// `dstr_ncopy`: `len` bytes of `src` (past its end read as NUL), in a new
/// allocation of exactly `len + 1`.
pub fn dstr_ncopy(d: &mut impl DstrStore, src: &[u8], len: usize) {
    free_if_array(d);
    if len == 0 {
        return;
    }
    d.alloc(len + 1);
    write_cstr(d, 0, &take_padded(src, len));
    d.set_len(len);
}

/// `dstr_ncopy_dstr`: `src` is the source dstr's string.
pub fn dstr_ncopy_dstr(d: &mut impl DstrStore, src: &[u8], len: usize) {
    free_if_array(d);
    if len == 0 {
        return;
    }
    let n = len.min(src.len());
    d.alloc(n + 1);
    write_cstr(d, 0, &src[..n]);
    d.set_len(n);
}

/// `dstr_copy_strref`.
pub fn dstr_copy_strref(d: &mut impl DstrStore, src: &[u8]) {
    free_if_array(d);
    dstr_ncopy(d, src, src.len());
}

/// `dstr_init_copy_strref`: forgets (does not free) what `d` held.
pub fn dstr_init_copy_strref(d: &mut impl DstrStore, src: &[u8]) {
    d.init();
    dstr_copy_strref(d, src);
}

/// `dstr_cat_dstr`: `src` is the source dstr's string.
pub fn dstr_cat_dstr(d: &mut impl DstrStore, src: &[u8]) {
    if src.is_empty() {
        return;
    }
    let len = d.len();
    dstr_ensure_capacity(d, len + src.len() + 1);
    write_cstr(d, len, src);
    d.set_len(len + src.len());
}

/// `dstr_ncat`: appends `len` bytes of `src` (NUL bytes included, past its
/// end read as NUL). NULL (`None`), a NUL first byte, or `len` 0 do
/// nothing.
pub fn dstr_ncat(d: &mut impl DstrStore, src: Option<&[u8]>, len: usize) {
    let Some(src) = src.filter(|s| at(s, 0) != 0) else {
        return;
    };
    if len == 0 {
        return;
    }
    let old = d.len();
    dstr_ensure_capacity(d, old + len + 1);
    write_cstr(d, old, &take_padded(src, len));
    d.set_len(old + len);
}

/// `dstr_cat_strref`.
pub fn dstr_cat_strref(d: &mut impl DstrStore, src: Option<&[u8]>, len: usize) {
    dstr_ncat(d, src, len);
}

/// `dstr_ncat_dstr`: `src` is the source dstr's string (`None` for a NULL
/// array).
pub fn dstr_ncat_dstr(d: &mut impl DstrStore, src: Option<&[u8]>, len: usize) {
    let Some(src) = src.filter(|s| at(s, 0) != 0) else {
        return;
    };
    if len == 0 {
        return;
    }
    let n = len.min(src.len());
    let old = d.len();
    dstr_ensure_capacity(d, old + n + 1);
    write_cstr(d, old, &src[..n]);
    d.set_len(old + n);
}

/// `dstr_cat` from `dstr.h`.
fn dstr_cat(d: &mut impl DstrStore, src: &[u8]) {
    if !src.is_empty() {
        dstr_ncat(d, Some(src), src.len());
    }
}

/// Inserts `src` at `idx < len`.
fn insert_at(d: &mut impl DstrStore, idx: usize, src: &[u8]) {
    let len = d.len();
    let tail = d.bytes()[idx..].to_vec();
    dstr_ensure_capacity(d, len + src.len() + 1);
    write_cstr(d, idx + src.len(), &tail);
    d.write(idx, src);
    d.set_len(len + src.len());
}

/// `dstr_insert`. `src` is the C string (`None` for NULL).
///
/// Intentional difference: `idx > len` does nothing (C moves memory from
/// before or past the buffer).
pub fn dstr_insert(d: &mut impl DstrStore, idx: usize, src: Option<&[u8]>) {
    let Some(src) = src.filter(|s| !s.is_empty()) else {
        return;
    };
    if idx == d.len() {
        dstr_cat(d, src);
    } else if idx < d.len() {
        insert_at(d, idx, src);
    }
}

/// `dstr_insert_dstr`: `src` is the source dstr's string. `idx > len` does
/// nothing, as for [`dstr_insert`].
pub fn dstr_insert_dstr(d: &mut impl DstrStore, idx: usize, src: &[u8]) {
    if src.is_empty() {
        return;
    }
    if idx == d.len() {
        dstr_cat_dstr(d, src);
    } else if idx < d.len() {
        insert_at(d, idx, src);
    }
}

/// `dstr_insert_ch`. Intentional difference: C moves one byte more than
/// needed, writing `array[len + 2]`, past the buffer when `capacity` is
/// exactly `len + 2`; that byte is past the string's NUL and is not
/// written here. `idx > len` does nothing.
pub fn dstr_insert_ch(d: &mut impl DstrStore, idx: usize, c: u8) {
    let len = d.len();
    if idx > len {
        return;
    }
    let tail = d.bytes()[idx..].to_vec();
    dstr_ensure_capacity(d, len + 2);
    if idx == len {
        // dstr_cat_ch
        write_cstr(d, len, &[c]);
    } else {
        write_cstr(d, idx + 1, &tail);
        d.write(idx, &[c]);
    }
    d.set_len(len + 1);
}

/// `dstr_remove`. Characterized, not endorsed: removing `count == len`
/// frees the dstr whatever `idx` is. Intentional difference:
/// `idx + count > len` does nothing (C moves memory past the string).
pub fn dstr_remove(d: &mut impl DstrStore, idx: usize, count: usize) {
    let len = d.len();
    if count == 0 {
        return;
    }
    if count == len {
        d.free();
        return;
    }
    let Some(end) = idx.checked_add(count).filter(|&e| e <= len) else {
        return;
    };
    if end == len {
        d.write(idx, &[0]);
    } else {
        let tail = d.bytes()[end..].to_vec();
        write_cstr(d, idx, &tail);
    }
    d.set_len(len - count);
}

/// `strstr` on the C string in `buf` starting at `from`.
fn strstr(buf: &[u8], from: usize, find: &[u8]) -> Option<usize> {
    let end = from + cstrlen(buf, from);
    buf[from..end]
        .windows(find.len())
        .position(|w| w == find)
        .map(|p| from + p)
}

/// `dstr_replace`: replaces every `find` with `replace` (`None` is empty)
/// in the C string, left to right.
///
/// Intentional difference: an empty `find` does nothing (C loops forever).
pub fn dstr_replace(d: &mut impl DstrStore, find: &[u8], replace: Option<&[u8]>) {
    if dstr_is_empty(d) || find.is_empty() {
        return;
    }
    let rep = replace.unwrap_or(&[]);
    let (find_len, rep_len) = (find.len(), rep.len());

    // Work on a copy of the string and its NUL, as C works in place.
    let mut buf = d.bytes().to_vec();
    buf.push(0);
    let mut count: usize = 0;
    let mut temp = 0;

    if rep_len < find_len {
        while let Some(pos) = strstr(&buf, temp, find) {
            let end = pos + find_len;
            let end_len = cstrlen(&buf, end);
            buf.copy_within(end..end + end_len + 1, pos + rep_len);
            buf[pos..pos + rep_len].copy_from_slice(rep);
            temp = pos + rep_len;
            count += 1;
        }
        if count != 0 {
            let len = d
                .len()
                .wrapping_add(rep_len.wrapping_sub(find_len).wrapping_mul(count));
            d.write(0, &buf);
            d.set_len(len);
        }
    } else if rep_len > find_len {
        while let Some(pos) = strstr(&buf, temp, find) {
            temp = pos + find_len;
            count += 1;
        }
        if count == 0 {
            return;
        }
        let len = d.len() + (rep_len - find_len) * count;
        d.set_len(len);
        dstr_ensure_capacity(d, len + 1);

        buf.resize(buf.len().max(len + 1), 0);
        temp = 0;
        while let Some(pos) = strstr(&buf, temp, find) {
            let end = pos + find_len;
            let end_len = cstrlen(&buf, end);
            buf.copy_within(end..end + end_len + 1, pos + rep_len);
            buf[pos..pos + rep_len].copy_from_slice(rep);
            temp = pos + rep_len;
        }
        let written = cstrlen(&buf, 0) + 1;
        d.write(0, &buf[..written]);
    } else {
        while let Some(pos) = strstr(&buf, temp, find) {
            buf[pos..pos + rep_len].copy_from_slice(rep);
            temp = pos + rep_len;
            count += 1;
        }
        if count != 0 {
            d.write(0, &buf);
        }
    }
}

/// `dstr_safe_printf`: copies `format`, then replaces `$1`..`$4` with each
/// value that is not `None`.
pub fn dstr_safe_printf(d: &mut impl DstrStore, format: Option<&[u8]>, values: [Option<&[u8]>; 4]) {
    dstr_copy(d, format);
    for (i, value) in values.iter().enumerate() {
        if let Some(v) = value {
            let tag = [b'$', b'1' + i as u8];
            dstr_replace(d, &tag, Some(v));
        }
    }
}

/// `dstr_depad`: [`strdepad`] in place; an all-padding string frees `d`.
pub fn dstr_depad(d: &mut impl DstrStore) {
    if d.is_null() {
        return;
    }
    let mut buf = d.bytes().to_vec();
    buf.push(0);
    strdepad(&mut buf);
    if buf[0] != 0 {
        d.write(0, &buf);
        d.set_len(cstrlen(&buf, 0));
    } else {
        d.free();
    }
}

/// `dstr_resize` from `dstr.h`.
fn dstr_resize(d: &mut impl DstrStore, num: usize) {
    if num == 0 {
        d.free();
        return;
    }
    dstr_ensure_capacity(d, num + 1);
    d.write(num, &[0]);
    d.set_len(num);
}

/// `dstr_left`: the first `pos` bytes of `src` (the source dstr's string;
/// past its end read as NUL). `same` is `dst == str`, which truncates in
/// place.
pub fn dstr_left(d: &mut impl DstrStore, src: &[u8], same: bool, pos: usize) {
    dstr_resize(d, pos);
    if !same && pos != 0 {
        d.write(0, &take_padded(src, pos));
    }
}

/// `dstr_mid`: `count` bytes of `src` from `start` (past its end read as
/// NUL), in a new allocation of exactly `count + 1`.
pub fn dstr_mid(d: &mut impl DstrStore, src: &[u8], start: usize, count: usize) {
    dstr_ncopy(d, src.get(start..).unwrap_or(&[]), count);
}

/// `dstr_right`: `src` from `pos` on. Intentional difference: `pos > len`
/// leaves `d` empty (C copies a wrapped-around length).
pub fn dstr_right(d: &mut impl DstrStore, src: &[u8], pos: usize) {
    d.free();
    let tail = src.get(pos..).unwrap_or(&[]);
    if !tail.is_empty() {
        dstr_ensure_capacity(d, tail.len() + 1);
        write_cstr(d, 0, tail);
        d.set_len(tail.len());
    }
}
