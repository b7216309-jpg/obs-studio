//! `struct plane` from `libobs/graphics/plane.h`. `plane.c` is not ported
//! yet; the type is here for the vec3 shims.

use super::vec3::vec3;
use crate::plane::Plane;

/// `struct plane`: a `vec3` direction and a distance, size 32, align 16.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct plane {
    pub dir: vec3,
    pub dist: f32,
}

impl From<plane> for Plane {
    fn from(p: plane) -> Self {
        Self::new(p.dir.into(), p.dist)
    }
}
