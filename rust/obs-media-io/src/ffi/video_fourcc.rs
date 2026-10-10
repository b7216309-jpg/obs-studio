//! C ABI shim for the exported function in `libobs/media-io/video-io.h`
//! that `video-fourcc.c` defines.

use core::ffi::c_int;

use crate::video_io::{VideoFormat, video_format_to_c};

/// Returns the video format a fourcc names, or `VIDEO_FORMAT_NONE`, as the
/// C `enum video_format` value.
#[unsafe(no_mangle)]
pub extern "C" fn video_format_from_fourcc(fourcc: u32) -> c_int {
    video_format_to_c(VideoFormat::from_fourcc(fourcc))
}
