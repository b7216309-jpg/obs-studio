//! Test-only: the original libobs C sources compiled with `oracle_`-prefixed
//! symbols, used as the reference in Tier 2 layout tests and Tier 3
//! differential tests (`docs/rust-port/testing-policy.md`). Delete each
//! oracle together with the C source it wraps.
//!
//! Also provides a test allocator (`bmalloc`/`brealloc`/`bfree`, from
//! `oracle/test_bmem.c`) standing in for libobs `util/bmem.c`, linked
//! whole-archive so any test binary using the oracle resolves it.

pub mod path_extension {
    use core::ffi::c_char;

    unsafe extern "C" {
        pub fn oracle_os_get_path_extension(path: *const c_char) -> *const c_char;
    }
}

pub mod crc32 {
    unsafe extern "C" {
        pub fn oracle_calc_crc32(crc: u32, buf: *const core::ffi::c_void, size: usize) -> u32;
    }
}

pub mod dstr {
    //! `util/dstr.c` (the part that stays after the `dstr-libc.c` split).
    use core::ffi::{c_char, c_int};

    use super::lexer::OracleStrref;

    /// Independent declaration of `struct dstr`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct OracleDstr {
        pub array: *mut c_char,
        pub len: usize,
        pub capacity: usize,
    }

    unsafe extern "C" {
        pub fn oracle_astrcmpi(str1: *const c_char, str2: *const c_char) -> c_int;
        pub fn oracle_astrcmp_n(str1: *const c_char, str2: *const c_char, n: usize) -> c_int;
        pub fn oracle_astrcmpi_n(str1: *const c_char, str2: *const c_char, n: usize) -> c_int;
        pub fn oracle_astrstri(str: *const c_char, find: *const c_char) -> *mut c_char;
        pub fn oracle_strdepad(str: *mut c_char) -> *mut c_char;
        pub fn oracle_strlist_split(
            str: *const c_char,
            split_ch: c_char,
            include_empty: bool,
        ) -> *mut *mut c_char;
        pub fn oracle_strlist_free(strlist: *mut *mut c_char);
        pub fn oracle_dstr_init_copy_strref(dst: *mut OracleDstr, src: *const OracleStrref);
        pub fn oracle_dstr_copy(dst: *mut OracleDstr, array: *const c_char);
        pub fn oracle_dstr_copy_strref(dst: *mut OracleDstr, src: *const OracleStrref);
        pub fn oracle_dstr_ncopy(dst: *mut OracleDstr, array: *const c_char, len: usize);
        pub fn oracle_dstr_ncopy_dstr(dst: *mut OracleDstr, str: *const OracleDstr, len: usize);
        pub fn oracle_dstr_cat_dstr(dst: *mut OracleDstr, str: *const OracleDstr);
        pub fn oracle_dstr_cat_strref(dst: *mut OracleDstr, str: *const OracleStrref);
        pub fn oracle_dstr_ncat(dst: *mut OracleDstr, array: *const c_char, len: usize);
        pub fn oracle_dstr_ncat_dstr(dst: *mut OracleDstr, str: *const OracleDstr, len: usize);
        pub fn oracle_dstr_insert(dst: *mut OracleDstr, idx: usize, array: *const c_char);
        pub fn oracle_dstr_insert_dstr(dst: *mut OracleDstr, idx: usize, str: *const OracleDstr);
        pub fn oracle_dstr_insert_ch(dst: *mut OracleDstr, idx: usize, ch: c_char);
        pub fn oracle_dstr_remove(dst: *mut OracleDstr, idx: usize, count: usize);
        pub fn oracle_dstr_safe_printf(
            dst: *mut OracleDstr,
            format: *const c_char,
            val1: *const c_char,
            val2: *const c_char,
            val3: *const c_char,
            val4: *const c_char,
        );
        pub fn oracle_dstr_replace(
            str: *mut OracleDstr,
            find: *const c_char,
            replace: *const c_char,
        );
        pub fn oracle_dstr_depad(str: *mut OracleDstr);
        pub fn oracle_dstr_left(dst: *mut OracleDstr, str: *const OracleDstr, pos: usize);
        pub fn oracle_dstr_mid(
            dst: *mut OracleDstr,
            str: *const OracleDstr,
            start: usize,
            count: usize,
        );
        pub fn oracle_dstr_right(dst: *mut OracleDstr, str: *const OracleDstr, pos: usize);

        pub fn oracle_dstr_size() -> usize;
        pub fn oracle_dstr_align() -> usize;
        pub fn oracle_dstr_offset_array() -> usize;
        pub fn oracle_dstr_offset_len() -> usize;
        pub fn oracle_dstr_offset_capacity() -> usize;
    }
}

