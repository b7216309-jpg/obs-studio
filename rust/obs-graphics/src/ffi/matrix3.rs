//! `struct matrix3` from `libobs/graphics/matrix3.h`. `matrix3.c` is not
//! ported yet; the type is here for the vec3 shims.

use super::vec3::vec3;
use crate::matrix3::Matrix3;

/// `struct matrix3`: three axes and a translation, size 64, align 16.
#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct matrix3 {
    pub x: vec3,
    pub y: vec3,
    pub z: vec3,
    pub t: vec3,
}

impl From<matrix3> for Matrix3 {
    fn from(m: matrix3) -> Self {
        Self::new(m.x.into(), m.y.into(), m.z.into(), m.t.into())
    }
}
