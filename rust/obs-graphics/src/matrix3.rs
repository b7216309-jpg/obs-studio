//! Safe counterpart of `struct matrix3`. `libobs/graphics/matrix3.c` is not
//! ported yet; this holds only what the vec3 port needs.

use crate::vec3::Vec3;

/// Three axes and a translation, the safe counterpart of `struct matrix3`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Matrix3 {
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
    pub t: Vec3,
}

impl Matrix3 {
    #[must_use]
    pub const fn new(x: Vec3, y: Vec3, z: Vec3, t: Vec3) -> Self {
        Self { x, y, z, t }
    }
}
