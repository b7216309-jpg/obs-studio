//! Tier 2: the `#[repr(C)]` mirrors of `libobs/util/lexer.h` match the
//! layouts the C compiler produces for the real header.

use core::ffi::c_int;
use core::mem::{align_of, offset_of, size_of};

use obs_c_oracle::lexer as c;
use obs_util::ffi::lexer::{base_token, error_data, error_item, lexer, strref};

#[test]
fn strref_layout_matches_c_header() {
    // SAFETY: the oracle layout functions take no arguments and only return constants.
    unsafe {
        assert_eq!(size_of::<strref>(), c::oracle_strref_size());
        assert_eq!(align_of::<strref>(), c::oracle_strref_align());
        assert_eq!(offset_of!(strref, array), c::oracle_strref_offset_array());
        assert_eq!(offset_of!(strref, len), c::oracle_strref_offset_len());
    }
}

#[test]
fn base_token_layout_matches_c_header() {
    // SAFETY: as above.
    unsafe {
        assert_eq!(size_of::<base_token>(), c::oracle_base_token_size());
        assert_eq!(align_of::<base_token>(), c::oracle_base_token_align());
        assert_eq!(
            offset_of!(base_token, text),
            c::oracle_base_token_offset_text()
        );
        assert_eq!(
            offset_of!(base_token, r#type),
            c::oracle_base_token_offset_type()
        );
        assert_eq!(
            offset_of!(base_token, passed_whitespace),
            c::oracle_base_token_offset_passed_whitespace()
        );
        assert_eq!(size_of::<c_int>(), c::oracle_base_token_type_size());
    }
}

#[test]
fn error_item_layout_matches_c_header() {
    // SAFETY: as above.
    unsafe {
        assert_eq!(size_of::<error_item>(), c::oracle_error_item_size());
        assert_eq!(align_of::<error_item>(), c::oracle_error_item_align());
        assert_eq!(
            offset_of!(error_item, error),
            c::oracle_error_item_offset_error()
        );
        assert_eq!(
            offset_of!(error_item, file),
            c::oracle_error_item_offset_file()
        );
        assert_eq!(
            offset_of!(error_item, row),
            c::oracle_error_item_offset_row()
        );
        assert_eq!(
            offset_of!(error_item, column),
            c::oracle_error_item_offset_column()
        );
        assert_eq!(
            offset_of!(error_item, level),
            c::oracle_error_item_offset_level()
        );
    }
}

#[test]
fn error_data_layout_matches_c_header() {
    // SAFETY: as above.
    unsafe {
        assert_eq!(size_of::<error_data>(), c::oracle_error_data_size());
        assert_eq!(align_of::<error_data>(), c::oracle_error_data_align());
        assert_eq!(
            offset_of!(error_data, errors),
            c::oracle_error_data_offset_errors()
        );
    }
}

#[test]
fn lexer_layout_matches_c_header() {
    // SAFETY: as above.
    unsafe {
        assert_eq!(size_of::<lexer>(), c::oracle_lexer_size());
        assert_eq!(align_of::<lexer>(), c::oracle_lexer_align());
        assert_eq!(offset_of!(lexer, text), c::oracle_lexer_offset_text());
        assert_eq!(offset_of!(lexer, offset), c::oracle_lexer_offset_offset());
    }
}
