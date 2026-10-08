//! Aggregates every Rust port's C ABI shim into one staticlib for libobs.
//!
//! libobs must link exactly one Rust staticlib: each staticlib carries its
//! own copy of `std`, so linking two would duplicate symbols.

pub use obs_util::ffi;
