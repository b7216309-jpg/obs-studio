//! Types from `libobs/media-io/video-io.h`.

use core::ffi::c_int;

/// `enum video_format`, in header order. The discriminants are the
/// header's values; the C ABI passes the enum as a `c_int`, converted with
/// [`video_format_to_c`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoFormat {
    None = 0,

    // planar 4:2:0 formats
    I420 = 1,
    Nv12 = 2,

    // packed 4:2:2 formats
    Yvyu = 3,
    Yuy2 = 4,
    Uyvy = 5,

    // packed uncompressed formats
    Rgba = 6,
    Bgra = 7,
    Bgrx = 8,
    Y800 = 9,

    // planar 4:4:4
    I444 = 10,

    // more packed uncompressed formats
    Bgr3 = 11,

    // planar 4:2:2
    I422 = 12,

    // planar 4:2:0 with alpha
    I40a = 13,

    // planar 4:2:2 with alpha
    I42a = 14,

    // planar 4:4:4 with alpha
    Yuva = 15,

    // packed 4:4:4 with alpha
    Ayuv = 16,

    // planar 4:2:0 format, 10 bpp
    I010 = 17,
    P010 = 18,

    // planar 4:2:2 format, 10 bpp
    I210 = 19,

    // planar 4:4:4 format, 12 bpp
    I412 = 20,

    // planar 4:4:4:4 format, 12 bpp
    Ya2l = 21,

    // planar 4:2:2 format, 16 bpp
    P216 = 22,

    // planar 4:4:4 format, 16 bpp
    P416 = 23,

    // packed 4:2:2 format, 10 bpp
    V210 = 24,

    // packed uncompressed 10-bit format
    R10l = 25,
}

impl VideoFormat {
    /// Every variant, in header order.
    pub const ALL: [Self; 26] = [
        Self::None,
        Self::I420,
        Self::Nv12,
        Self::Yvyu,
        Self::Yuy2,
        Self::Uyvy,
        Self::Rgba,
        Self::Bgra,
        Self::Bgrx,
        Self::Y800,
        Self::I444,
        Self::Bgr3,
        Self::I422,
        Self::I40a,
        Self::I42a,
        Self::Yuva,
        Self::Ayuv,
        Self::I010,
        Self::P010,
        Self::I210,
        Self::I412,
        Self::Ya2l,
        Self::P216,
        Self::P416,
        Self::V210,
        Self::R10l,
    ];
}

/// The C value of `enum video_format` for `format`.
pub fn video_format_to_c(format: VideoFormat) -> c_int {
    format as c_int
}

/// The format whose C value is `value`, or `None` for a value outside
/// `enum video_format`.
pub fn video_format_from_c(value: c_int) -> Option<VideoFormat> {
    VideoFormat::ALL
        .into_iter()
        .find(|&format| video_format_to_c(format) == value)
}

/// `enum video_colorspace`, in header order. The discriminants are the
/// header's values; the C ABI passes the enum as a `c_int`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoColorspace {
    Default = 0,
    Cs601 = 1,
    Cs709 = 2,
    Srgb = 3,
    Cs2100Pq = 4,
    Cs2100Hlg = 5,
}

impl VideoColorspace {
    /// Every variant, in header order.
    pub const ALL: [Self; 6] = [
        Self::Default,
        Self::Cs601,
        Self::Cs709,
        Self::Srgb,
        Self::Cs2100Pq,
        Self::Cs2100Hlg,
    ];
}

/// The C value of `enum video_colorspace` for `color_space`.
pub fn video_colorspace_to_c(color_space: VideoColorspace) -> c_int {
    color_space as c_int
}

/// The colorspace whose C value is `value`, or `None` for a value outside
/// `enum video_colorspace`.
pub fn video_colorspace_from_c(value: c_int) -> Option<VideoColorspace> {
    VideoColorspace::ALL
        .into_iter()
        .find(|&color_space| video_colorspace_to_c(color_space) == value)
}

/// `enum video_range_type`, in header order. The discriminants are the
/// header's values; the C ABI passes the enum as a `c_int`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VideoRangeType {
    Default = 0,
    Partial = 1,
    Full = 2,
}

impl VideoRangeType {
    /// Every variant, in header order.
    pub const ALL: [Self; 3] = [Self::Default, Self::Partial, Self::Full];
}

/// The C value of `enum video_range_type` for `range`.
pub fn video_range_type_to_c(range: VideoRangeType) -> c_int {
    range as c_int
}

/// The range type whose C value is `value`, or `None` for a value outside
/// `enum video_range_type`.
pub fn video_range_type_from_c(value: c_int) -> Option<VideoRangeType> {
    VideoRangeType::ALL
        .into_iter()
        .find(|&range| video_range_type_to_c(range) == value)
}