pub mod lexer {
    //! `util/lexer.c`. Its `dstr_catf` and `bmemdup` come from
    //! `oracle/test_dstr.c` and `oracle/test_bmem.c`.
    use core::ffi::{c_char, c_int};

    /// Independent declaration of `struct strref`.
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct OracleStrref {
        pub array: *const c_char,
        pub len: usize,
    }

    /// Independent declaration of `struct base_token`.
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct OracleBaseToken {
        pub text: OracleStrref,
        pub kind: c_int,
        pub passed_whitespace: bool,
    }

    /// Independent declaration of `struct error_item`.
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct OracleErrorItem {
        pub error: *mut c_char,
        pub file: *const c_char,
        pub row: u32,
        pub column: u32,
        pub level: c_int,
    }

    /// Independent declaration of `struct error_data`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct OracleErrorData {
        pub errors: super::darray::OracleDarray,
    }

    /// Independent declaration of `struct lexer`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct OracleLexer {
        pub text: *mut c_char,
        pub offset: *const c_char,
    }

    unsafe extern "C" {
        pub fn oracle_strref_cmp(str1: *const OracleStrref, str2: *const c_char) -> c_int;
        pub fn oracle_strref_cmpi(str1: *const OracleStrref, str2: *const c_char) -> c_int;
        pub fn oracle_strref_cmp_strref(
            str1: *const OracleStrref,
            str2: *const OracleStrref,
        ) -> c_int;
        pub fn oracle_strref_cmpi_strref(
            str1: *const OracleStrref,
            str2: *const OracleStrref,
        ) -> c_int;
        pub fn oracle_valid_int_str(str: *const c_char, n: usize) -> bool;
        pub fn oracle_valid_float_str(str: *const c_char, n: usize) -> bool;
        pub fn oracle_error_data_add(
            data: *mut OracleErrorData,
            file: *const c_char,
            row: u32,
            column: u32,
            msg: *const c_char,
            level: c_int,
        );
        pub fn oracle_error_data_buildstring(ed: *mut OracleErrorData) -> *mut c_char;
        pub fn oracle_lexer_getbasetoken(
            lex: *mut OracleLexer,
            token: *mut OracleBaseToken,
            iws: c_int,
        ) -> bool;
        pub fn oracle_lexer_getstroffset(
            lex: *const OracleLexer,
            str: *const c_char,
            row: *mut u32,
            col: *mut u32,
        );

        pub fn oracle_strref_size() -> usize;
        pub fn oracle_strref_align() -> usize;
        pub fn oracle_strref_offset_array() -> usize;
        pub fn oracle_strref_offset_len() -> usize;
        pub fn oracle_base_token_size() -> usize;
        pub fn oracle_base_token_align() -> usize;
        pub fn oracle_base_token_offset_text() -> usize;
        pub fn oracle_base_token_offset_type() -> usize;
        pub fn oracle_base_token_offset_passed_whitespace() -> usize;
        pub fn oracle_base_token_type_size() -> usize;
        pub fn oracle_error_item_size() -> usize;
        pub fn oracle_error_item_align() -> usize;
        pub fn oracle_error_item_offset_error() -> usize;
        pub fn oracle_error_item_offset_file() -> usize;
        pub fn oracle_error_item_offset_row() -> usize;
        pub fn oracle_error_item_offset_column() -> usize;
        pub fn oracle_error_item_offset_level() -> usize;
        pub fn oracle_error_data_size() -> usize;
        pub fn oracle_error_data_align() -> usize;
        pub fn oracle_error_data_offset_errors() -> usize;
        pub fn oracle_lexer_size() -> usize;
        pub fn oracle_lexer_align() -> usize;
        pub fn oracle_lexer_offset_text() -> usize;
        pub fn oracle_lexer_offset_offset() -> usize;
    }
}

pub mod vec2 {
    use core::ffi::c_int;

    /// Independent declaration of `struct vec2`. Intentionally not shared
    /// with `obs-graphics`.
    #[repr(C)]
    #[derive(Debug, Clone, Copy, Default, PartialEq)]
    pub struct OracleVec2 {
        pub x: f32,
        pub y: f32,
    }

    unsafe extern "C" {
        pub fn oracle_vec2_abs(dst: *mut OracleVec2, v: *const OracleVec2);
        pub fn oracle_vec2_floor(dst: *mut OracleVec2, v: *const OracleVec2);
        pub fn oracle_vec2_ceil(dst: *mut OracleVec2, v: *const OracleVec2);
        pub fn oracle_vec2_close(
            v1: *const OracleVec2,
            v2: *const OracleVec2,
            epsilon: f32,
        ) -> c_int;
        pub fn oracle_vec2_norm(dst: *mut OracleVec2, v: *const OracleVec2);

        pub fn oracle_vec2_size() -> usize;
        pub fn oracle_vec2_align() -> usize;
        pub fn oracle_vec2_offset_x() -> usize;
        pub fn oracle_vec2_offset_y() -> usize;
        pub fn oracle_vec2_offset_ptr() -> usize;
    }
}

