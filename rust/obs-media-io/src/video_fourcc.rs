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
        let _ = fourcc;
        todo!()
    }
}
