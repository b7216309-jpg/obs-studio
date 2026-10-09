//! Tier 2: `#[repr(C)] dstr` matches the layout the C compiler produces for
//! `libobs/util/dstr.h`.

use core::mem::{align_of, offset_of, size_of};

use obs_c_oracle::dstr as c;
use obs_util::ffi::dstr::dstr;

#[test]
fn dstr_layout_matches_c_header() {
    // SAFETY: the oracle layout functions take no arguments and only return constants.
    unsafe {
        assert_eq!(size_of::<dstr>(), c::oracle_dstr_size());
        assert_eq!(align_of::<dstr>(), c::oracle_dstr_align());
        assert_eq!(offset_of!(dstr, array), c::oracle_dstr_offset_array());
        assert_eq!(offset_of!(dstr, len), c::oracle_dstr_offset_len());
        assert_eq!(offset_of!(dstr, capacity), c::oracle_dstr_offset_capacity());
    }
}
