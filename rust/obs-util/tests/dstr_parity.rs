//! Tier 3: the Rust C ABI shim (and through it the safe core) behaves
//! exactly like the original `util/dstr.c`, compiled as an oracle.
//!
//! The `dstr` functions are checked model-style: a random sequence of
//! operations runs on a dstr through the shim and on another through the
//! oracle, and after every operation both must agree on `array` being
//! NULL, the C string, `len` and `capacity`. Operation arguments stay
//! within what C defines; the intentional differences (out-of-range
//! indexes, empty or NULL `find`, sources aliasing the destination where C
//! reads moved memory, `insert_ch`'s out-of-bounds write, reading past a
//! source) are excluded here and covered in `dstr.rs`.
//!
//! Reminder: the mutation check (break the core, see this file fail) is done
//! once per port, not on every change.

use core::ffi::{CStr, c_char, c_void};
use core::ptr;

use obs_c_oracle::dstr::{self as c, OracleDstr};
use obs_c_oracle::lexer::OracleStrref;
use obs_util::ffi::darray::bfree;
use obs_util::ffi::dstr::{self as rs, dstr};
use obs_util::ffi::lexer::strref;
use proptest::prelude::*;

/// Short strings over a small alphabet, so finds and placeholders match.
fn text() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(prop::sample::select(&b"ab -$1234\t\r\nAB\xe9"[..]), 0..10)
}

/// Bytes for `dstr_ncat`, which copies `len` raw bytes: NUL included, so a
/// NUL first byte and embedded NULs are reached.
fn raw() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(prop::sample::select(&b"ab \0$1"[..]), 0..8)
}

fn cz(v: &[u8]) -> Vec<u8> {
    let mut v = v.to_vec();
    v.push(0);
    v
}

fn p(v: &Option<Vec<u8>>) -> *const c_char {
    v.as_ref().map_or(ptr::null(), |v| v.as_ptr().cast())
}

#[derive(Clone, Debug)]
enum Op {
    Copy(Option<Vec<u8>>),
    Ncopy(Vec<u8>, usize),
    NcopyDstr(usize),
    CopyStrref(Vec<u8>, usize),
    InitCopyStrref(Vec<u8>, usize),
    CatDstr { self_src: bool },
    CatStrref(Vec<u8>, usize),
    Ncat(Option<Vec<u8>>, usize),
    NcatDstr(usize),
    Insert(usize, Option<Vec<u8>>),
    InsertDstr(usize),
    InsertCh(usize, u8),
    Remove(usize, usize),
    Replace(Vec<u8>, Option<Vec<u8>>),
    SafePrintf(Option<Vec<u8>>, [Option<Vec<u8>>; 4]),
    Depad,
    Left { self_src: bool, pos: usize },
    Mid(usize, usize),
    Right(usize),
    SetOther(Vec<u8>),
}

/// Raw choices; indexes are fitted to the current state when applied.
fn op() -> impl Strategy<Value = Op> {
    let opt = || prop::option::weighted(0.9, text());
    let idx = || any::<usize>();
    prop_oneof![
        opt().prop_map(Op::Copy),
        (text(), idx()).prop_map(|(s, n)| Op::Ncopy(s, n)),
        idx().prop_map(Op::NcopyDstr),
        (text(), idx()).prop_map(|(s, n)| Op::CopyStrref(s, n)),
        (text(), idx()).prop_map(|(s, n)| Op::InitCopyStrref(s, n)),
        any::<bool>().prop_map(|self_src| Op::CatDstr { self_src }),
        (text(), idx()).prop_map(|(s, n)| Op::CatStrref(s, n)),
        (prop::option::weighted(0.9, raw()), idx()).prop_map(|(s, n)| Op::Ncat(s, n)),
        idx().prop_map(Op::NcatDstr),
        (idx(), opt()).prop_map(|(i, s)| Op::Insert(i, s)),
        idx().prop_map(Op::InsertDstr),
        (idx(), any::<u8>()).prop_map(|(i, c)| Op::InsertCh(i, c)),
        (idx(), idx()).prop_map(|(i, n)| Op::Remove(i, n)),
        (
            prop::collection::vec(prop::sample::select(&b"ab$1 "[..]), 1..3),
            prop::option::weighted(
                0.9,
                prop::collection::vec(prop::sample::select(&b"ab$1 X"[..]), 0..4)
            ),
        )
            .prop_map(|(f, r)| Op::Replace(f, r)),
        (opt(), [opt(), opt(), opt(), opt()]).prop_map(|(f, v)| Op::SafePrintf(f, v)),
        Just(Op::Depad),
        (any::<bool>(), idx()).prop_map(|(self_src, pos)| Op::Left { self_src, pos }),
        (idx(), idx()).prop_map(|(s, n)| Op::Mid(s, n)),
        idx().prop_map(Op::Right),
        text().prop_map(Op::SetOther),
    ]
}

