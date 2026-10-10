//! Aggregates every Rust port's C ABI shim into one staticlib for libobs.
//!
//! libobs must link exactly one Rust staticlib: each staticlib carries its
//! own copy of `std`, so linking two would duplicate symbols.

pub use obs_util::ffi;
// Each port crate's ffi is re-exported so its no_mangle shims land in the staticlib.
pub use obs_codec::ffi as codec_ffi;
pub use obs_graphics::ffi as graphics_ffi;
