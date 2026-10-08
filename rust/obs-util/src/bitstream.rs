//! MSB-first bit reader. Port of `libobs/util/bitstream.c`.

/// Reads bits MSB-first from a byte slice. Reads past the end yield 0.
pub struct BitstreamReader<'a> {
    buf: &'a [u8],
    pos: usize,
    sub_pos: u8,
}

impl<'a> BitstreamReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        todo!()
    }

    pub fn read_bit(&mut self) -> u8 {
        todo!()
    }

    /// Reads `bits` bits. Like the C version, only the low 8 bits of the
    /// result are kept when `bits > 8`.
    pub fn read_bits(&mut self, bits: u32) -> u8 {
        todo!()
    }

    pub fn r8(&mut self) -> u8 {
        todo!()
    }

    /// Reads a big-endian `u16`.
    pub fn r16(&mut self) -> u16 {
        todo!()
    }
}
