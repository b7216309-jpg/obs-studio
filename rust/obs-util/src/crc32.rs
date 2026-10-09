//! Safe core for `calc_crc32` (`libobs/util/crc32.c`).

/// Reflected CRC-32 polynomial (IEEE 802.3).
const POLY: u32 = 0xEDB8_8320;

/// The 256-entry lookup table, identical to `crc32_tab` in the C source.
const TABLE: [u32; 256] = make_table();

const fn make_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { (c >> 1) ^ POLY } else { c >> 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

/// Updates the CRC-32 `crc` with `buf` and returns the new value.
///
/// Table-driven reflected CRC-32 (polynomial `0xEDB88320`). The running value
/// is inverted before and after processing, so passing the result of a
/// previous call as `crc` continues the checksum; start with `0`.
pub fn calc_crc32(crc: u32, buf: &[u8]) -> u32 {
    let mut crc = !crc;
    for &b in buf {
        crc = TABLE[((crc ^ u32::from(b)) & 0xFF) as usize] ^ (crc >> 8);
    }
    !crc
}
