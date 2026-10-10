//! Port of `libobs/media-io/video-matrices.c`.

use crate::video_io::{VideoColorspace, VideoFormat, VideoRangeType};

/// The YUV to RGB conversion `video_format_get_parameters` writes: a
/// row-major 4x4 matrix and the per-channel range limits, normalized to
/// 0..1.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorParameters {
    pub matrix: [f32; 16],
    pub range_min: [f32; 3],
    pub range_max: [f32; 3],
}

impl VideoFormat {
    /// The bits per channel `video_format_get_parameters_for_format` uses
    /// for this format.
    pub fn bits_per_channel(self) -> u32 {
        todo!()
    }
}

/// The parameters for `color_space` and `range` at 10 bits for Rec. 2100
/// (PQ and HLG) and 8 bits otherwise (`video_format_get_parameters`).
pub fn parameters(color_space: VideoColorspace, range: VideoRangeType) -> ColorParameters {
    let _ = (color_space, range);
    todo!()
}

/// The parameters for `color_space` and `range` at the bit depth of
/// `format` (`video_format_get_parameters_for_format`).
pub fn parameters_for_format(
    color_space: VideoColorspace,
    range: VideoRangeType,
    format: VideoFormat,
) -> ColorParameters {
    let _ = (color_space, range, format);
    todo!()
}
