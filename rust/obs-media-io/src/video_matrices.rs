//! Port of `libobs/media-io/video-matrices.c`.
//!
//! The C fills a table of every matrix on first use, behind an
//! unsynchronized `static bool`. This port has no shared state: it computes
//! the one requested matrix on each call, with the same `f32` operations in
//! the same order, so the results are bit-identical to the table's.

use obs_graphics::matrix3::Matrix3;
use obs_graphics::vec3::Vec3;

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
        match self {
            Self::I010 | Self::P010 | Self::I210 | Self::V210 | Self::R10l => 10,
            Self::I412 | Self::Ya2l => 12,
            Self::P216 | Self::P416 => 16,
            _ => 8,
        }
    }
}

/// The parameters for `color_space` and `range` at 10 bits for Rec. 2100
/// (PQ and HLG) and 8 bits otherwise (`video_format_get_parameters`).
pub fn parameters(color_space: VideoColorspace, range: VideoRangeType) -> ColorParameters {
    let bpc = match color_space {
        VideoColorspace::Cs2100Pq | VideoColorspace::Cs2100Hlg => 10,
        _ => 8,
    };
    parameters_for_bpc(color_space, range, bpc)
}

/// The parameters for `color_space` and `range` at the bit depth of
/// `format` (`video_format_get_parameters_for_format`).
pub fn parameters_for_format(
    color_space: VideoColorspace,
    range: VideoRangeType,
    format: VideoFormat,
) -> ColorParameters {
    parameters_for_bpc(color_space, range, format.bits_per_channel())
}

/// `Kb` and `Kr` of the `format_info` entry `color_space` maps to: DEFAULT
/// and sRGB use Rec. 709, HLG uses PQ.
fn coefficients(color_space: VideoColorspace) -> (f32, f32) {
    match color_space {
        VideoColorspace::Cs601 => (0.114, 0.299),
        VideoColorspace::Default | VideoColorspace::Cs709 | VideoColorspace::Srgb => {
            (0.0722, 0.2126)
        }
        VideoColorspace::Cs2100Pq | VideoColorspace::Cs2100Hlg => (0.0593, 0.2627),
    }
}

/// `video_format_get_parameters_for_bpc`. `bpc` is clamped to 8..=16, as in
/// C, though both callers pass 8, 10, 12 or 16.
fn parameters_for_bpc(
    color_space: VideoColorspace,
    range: VideoRangeType,
    bpc: u32,
) -> ColorParameters {
    let (kb, kr) = coefficients(color_space);

    // `initialize_matrices` doubles these from their 8-bit values once per
    // extra bit; powers of two keep every step exact.
    let scale = (1u32 << (bpc.clamp(8, 16) - 8)) as f32;
    let min_value = 16.0 * scale;
    let max_luma = 235.0 * scale;
    let max_chroma = 240.0 * scale;
    let range_max = 256.0 * scale - 1.0;
    let mid_chroma = 0.5 * (min_value + max_chroma);

    if range == VideoRangeType::Full {
        ColorParameters {
            matrix: initialize_matrix(
                kb,
                kr,
                range_max,
                [0.0; 3],
                [range_max; 3],
                [0.0, mid_chroma, mid_chroma],
            ),
            range_min: [0.0; 3],
            range_max: [1.0; 3],
        }
    } else {
        ColorParameters {
            matrix: initialize_matrix(
                kb,
                kr,
                range_max,
                [min_value; 3],
                [max_luma, max_chroma, max_chroma],
                [min_value, mid_chroma, mid_chroma],
            ),
            range_min: [min_value / range_max; 3],
            range_max: [
                max_luma / range_max,
                max_chroma / range_max,
                max_chroma / range_max,
            ],
        }
    }
}

/// `initialize_matrix`: the YUV to RGB matrix for `kb` and `kr` with the
/// given limits, and the black-level offsets rotated into the last column.
fn initialize_matrix(
    kb: f32,
    kr: f32,
    bit_range_max: f32,
    range_min: [f32; 3],
    range_max: [f32; 3],
    black_levels: [f32; 3],
) -> [f32; 16] {
    let yvals = range_max[0] - range_min[0];
    let uvals = (range_max[1] - range_min[1]) / 2.0;
    let vvals = (range_max[2] - range_min[2]) / 2.0;

    let yscale = bit_range_max / yvals;
    let uscale = bit_range_max / uvals;
    let vscale = bit_range_max / vvals;

    let kg = 1.0 - kb - kr;

    // `color_matrix.t` is left uninitialized in C; `vec3_rotate` ignores it.
    let color_matrix = Matrix3::new(
        Vec3::new(yscale, 0.0, vscale * (1.0 - kr)),
        Vec3::new(
            yscale,
            uscale * (kb - 1.0) * kb / kg,
            vscale * (kr - 1.0) * kr / kg,
        ),
        Vec3::new(yscale, uscale * (1.0 - kb), 0.0),
        Vec3::default(),
    );

    let offsets = Vec3::new(
        -black_levels[0] / bit_range_max,
        -black_levels[1] / bit_range_max,
        -black_levels[2] / bit_range_max,
    );
    let multiplied = offsets.rotate(&color_matrix);

    let (x, y, z) = (color_matrix.x, color_matrix.y, color_matrix.z);
    [
        x.x,
        x.y,
        x.z,
        multiplied.x,
        y.x,
        y.y,
        y.z,
        multiplied.y,
        z.x,
        z.y,
        z.z,
        multiplied.z,
        0.0,
        0.0,
        0.0,
        1.0,
    ]
}
