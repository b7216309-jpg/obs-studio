//! `struct matrix4` from `libobs/graphics/matrix4.h`. `matrix4.c` is not
//! ported yet; the type is here for the vec4 shims.

use super::vec4::vec4;
use crate::matrix4::Matrix4;

/// `struct matrix4`: four `vec4` rows, size 64, align 16.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct matrix4 {
    pub x: vec4,
    pub y: vec4,
    pub z: vec4,
    pub t: vec4,
}

impl From<matrix4> for Matrix4 {
    fn from(m: matrix4) -> Self {
        Self::new(m.x.into(), m.y.into(), m.z.into(), m.t.into())
    }
}