/// What both sides must agree on.
fn state(array: *const c_char, len: usize, capacity: usize) -> (Option<Vec<u8>>, usize, usize) {
    // SAFETY: a valid dstr's array is NULL or a NUL-terminated buffer; the
    // C string is read up to `len` at most.
    let s = (!array.is_null()).then(|| unsafe {
        let mut n = 0;
        while n < len && *array.add(n) != 0 {
            n += 1;
        }
        core::slice::from_raw_parts(array.cast::<u8>(), n).to_vec()
    });
    (s, len, capacity)
}

struct Pair {
    r: dstr,
    o: OracleDstr,
}

impl Pair {
    fn new() -> Self {
        Pair {
            r: dstr {
                array: ptr::null_mut(),
                len: 0,
                capacity: 0,
            },
            o: OracleDstr {
                array: ptr::null_mut(),
                len: 0,
                capacity: 0,
            },
        }
    }

    fn check(&self) -> Result<(), TestCaseError> {
        prop_assert_eq!(
            state(self.r.array, self.r.len, self.r.capacity),
            state(self.o.array, self.o.len, self.o.capacity)
        );
        Ok(())
    }

    fn free(&mut self) {
        // SAFETY: both arrays are NULL or bmalloc buffers owned here.
        unsafe {
            bfree(self.r.array.cast::<c_void>());
            bfree(self.o.array.cast::<c_void>());
        }
        *self = Pair::new();
    }
}

