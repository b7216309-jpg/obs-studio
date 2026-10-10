//! Safe core of `libobs/graphics/vec3.c`.
//!
//! `struct vec3` is a union with `__m128`, so it has a fourth lane `w`.
//! The C code keeps `w` at 0 in its results, but `vec3_dot` multiplies all
//! four lanes, so a nonzero or NaN `w` coming from a caller changes the
//! result. [`Vec3`] keeps `w` for that reason, and [`Vec3::dot`] sums the
//! four products in the SSE order, `(m3 + m1) + (m2 + m0)`, so results are
//! bit-identical to the C code.

use crate::matrix3::Matrix3;
use crate::matrix4::Matrix4;
use crate::plane::Plane;
use crate::vec4::Vec4;

/// A 3D vector of `f32` plus the SSE padding lane `w`, the safe
/// counterpart of `struct vec3`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec3 {
    /// A vector with `w` at 0, as `vec3_set` builds it.
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, w: 0.0 }
    }

    /// All four lanes, including a caller-provided `w`.
    #[must_use]
    pub const fn with_w(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// `x`, `y`, `z` of `v`, with `w` cleared (`vec3_from_vec4`).
    #[must_use]
    pub const fn from_vec4(v: Vec4) -> Self {
        let _ = v;
        todo!()
    }

    /// Dot product over all four lanes, summed in the order of the SSE
    /// `vec3_dot`.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        let _ = (self, other);
        todo!()
    }

    /// Lane-wise difference with `w` cleared (`vec3_sub`).
    #[must_use]
    pub fn sub_xyz(self, other: Self) -> Self {
        let _ = (self, other);
        todo!()
    }

    /// Every lane times `f`, `w` included (`vec3_mulf`).
    #[must_use]
    pub fn mulf(self, f: f32) -> Self {
        let _ = (self, f);
        todo!()
    }

    /// Signed distance from `p` (`vec3_plane_dist`).
    #[must_use]
    pub fn plane_dist(self, p: &Plane) -> f32 {
        let _ = (self, p);
        todo!()
    }

    /// Dot product with each axis of `m`; `t` is ignored (`vec3_rotate`).
    #[must_use]
    pub fn rotate(self, m: &Matrix3) -> Self {
        let _ = (self, m);
        todo!()
    }

    /// As a point (`w` = 1) through `m`, then back to 3D (`vec3_transform`).
    #[must_use]
    pub fn transform(self, m: &Matrix4) -> Self {
        let _ = (self, m);
        todo!()
    }

    /// Subtract `m.t`, then rotate by the axes of `m` (`vec3_transform3x4`).
    #[must_use]
    pub fn transform3x4(self, m: &Matrix3) -> Self {
        let _ = (self, m);
        todo!()
    }

    /// Reflection through the plane `p` (`vec3_mirror`).
    #[must_use]
    pub fn mirror(self, p: &Plane) -> Self {
        let _ = (self, p);
        todo!()
    }

    /// Reflection against the direction `vec` (`vec3_mirrorv`).
    #[must_use]
    pub fn mirrorv(self, vec: Self) -> Self {
        let _ = (self, vec);
        todo!()
    }

    /// `x`, `y`, `z` from three calls to `next`, in that order
    /// (`vec3_rand`, where `next` is libobs `rand_float`).
    #[must_use]
    pub fn rand(next: impl FnMut() -> f32) -> Self {
        let _ = next;
        todo!()
    }
}
