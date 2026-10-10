//! Safe core for `libobs/util/config-file.c`.
//!
//! The C ABI shim lives in [`crate::ffi::config_file`].

/// `CONFIG_SUCCESS` from `util/config-file.h`.
pub const CONFIG_SUCCESS: i32 = 0;
/// `CONFIG_FILENOTFOUND` from `util/config-file.h`.
pub const CONFIG_FILENOTFOUND: i32 = -1;
/// `CONFIG_ERROR` from `util/config-file.h`.
pub const CONFIG_ERROR: i32 = -2;

/// `enum config_open_type` from `util/config-file.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenType {
    Existing,
    Always,
}

/// C keeps `CONFIG_OPEN_EXISTING` for any value other than
/// `CONFIG_OPEN_ALWAYS`.
pub fn open_type_from_c(v: i32) -> OpenType {
    match v {
        1 => OpenType::Always,
        _ => OpenType::Existing,
    }
}

#[derive(Debug)]
pub struct Config;

impl Config {
    pub fn open_string(_text: &[u8]) -> Self {
        todo!()
    }

    pub fn open(_file: &[u8], _open_type: OpenType) -> Result<Self, i32> {
        todo!()
    }

    pub fn create(_file: &[u8]) -> Option<Self> {
        todo!()
    }

    pub fn open_defaults(&self, _file: &[u8]) -> i32 {
        todo!()
    }

    pub fn save(&self) -> i32 {
        todo!()
    }

    pub fn save_safe(&self, _temp_ext: Option<&[u8]>, _backup_ext: Option<&[u8]>) -> i32 {
        todo!()
    }

    pub fn num_sections(&self) -> usize {
        todo!()
    }

    pub fn get_section(&self, _idx: usize) -> Option<Vec<u8>> {
        todo!()
    }

    pub fn get_string(&self, _section: &[u8], _name: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    pub fn get_int(&self, _section: &[u8], _name: &[u8]) -> i64 {
        todo!()
    }

    pub fn get_uint(&self, _section: &[u8], _name: &[u8]) -> u64 {
        todo!()
    }

    pub fn get_bool(&self, _section: &[u8], _name: &[u8]) -> bool {
        todo!()
    }

    pub fn get_double(&self, _section: &[u8], _name: &[u8]) -> f64 {
        todo!()
    }

    pub fn set_string(&self, _section: &[u8], _name: &[u8], _value: &[u8]) {
        todo!()
    }

    pub fn set_int(&self, _section: &[u8], _name: &[u8], _value: i64) {
        todo!()
    }

    pub fn set_uint(&self, _section: &[u8], _name: &[u8], _value: u64) {
        todo!()
    }

    pub fn set_bool(&self, _section: &[u8], _name: &[u8], _value: bool) {
        todo!()
    }

    pub fn set_double(&self, _section: &[u8], _name: &[u8], _value: f64) {
        todo!()
    }

    pub fn remove_value(&self, _section: &[u8], _name: &[u8]) -> bool {
        todo!()
    }

    pub fn set_default_string(&self, _section: &[u8], _name: &[u8], _value: &[u8]) {
        todo!()
    }

    pub fn set_default_int(&self, _section: &[u8], _name: &[u8], _value: i64) {
        todo!()
    }

    pub fn set_default_uint(&self, _section: &[u8], _name: &[u8], _value: u64) {
        todo!()
    }

    pub fn set_default_bool(&self, _section: &[u8], _name: &[u8], _value: bool) {
        todo!()
    }

    pub fn set_default_double(&self, _section: &[u8], _name: &[u8], _value: f64) {
        todo!()
    }

    pub fn get_default_string(&self, _section: &[u8], _name: &[u8]) -> Option<Vec<u8>> {
        todo!()
    }

    pub fn get_default_int(&self, _section: &[u8], _name: &[u8]) -> i64 {
        todo!()
    }

    pub fn get_default_uint(&self, _section: &[u8], _name: &[u8]) -> u64 {
        todo!()
    }

    pub fn get_default_bool(&self, _section: &[u8], _name: &[u8]) -> bool {
        todo!()
    }

    pub fn get_default_double(&self, _section: &[u8], _name: &[u8]) -> f64 {
        todo!()
    }

    pub fn has_user_value(&self, _section: &[u8], _name: &[u8]) -> bool {
        todo!()
    }

    pub fn has_default_value(&self, _section: &[u8], _name: &[u8]) -> bool {
        todo!()
    }
}
