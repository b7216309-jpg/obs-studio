//! Safe counterpart of `struct quat`. `libobs/graphics/quat.c` is not
//! ported yet; this holds only what the axisang port needs.

/// A quaternion with vector part (`x`, `y`, `z`) and scalar part `w`, the
/// safe counterpart of `struct quat`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}
