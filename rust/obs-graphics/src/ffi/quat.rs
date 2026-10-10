//! `struct quat` from `libobs/graphics/quat.h`. `quat.c` is not ported yet;
//! the type is here for the axisang shim.

use crate::quat::Quat;

/// `struct quat`: a union with `__m128`, so size 16, align 16.
#[repr(C, align(16))]
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl From<quat> for Quat {
    fn from(q: quat) -> Self {
        Self::new(q.x, q.y, q.z, q.w)
    }
}
