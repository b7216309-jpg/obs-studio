/* Test-only oracle: the original libobs/util/dstr.c (without the functions
 * moved to dstr-libc.c), unmodified, with every global symbol renamed to
 * oracle_* so it can link next to the Rust implementation, plus the layout
 * of struct dstr as the C compiler sees it. */
#include <stddef.h>

#define astrcmpi oracle_astrcmpi
#define astrcmp_n oracle_astrcmp_n
#define astrcmpi_n oracle_astrcmpi_n
#define astrstri oracle_astrstri
#define strdepad oracle_strdepad
#define strlist_split oracle_strlist_split
#define strlist_free oracle_strlist_free
#define dstr_init_copy_strref oracle_dstr_init_copy_strref
#define dstr_copy oracle_dstr_copy
#define dstr_copy_strref oracle_dstr_copy_strref
#define dstr_ncopy oracle_dstr_ncopy
#define dstr_ncopy_dstr oracle_dstr_ncopy_dstr
#define dstr_cat_dstr oracle_dstr_cat_dstr
#define dstr_cat_strref oracle_dstr_cat_strref
#define dstr_ncat oracle_dstr_ncat
#define dstr_ncat_dstr oracle_dstr_ncat_dstr
#define dstr_insert oracle_dstr_insert
#define dstr_insert_dstr oracle_dstr_insert_dstr
#define dstr_insert_ch oracle_dstr_insert_ch
#define dstr_remove oracle_dstr_remove
#define dstr_safe_printf oracle_dstr_safe_printf
#define dstr_replace oracle_dstr_replace
#define dstr_depad oracle_dstr_depad
#define dstr_left oracle_dstr_left
#define dstr_mid oracle_dstr_mid
#define dstr_right oracle_dstr_right

#include "util/dstr.c"

size_t oracle_dstr_size(void)
{
	return sizeof(struct dstr);
}
size_t oracle_dstr_align(void)
{
	return _Alignof(struct dstr);
}
size_t oracle_dstr_offset_array(void)
{
	return offsetof(struct dstr, array);
}
size_t oracle_dstr_offset_len(void)
{
	return offsetof(struct dstr, len);
}
size_t oracle_dstr_offset_capacity(void)
{
	return offsetof(struct dstr, capacity);
}
