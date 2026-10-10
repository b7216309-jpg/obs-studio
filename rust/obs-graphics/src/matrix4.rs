//! Safe counterpart of `struct matrix4`. `libobs/graphics/matrix4.c` is not
//! ported yet; this holds only what the vec4 port needs.

use crate::vec4::Vec4;

/// A 4x4 matrix stored as four rows, the safe counterpart of
/// `struct matrix4`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Matrix4 {
    pub x: Vec4,
    pub y: Vec4,
    pub z: Vec4,
    pub t: Vec4,
}

impl Matrix4 {
    #[must_use]
    pub const fn new(x: Vec4, y: Vec4, z: Vec4, t: Vec4) -> Self {
        Self { x, y, z, t }
    }

    /// Swaps rows and columns, as `matrix4_transpose` does. Only moves
    /// values, so it is exact.
    #[must_use]
    pub const fn transpose(&self) -> Self {
        todo!()
    }
}