/// Applies `op` to `d` on both sides, fitting indexes to the current state
/// and skipping arguments outside C's defined behavior.
fn apply(d: &mut Pair, other: &mut Pair, op: &Op) {
    let len = d.r.len;
    let olen = other.r.len;
    // SAFETY: every pointer passed is NULL or valid for what each function
    // reads; dstrs are valid; NUL-terminated buffers outlive the calls.
    unsafe {
        match op {
            Op::Copy(s) => {
                let s = s.as_deref().map(cz);
                rs::dstr_copy(&mut d.r, p(&s));
                c::oracle_dstr_copy(&mut d.o, p(&s));
            }
            Op::Ncopy(s, n) => {
                let n = n % (s.len() + 1);
                let b = cz(s);
                rs::dstr_ncopy(&mut d.r, b.as_ptr().cast(), n);
                c::oracle_dstr_ncopy(&mut d.o, b.as_ptr().cast(), n);
            }
            Op::NcopyDstr(n) => {
                // C copies from a NULL array when the source is empty.
                let n = if olen == 0 { 0 } else { n % (olen + 3) };
                rs::dstr_ncopy_dstr(&mut d.r, &other.r, n);
                c::oracle_dstr_ncopy_dstr(&mut d.o, &other.o, n);
            }
            Op::CopyStrref(s, n) | Op::InitCopyStrref(s, n) => {
                let n = n % (s.len() + 1);
                let b = cz(s);
                let rr = strref {
                    array: b.as_ptr().cast(),
                    len: n,
                };
                let or = OracleStrref {
                    array: b.as_ptr().cast(),
                    len: n,
                };
                if matches!(op, Op::InitCopyStrref(..)) {
                    d.free(); // init forgets the old array; free it first
                    rs::dstr_init_copy_strref(&mut d.r, &rr);
                    c::oracle_dstr_init_copy_strref(&mut d.o, &or);
                } else {
                    rs::dstr_copy_strref(&mut d.r, &rr);
                    c::oracle_dstr_copy_strref(&mut d.o, &or);
                }
            }
            Op::CatDstr { self_src } => {
                if *self_src {
                    rs::dstr_cat_dstr(&mut d.r, ptr::addr_of!(d.r));
                    c::oracle_dstr_cat_dstr(&mut d.o, ptr::addr_of!(d.o));
                } else {
                    rs::dstr_cat_dstr(&mut d.r, &other.r);
                    c::oracle_dstr_cat_dstr(&mut d.o, &other.o);
                }
            }
            Op::CatStrref(s, n) => {
                let n = n % (s.len() + 1);
                let b = cz(s);
                let rr = strref {
                    array: b.as_ptr().cast(),
                    len: n,
                };
                let or = OracleStrref {
                    array: b.as_ptr().cast(),
                    len: n,
                };
                rs::dstr_cat_strref(&mut d.r, &rr);
                c::oracle_dstr_cat_strref(&mut d.o, &or);
            }
            Op::Ncat(s, n) => {
                // up to the whole buffer, its NUL included
                let n = n % (s.as_ref().map_or(0, Vec::len) + 2);
                let s = s.as_deref().map(cz);
                rs::dstr_ncat(&mut d.r, p(&s), n);
                c::oracle_dstr_ncat(&mut d.o, p(&s), n);
            }
            Op::NcatDstr(n) => {
                let n = n % (olen + 3);
                rs::dstr_ncat_dstr(&mut d.r, &other.r, n);
                c::oracle_dstr_ncat_dstr(&mut d.o, &other.o, n);
            }
            Op::Insert(i, s) => {
                let i = i % (len + 1);
                let s = s.as_deref().map(cz);
                rs::dstr_insert(&mut d.r, i, p(&s));
                c::oracle_dstr_insert(&mut d.o, i, p(&s));
            }
            Op::InsertDstr(i) => {
                let i = i % (len + 1);
                rs::dstr_insert_dstr(&mut d.r, i, &other.r);
                c::oracle_dstr_insert_dstr(&mut d.o, i, &other.o);
            }
            Op::InsertCh(i, ch) => {
                let i = i % (len + 1);
                // C writes array[len + 2] mid-string: skip when that is past
                // the buffer it will have.
                let cap = d.r.capacity;
                let new_cap = if len + 2 <= cap && !d.r.array.is_null() {
                    cap
                } else {
                    (if cap == 0 { len + 2 } else { cap * 2 }).max(len + 2)
                };
                if i < len && new_cap <= len + 2 {
                    return;
                }
                rs::dstr_insert_ch(&mut d.r, i, *ch as c_char);
                c::oracle_dstr_insert_ch(&mut d.o, i, *ch as c_char);
            }
            Op::Remove(i, n) => {
                let n = n % (len + 1);
                let i = if n == len { *i % 4 } else { i % (len - n + 1) };
                rs::dstr_remove(&mut d.r, i, n);
                c::oracle_dstr_remove(&mut d.o, i, n);
            }
            Op::Replace(f, r) => {
                let f = cz(f);
                let r = r.as_deref().map(cz);
                rs::dstr_replace(&mut d.r, f.as_ptr().cast(), p(&r));
                c::oracle_dstr_replace(&mut d.o, f.as_ptr().cast(), p(&r));
            }
            Op::SafePrintf(f, v) => {
                let f = f.as_deref().map(cz);
                let v: Vec<_> = v.iter().map(|x| x.as_deref().map(cz)).collect();
                rs::dstr_safe_printf(&mut d.r, p(&f), p(&v[0]), p(&v[1]), p(&v[2]), p(&v[3]));
                c::oracle_dstr_safe_printf(&mut d.o, p(&f), p(&v[0]), p(&v[1]), p(&v[2]), p(&v[3]));
            }
            Op::Depad => {
                rs::dstr_depad(&mut d.r);
                c::oracle_dstr_depad(&mut d.o);
            }
            Op::Left { self_src, pos } => {
                if *self_src {
                    let pos = pos % (len + 1);
                    rs::dstr_left(&mut d.r, ptr::addr_of!(d.r), pos);
                    c::oracle_dstr_left(&mut d.o, ptr::addr_of!(d.o), pos);
                } else {
                    let pos = pos % (olen + 1);
                    rs::dstr_left(&mut d.r, &other.r, pos);
                    c::oracle_dstr_left(&mut d.o, &other.o, pos);
                }
            }
            Op::Mid(s, n) => {
                let s = s % (olen + 1);
                let n = if olen == 0 { 0 } else { n % (olen - s + 1) };
                rs::dstr_mid(&mut d.r, &other.r, s, n);
                c::oracle_dstr_mid(&mut d.o, &other.o, s, n);
            }
            Op::Right(pos) => {
                let pos = pos % (olen + 1);
                rs::dstr_right(&mut d.r, &other.r, pos);
                c::oracle_dstr_right(&mut d.o, &other.o, pos);
            }
            Op::SetOther(s) => {
                let s = cz(s);
                rs::dstr_copy(&mut other.r, s.as_ptr().cast());
                c::oracle_dstr_copy(&mut other.o, s.as_ptr().cast());
            }
        }
    }
}