pub mod darray {
    use core::ffi::c_void;

    /// Independent declaration of `struct darray`. Intentionally not shared
    /// with `obs-util`.
    #[repr(C)]
    #[derive(Debug)]
    pub struct OracleDarray {
        pub array: *mut c_void,
        pub num: usize,
        pub capacity: usize,
    }

    unsafe extern "C" {
        pub fn oracle_darray_free(da: *mut OracleDarray);
        pub fn oracle_darray_reserve(es: usize, da: *mut OracleDarray, capacity: usize);
        pub fn oracle_darray_ensure_capacity(es: usize, da: *mut OracleDarray, new_size: usize);
        pub fn oracle_darray_resize(es: usize, da: *mut OracleDarray, size: usize);
        pub fn oracle_darray_clear(da: *mut OracleDarray);
        pub fn oracle_darray_push_back_array(
            es: usize,
            da: *mut OracleDarray,
            array: *const c_void,
            num: usize,
        ) -> usize;
        pub fn oracle_darray_erase(es: usize, da: *mut OracleDarray, idx: usize);
        pub fn oracle_darray_pop_back(es: usize, da: *mut OracleDarray);

        pub fn oracle_darray_size() -> usize;
        pub fn oracle_darray_align() -> usize;
        pub fn oracle_darray_offset_array() -> usize;
        pub fn oracle_darray_offset_num() -> usize;
        pub fn oracle_darray_offset_capacity() -> usize;
    }
}

pub mod array_serializer {
    use core::ffi::{c_int, c_void};

    /// Independent declaration of `struct serializer`.
    #[repr(C)]
    pub struct OracleSerializer {
        pub data: *mut c_void,
        pub read: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> usize>,
        pub write: Option<unsafe extern "C" fn(*mut c_void, *const c_void, usize) -> usize>,
        pub seek: Option<unsafe extern "C" fn(*mut c_void, i64, c_int) -> i64>,
        pub get_pos: Option<unsafe extern "C" fn(*mut c_void) -> i64>,
    }

    /// Independent declaration of `struct array_output_data`.
    #[repr(C)]
    pub struct OracleArrayOutputData {
        pub bytes: super::darray::OracleDarray,
        pub cur_pos: usize,
    }

    unsafe extern "C" {
        pub fn oracle_array_output_serializer_init(
            s: *mut OracleSerializer,
            data: *mut OracleArrayOutputData,
        );
        pub fn oracle_array_output_serializer_free(data: *mut OracleArrayOutputData);
        pub fn oracle_array_output_serializer_reset(data: *mut OracleArrayOutputData);

        pub fn oracle_serializer_size() -> usize;
        pub fn oracle_serializer_align() -> usize;
        pub fn oracle_serializer_offset_data() -> usize;
        pub fn oracle_serializer_offset_read() -> usize;
        pub fn oracle_serializer_offset_write() -> usize;
        pub fn oracle_serializer_offset_seek() -> usize;
        pub fn oracle_serializer_offset_get_pos() -> usize;

        pub fn oracle_array_output_data_size() -> usize;
        pub fn oracle_array_output_data_align() -> usize;
        pub fn oracle_array_output_data_offset_bytes() -> usize;
        pub fn oracle_array_output_data_offset_cur_pos() -> usize;
    }
}

pub mod bitstream {
    use core::ffi::c_int;

    /// Independent declaration of `struct bitstream_reader` for calling the
    /// oracle. Intentionally not shared with `obs-util`.
    #[repr(C)]
    #[allow(non_snake_case)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OracleReader {
        pub pos: u8,
        pub subPos: u8,
        pub buf: *mut u8,
        pub len: usize,
    }

    unsafe extern "C" {
        pub fn oracle_bitstream_reader_init(r: *mut OracleReader, data: *mut u8, len: usize);
        pub fn oracle_bitstream_reader_read_bits(r: *mut OracleReader, bits: c_int) -> u8;
        pub fn oracle_bitstream_reader_r8(r: *mut OracleReader) -> u8;
        pub fn oracle_bitstream_reader_r16(r: *mut OracleReader) -> u16;

        pub fn oracle_bitstream_reader_size() -> usize;
        pub fn oracle_bitstream_reader_align() -> usize;
        pub fn oracle_bitstream_reader_offset_pos() -> usize;
        pub fn oracle_bitstream_reader_offset_subPos() -> usize;
        pub fn oracle_bitstream_reader_offset_buf() -> usize;
        pub fn oracle_bitstream_reader_offset_len() -> usize;
    }
}
