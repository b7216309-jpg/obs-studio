//! MSB-first bit reader. Port of `libobs/util/bitstream.c`.

/// Byte position within the buffer.
///
/// The safe API uses `usize`. The C ABI keeps the original `uint8_t pos`,
/// which wraps back to 0 after byte 255 and re-reads the start of the
/// buffer; that behavior is part of the public struct layout, so the shim
/// reproduces it with `u8`.
pub(crate) trait Position: Copy {
    fn index(self) -> usize;
    fn advance(&mut self);
}

impl Position for usize {
    fn index(self) -> usize {
        self
    }

    fn advance(&mut self) {
        *self += 1;
    }
}

impl Position for u8 {
    fn index(self) -> usize {
        usize::from(self)
    }

    fn advance(&mut self) {
        *self = self.wrapping_add(1);
    }
}

/// Shared implementation behind [`BitstreamReader`] and the C shim.
pub(crate) struct Reader<'a, P> {
    pub(crate) buf: &'a [u8],
    pub(crate) pos: P,
    pub(crate) sub_pos: u8,
}

impl<P: Position> Reader<'_, P> {
    pub(crate) fn read_bit(&mut self) -> u8 {
        let Some(&byte) = self.buf.get(self.pos.index()) else {
            return 0;
        };

        // Same test as C: a zero `sub_pos` (from a zeroed, uninitialized C
        // struct) reads as 1.
        let bit = u8::from(byte & self.sub_pos == self.sub_pos);

        self.sub_pos >>= 1;
        if self.sub_pos == 0 {
            self.sub_pos = 0x80;
            self.pos.advance();
        }

        bit
    }

    pub(crate) fn read_bits(&mut self, bits: u32) -> u8 {
        let mut res: u8 = 0;
        for _ in 0..bits {
            res = (res << 1) | self.read_bit();
        }
        res
    }

    pub(crate) fn r8(&mut self) -> u8 {
        self.read_bits(8)
    }

    pub(crate) fn r16(&mut self) -> u16 {
        let hi = u16::from(self.r8());
        (hi << 8) | u16::from(self.r8())
    }
}

/// Reads bits MSB-first from a byte slice. Reads past the end yield 0.
pub struct BitstreamReader<'a>(Reader<'a, usize>);

impl<'a> BitstreamReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self(Reader {
            buf,
            pos: 0,
            sub_pos: 0x80,
        })
    }

    pub fn read_bit(&mut self) -> u8 {
        self.0.read_bit()
    }

    /// Reads `bits` bits. Like the C version, only the low 8 bits of the
    /// result are kept when `bits > 8`.
    pub fn read_bits(&mut self, bits: u32) -> u8 {
        self.0.read_bits(bits)
    }

    pub fn r8(&mut self) -> u8 {
        self.0.r8()
    }

    /// Reads a big-endian `u16`.
    pub fn r16(&mut self) -> u16 {
        self.0.r16()
    }
}
