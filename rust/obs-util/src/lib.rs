//! Rust ports of `libobs/util`.
//!
//! Safe cores live at the crate root; the C ABI shims that replace the
//! original `EXPORT` symbols live in [`ffi`]. See
//! `docs/rust-port/testing-policy.md`.

#[cfg(test)]
use obs_c_oracle as _; // links the test bmalloc/bfree (oracle/test_bmem.c)

pub mod array_serializer;
pub mod base;
pub mod bitstream;
pub mod crc32;
pub mod darray;
pub mod ffi;
pub mod file_serializer;
pub mod path_extension;
pub mod task;
