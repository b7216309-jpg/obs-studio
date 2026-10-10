//! C ABI shims for the exported functions in `libobs/graphics/vec4.h`.
//!
//! The shims only convert to the safe core in [`crate::vec4`]. Each reads
//! all of its inputs before writing `dst`, so C callers may alias `dst` and
//! an input, as `vec3_transform` does.

use core::ptr;

use super::matrix4::matrix4;
use super::vec3::vec3;
use crate::vec4::Vec4;

/// `struct vec4`. The C union with `float ptr[4]` and `__m128 m` has the
/// same layout: size 16, align 16, `x`/`y`/`z`/`w` at 0/4/8/12, `ptr` and
/// `m` at 0.
#[repr(C, align(16))]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<vec4> for Vec4 {
    fn from(v: vec4) -> Self {
        Self::new(v.x, v.y, v.z, v.w)
    }
}

impl From<Vec4> for vec4 {
    fn from(v: Vec4) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: v.w,
        }
    }
}

/// Writes `*v` with `w` set to 1 to `*dst`.
///
/// # Safety
///
/// `v` must be valid for reads of a `vec3` and `dst` valid for writes of a
/// `vec4`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec4_from_vec3(dst: *mut vec4, v: *const vec3) {
    // SAFETY: the caller guarantees `v` is readable; read before writing `dst`.
    let src = unsafe { ptr::read(v) };
    // SAFETY: the caller guarantees `dst` is writable.
    unsafe { ptr::write(dst, Vec4::from_xyz(src.x, src.y, src.z).into()) };
}

/// Writes `*v` transformed by `*m` (row vector times matrix) to `*dst`.
///
/// # Safety
///
/// `v` must be valid for reads of a `vec4`, `m` for reads of a `matrix4`,
/// and `dst` valid for writes of a `vec4`. `dst` may equal `v`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn vec4_transform(dst: *mut vec4, v: *const vec4, m: *const matrix4) {
    // SAFETY: the caller guarantees `v` and `m` are readable; read both
    // before writing `dst`.
    let (src, mat) = unsafe { (ptr::read(v), ptr::read(m)) };
    // SAFETY: the caller guarantees `dst` is writable.
    unsafe { ptr::write(dst, Vec4::from(src).transform(&mat.into()).into()) };
}
