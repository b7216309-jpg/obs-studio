//! Port of `libobs/media-io/video-fourcc.c`.

use crate::video_io::VideoFormat;

/// `MAKE_FOURCC`: packs four characters with the first as the low byte.
pub const fn make_fourcc(code: [u8; 4]) -> u32 {
    u32::from_le_bytes(code)
}

impl VideoFormat {
    /// The format a fourcc names, or [`VideoFormat::None`] for a code the
    /// table does not list. Matching is exact and case-sensitive.
    pub fn from_fourcc(fourcc: u32) -> Self {
        match &fourcc.to_le_bytes() {
            b"UYVY" | b"HDYC" | b"UYNV" | b"UYNY" | b"uyv1" | b"2vuy" | b"2Vuy" => Self::Uyvy,
            b"YUY2" | b"Y422" | b"V422" | b"VYUY" | b"YUNV" | b"yuv2" | b"yuvs" => Self::Yuy2,
            b"YVYU" => Self::Yvyu,
            b"Y800" => Self::Y800,
            _ => Self::None,
        }
    }
}
