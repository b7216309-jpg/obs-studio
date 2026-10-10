//! Safe core of `libobs/graphics/vec4.c`.
//!
//! `vec4_dot` in `vec4.h` sums the four products through SSE shuffles, as
//! `(m3 + m1) + (m2 + m0)`. [`Vec4::dot`] adds them in that order, so
//! results are bit-identical to the C code.

use crate::matrix4::Matrix4;

/// A 4D vector of `f32`, the safe counterpart of `struct vec4`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// A point from 3D coordinates, with `w` set to 1 (`vec4_from_vec3`).
    #[must_use]
    pub const fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self::new(x, y, z, 1.0)
    }

    /// Dot product, summed in the order of the SSE `vec4_dot`.
    #[must_use]
    pub fn dot(self, other: Self) -> f32 {
        let m0 = self.x * other.x;
        let m1 = self.y * other.y;
        let m2 = self.z * other.z;
        let m3 = self.w * other.w;
        (m3 + m1) + (m2 + m0)
    }

    /// Row vector times matrix (`vec4_transform`): each component is the
    /// dot product of `self` with a column of `m`.
    #[must_use]
    pub fn transform(self, m: &Matrix4) -> Self {
        let cols = m.transpose();
        Self::new(
            cols.x.dot(self),
            cols.y.dot(self),
            cols.z.dot(self),
            cols.t.dot(self),
        )
    }
}
