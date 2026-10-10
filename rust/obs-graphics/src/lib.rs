//! Rust ports of `libobs/graphics`.
//!
//! Safe cores live at the crate root; the C ABI shims that replace the
//! original `EXPORT` symbols live in [`ffi`]. See
//! `docs/rust-port/testing-policy.md`.

#[cfg(test)]
use obs_c_oracle as _; // links the test bmalloc/bfree (oracle/test_bmem.c)

pub mod axisang;
pub mod ffi;
pub mod matrix3;
pub mod matrix4;
pub mod plane;
pub mod quat;
pub mod vec2;
pub mod vec3;
pub mod vec4;