fn cstring() -> impl Strategy<Value = Option<Vec<u8>>> {
    prop::option::weighted(0.95, text().prop_map(|v| cz(&v)))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn dstr_ops_match_c(start in text(), ops in prop::collection::vec(op(), 1..24)) {
        let mut d = Pair::new();
        let mut other = Pair::new();
        apply(&mut other, &mut Pair::new(), &Op::Copy(Some(start)));
        for op in &ops {
            apply(&mut d, &mut other, op);
            d.check()?;
            other.check()?;
        }
        d.free();
        other.free();
    }

    #[test]
    fn compares_match_c(a in cstring(), b in cstring(), n in 0usize..12) {
        // SAFETY: NULL or NUL-terminated strings.
        unsafe {
            prop_assert_eq!(rs::astrcmpi(p(&a), p(&b)), c::oracle_astrcmpi(p(&a), p(&b)));
            prop_assert_eq!(rs::astrcmp_n(p(&a), p(&b), n), c::oracle_astrcmp_n(p(&a), p(&b), n));
            prop_assert_eq!(rs::astrcmpi_n(p(&a), p(&b), n), c::oracle_astrcmpi_n(p(&a), p(&b), n));
            let (x, y) = (rs::astrstri(p(&a), p(&b)), c::oracle_astrstri(p(&a), p(&b)));
            prop_assert_eq!(x, y);
        }
    }

    #[test]
    fn strdepad_matches_c(s in text()) {
        let (mut x, mut y) = (cz(&s), cz(&s));
        // SAFETY: writable NUL-terminated buffers.
        unsafe {
            let rx = rs::strdepad(x.as_mut_ptr().cast());
            let ry = c::oracle_strdepad(y.as_mut_ptr().cast());
            prop_assert_eq!(rx.cast::<u8>(), x.as_mut_ptr());
            prop_assert_eq!(ry.cast::<u8>(), y.as_mut_ptr());
        }
        let cut = |v: &[u8]| v[..v.iter().position(|&b| b == 0).unwrap()].to_vec();
        prop_assert_eq!(cut(&x), cut(&y));
    }

    #[test]
    fn strlist_split_matches_c(s in cstring(), ch in prop::sample::select(&b", a\t$\xe9"[..]), inc in any::<bool>()) {
        let list = |l: *mut *mut c_char| -> Option<Vec<Vec<u8>>> {
            // SAFETY: NULL or a NULL-terminated table of C strings.
            (!l.is_null()).then(|| unsafe {
                let mut v = Vec::new();
                let mut i = 0;
                while !(*l.add(i)).is_null() {
                    v.push(CStr::from_ptr(*l.add(i)).to_bytes().to_vec());
                    i += 1;
                }
                v
            })
        };
        // SAFETY: NULL or a NUL-terminated string.
        unsafe {
            let x = rs::strlist_split(p(&s), ch as c_char, inc);
            let y = c::oracle_strlist_split(p(&s), ch as c_char, inc);
            prop_assert_eq!(list(x), list(y));
            rs::strlist_free(x);
            c::oracle_strlist_free(y);
        }
    }
}

#[test]
fn null_arguments() {
    // SAFETY: NULL is handled first by both.
    unsafe {
        assert!(rs::astrstri(ptr::null(), c"a".as_ptr()).is_null());
        assert!(rs::astrstri(c"a".as_ptr(), ptr::null()).is_null());
        assert!(rs::strdepad(ptr::null_mut()).is_null());
        assert!(rs::strlist_split(ptr::null(), b',' as c_char, true).is_null());
        assert!(c::oracle_strlist_split(ptr::null(), b',' as c_char, true).is_null());
        assert_eq!(rs::astrcmpi(ptr::null(), c"".as_ptr()), 0);
        assert_eq!(rs::astrcmp_n(ptr::null(), c"a".as_ptr(), 1), -1);
    }
}
