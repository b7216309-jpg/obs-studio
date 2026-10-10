//! Safe counterpart of `struct plane`. `libobs/graphics/plane.c` is not
//! ported yet; this holds only what the vec3 port needs.

use crate::vec3::Vec3;

/// A plane `dot(dir, v) == dist`, the safe counterpart of `struct plane`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Plane {
    pub dir: Vec3,
    pub dist: f32,
}

impl Plane {
    #[must_use]
    pub const fn new(dir: Vec3, dist: f32) -> Self {
        Self { dir, dist }
    }
}
