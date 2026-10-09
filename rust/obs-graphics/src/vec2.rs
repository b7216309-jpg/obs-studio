//! Safe core of `libobs/graphics/vec2.c`.
//!
//! Every method performs the same `f32` operations in the same order as the
//! C code, so results are bit-identical.

/// A 2D vector of `f32`, the safe counterpart of `struct vec2`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Per-component absolute value (`vec2_abs`).
    #[must_use]
    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    /// Per-component floor (`vec2_floor`).
    #[must_use]
    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }

    /// Per-component ceiling (`vec2_ceil`).
    #[must_use]
    pub fn ceil(self) -> Self {
        Self::new(self.x.ceil(), self.y.ceil())
    }

    /// Euclidean length, `sqrtf(x*x + y*y)` (`vec2_len`).
    #[must_use]
    #[allow(clippy::len_without_is_empty)]
    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// True when every component differs by at most `epsilon`
    /// (`vec2_close`). NaN components are never close.
    #[must_use]
    pub fn close(self, other: Self, epsilon: f32) -> bool {
        (self.x - other.x).abs() <= epsilon && (self.y - other.y).abs() <= epsilon
    }

    /// Normalizes the vector (`vec2_norm`).
    ///
    /// Returns `None` when the length is zero or NaN. That maps to the C
    /// behaviour of leaving `dst` unchanged.
    #[must_use]
    pub fn norm(self) -> Option<Self> {
        let len = self.len();
        if len > 0.0 {
            let inv = 1.0 / len;
            Some(Self::new(self.x * inv, self.y * inv))
        } else {
            None
        }
    }
}
