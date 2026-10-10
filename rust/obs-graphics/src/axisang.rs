//! Safe core of `libobs/graphics/axisang.c`.
//!
//! The arithmetic follows the C expression order, `(x*x + y*y) + z*z`, and
//! `acos` is the C library's `acosf`, so results are bit-identical to the C
//! code.

use crate::quat::Quat;

/// `EPSILON` from `math-defs.h`.
const EPSILON: f32 = 1e-4;

/// An axis (`x`, `y`, `z`) and an angle in radians (`w`), the safe
/// counterpart of `struct axisang`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AxisAng {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl AxisAng {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    /// The rotation of the unit quaternion `q` as a normalized axis and an
    /// angle (`axisang_from_quat`).
    ///
    /// When the vector part is within `EPSILON` of zero (no rotation), the
    /// result is all zeros. A NaN length is not close to zero, as in C, so
    /// it takes the normalizing branch and yields NaNs.
    #[must_use]
    pub fn from_quat(q: Quat) -> Self {
        let len = q.x * q.x + q.y * q.y + q.z * q.z;
        // close_float(len, 0.0f, EPSILON)
        if (len - 0.0).abs() <= EPSILON {
            return Self::default();
        }
        let leni = 1.0 / len.sqrt();
        Self::new(q.x * leni, q.y * leni, q.z * leni, q.w.acos() * 2.0)
    }
}
