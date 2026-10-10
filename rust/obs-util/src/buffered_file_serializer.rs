//! Safe core for `libobs/util/buffered-file-serializer.c`.
//!
//! RED stub. The Tier 1 tests name the cmocka cases they mirror.

use std::path::Path;

/// `enum serialize_seek_type` in `libobs/util/serializer.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekType {
    Start,
    Current,
    End,
}

/// Map a C `serialize_seek_type`. Anything else leaves the position alone,
/// matching the C `switch` with no default.
pub fn seek_type_from_c(code: i32) -> Option<SeekType> {
    match code {
        0 => Some(SeekType::Start),
        1 => Some(SeekType::Current),
        2 => Some(SeekType::End),
        _ => None,
    }
}

/// Buffered file output. Stubbed until the port is filled in.
pub struct Output {
    _private: (),
}

impl Output {
    /// `buffered_file_serializer_init`. A zero size selects the C default.
    pub fn create(_path: &Path, _max_bufsize: usize, _chunk_size: usize) -> Option<Self> {
        todo!("buffered-file-serializer")
    }

    /// `buffered_file_serializer_init_defaults`.
    pub fn create_defaults(path: &Path) -> Option<Self> {
        Self::create(path, 0, 0)
    }

    pub fn write(&self, _buf: &[u8]) -> usize {
        todo!("buffered-file-serializer")
    }

    pub fn seek(&self, _offset: i64, _kind: Option<SeekType>) -> i64 {
        todo!("buffered-file-serializer")
    }

    pub fn pos(&self) -> i64 {
        todo!("buffered-file-serializer")
    }
}
