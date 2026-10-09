//! Safe core for `os_get_path_extension` (`libobs/util/path-extension.c`).

/// Returns the extension of `path`, including the leading `.`.
///
/// Scans from the end for the last `.` and returns the suffix starting at
/// it. The scan stops with `None` on reaching a `/` or `\` first, or the
/// start of the path. This matches the C loop exactly, so `"/\\."` gives
/// `"."` and the empty path gives `None`.
pub fn path_extension(path: &[u8]) -> Option<&[u8]> {
    for (i, &c) in path.iter().enumerate().rev() {
        match c {
            b'.' => return Some(&path[i..]),
            b'/' | b'\\' => return None,
            _ => {}
        }
    }
    None
}
