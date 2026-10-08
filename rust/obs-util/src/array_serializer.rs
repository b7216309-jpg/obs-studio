//! Safe core for `libobs/util/array-serializer.c`.
//!
//! [`ArrayOutput`] reproduces the C write/seek/get_pos behaviour exactly,
//! including its quirks (`get_pos` reports the buffer length, not the write
//! position; appending writes at the old length). The C ABI shim lives in
//! [`crate::ffi::array_serializer`]. `serializer.h` is header-inline and is
//! covered by a layout test only.

use core::ffi::c_int;

use crate::darray::DArray;

/// `enum serialize_seek_type` from `serializer.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeekType {
    Start,
    Current,
    End,
}

/// Converts the C enum value; anything outside 0..=2 is `None`.
pub fn seek_type_from_c(v: c_int) -> Option<SeekType> {
    match v {
        0 => Some(SeekType::Start),
        1 => Some(SeekType::Current),
        2 => Some(SeekType::End),
        _ => None,
    }
}

/// New position for a seek, with the C unsigned wrapping arithmetic.
/// `None` kind (an invalid enum value) targets 0. Returns `None` when the
/// target is past `num`.
pub fn seek_target(
    cur_pos: usize,
    num: usize,
    offset: i64,
    kind: Option<SeekType>,
) -> Option<usize> {
    let target = match kind {
        Some(SeekType::Start) => offset as usize,
        Some(SeekType::Current) => cur_pos.wrapping_add(offset as usize),
        Some(SeekType::End) => num.wrapping_sub(offset as usize),
        None => 0,
    };
    if target > num { None } else { Some(target) }
}

/// How a write of `size` bytes is carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritePlan {
    /// Copy over the buffer at `offset`, growing the length to `new_num`.
    Overwrite { offset: usize, new_num: usize },
    /// `da_push_back_array`: written at the OLD length, not at `cur_pos`.
    Append,
}

/// Chooses the write strategy. In every case `cur_pos += size` follows.
pub fn plan_write(cur_pos: usize, num: usize, size: usize) -> WritePlan {
    if cur_pos < num {
        WritePlan::Overwrite {
            offset: cur_pos,
            new_num: num.max(cur_pos.wrapping_add(size)),
        }
    } else {
        WritePlan::Append
    }
}

/// Mirror of `struct array_output_data` plus its serializer callbacks.
#[derive(Debug, Clone, Default)]
pub struct ArrayOutput {
    bytes: DArray<u8>,
    cur_pos: usize,
}

impl ArrayOutput {
    pub fn new() -> Self {
        Self {
            bytes: DArray::new(),
            cur_pos: 0,
        }
    }

    /// `array_output_write`; returns `data.len()`.
    pub fn write(&mut self, data: &[u8]) -> usize {
        let size = data.len();
        match plan_write(self.cur_pos, self.bytes.len(), size) {
            WritePlan::Overwrite { offset, new_num } => {
                // C: darray_ensure_capacity(new_num) and num = new_num when
                // growing. `resize` does exactly that (its zero fill is fully
                // overwritten below, since new bytes lie in offset..offset+size).
                self.bytes.resize(new_num);
                self.bytes.as_mut_slice()[offset..offset + size].copy_from_slice(data);
            }
            WritePlan::Append => {
                self.bytes.push_back_array(data);
            }
        }
        self.cur_pos = self.cur_pos.wrapping_add(size);
        size
    }

    /// `array_output_get_pos`: the buffer length, NOT `cur_pos`.
    pub fn get_pos(&self) -> i64 {
        self.bytes.len() as i64
    }

    /// `array_output_seek`; `-1` on failure (position unchanged).
    pub fn seek(&mut self, offset: i64, kind: Option<SeekType>) -> i64 {
        match seek_target(self.cur_pos, self.bytes.len(), offset, kind) {
            Some(target) => {
                self.cur_pos = target;
                target as i64
            }
            None => -1,
        }
    }

    /// `array_output_serializer_reset`: clear (keeps capacity) and rewind.
    pub fn reset(&mut self) {
        self.bytes.clear();
        self.cur_pos = 0;
    }

    pub fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    pub fn cur_pos(&self) -> usize {
        self.cur_pos
    }

    /// The C-model capacity of the byte buffer.
    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }
}
