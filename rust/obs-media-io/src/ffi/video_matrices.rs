//! C ABI shims for the exported functions in `libobs/media-io/video-io.h`
//! that `video-matrices.c` defines.
//!
//! The enums cross the C ABI as `c_int`s. A colorspace outside
//! `enum video_colorspace` makes both functions return false without
//! writing. `video-matrices.c` only tests `range` against
//! `VIDEO_RANGE_FULL` and lists the formats it gives more than 8 bits, so
//! values outside those enums convert to `VIDEO_RANGE_PARTIAL` and
//! `VIDEO_FORMAT_NONE`, which behave the same.

use core::ffi::c_int;
use core::ptr;

use crate::video_io::{
    VideoFormat, VideoRangeType, video_colorspace_from_c, video_format_from_c,
    video_range_type_from_c,
};
use crate::video_matrices::{self, ColorParameters};

/// Writes `params` through the C out-pointers, skipping null range ones.
///
/// # Safety
///
/// As [`video_format_get_parameters`].
unsafe fn write_parameters(
    params: &ColorParameters,
    matrix: *mut f32,
    range_min: *mut f32,
    range_max: *mut f32,
) {
    // SAFETY: the caller guarantees `matrix` is writable for 16 floats.
    unsafe { ptr::write(matrix.cast::<[f32; 16]>(), params.matrix) };
    if !range_min.is_null() {
        // SAFETY: the caller guarantees a non-null `range_min` is writable
        // for 3 floats.
        unsafe { ptr::write(range_min.cast::<[f32; 3]>(), params.range_min) };
    }
    if !range_max.is_null() {
        // SAFETY: as for `range_min`.
        unsafe { ptr::write(range_max.cast::<[f32; 3]>(), params.range_max) };
    }
}

/// Writes the YUV to RGB matrix and range limits for `color_space` and
/// `range` at its default bit depth. Returns false, writing nothing, for an
/// unknown `color_space`.
///
/// # Safety
///
/// `matrix` must be valid for writes of 16 floats. `range_min` and
/// `range_max` must each be null or valid for writes of 3 floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn video_format_get_parameters(
    color_space: c_int,
    range: c_int,
    matrix: *mut f32,
    range_min: *mut f32,
    range_max: *mut f32,
) -> bool {
    let Some(color_space) = video_colorspace_from_c(color_space) else {
        return false;
    };
    let range = video_range_type_from_c(range).unwrap_or(VideoRangeType::Partial);
    let params = video_matrices::parameters(color_space, range);
    // SAFETY: forwarded from this function's contract.
    unsafe { write_parameters(&params, matrix, range_min, range_max) };
    true
}

/// As [`video_format_get_parameters`], at the bit depth of `format`.
///
/// # Safety
///
/// As [`video_format_get_parameters`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn video_format_get_parameters_for_format(
    color_space: c_int,
    range: c_int,
    format: c_int,
    matrix: *mut f32,
    range_min: *mut f32,
    range_max: *mut f32,
) -> bool {
    let Some(color_space) = video_colorspace_from_c(color_space) else {
        return false;
    };
    let range = video_range_type_from_c(range).unwrap_or(VideoRangeType::Partial);
    let format = video_format_from_c(format).unwrap_or(VideoFormat::None);
    let params = video_matrices::parameters_for_format(color_space, range, format);
    // SAFETY: forwarded from this function's contract.
    unsafe { write_parameters(&params, matrix, range_min, range_max) };
    true
}
