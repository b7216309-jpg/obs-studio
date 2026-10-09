/* Test-only oracle: the original libobs/util/lexer.c, unmodified, with every
 * global symbol renamed to oracle_* so it can link next to the Rust
 * implementation, plus the struct layouts as the C compiler sees them. Its
 * dstr_catf and bmemdup come from oracle/test_dstr.c and test_bmem.c. */
#include <stddef.h>

#define strref_cmp oracle_strref_cmp
#define strref_cmpi oracle_strref_cmpi
#define strref_cmp_strref oracle_strref_cmp_strref
#define strref_cmpi_strref oracle_strref_cmpi_strref
#define valid_int_str oracle_valid_int_str
#define valid_float_str oracle_valid_float_str
#define error_data_add oracle_error_data_add
#define error_data_buildstring oracle_error_data_buildstring
#define lexer_getbasetoken oracle_lexer_getbasetoken
#define lexer_getstroffset oracle_lexer_getstroffset

#include "util/lexer.c"

#define LAYOUT(type)                               \
	size_t oracle_##type##_size(void)          \
	{                                          \
		return sizeof(struct type);        \
	}                                          \
	size_t oracle_##type##_align(void)         \
	{                                          \
		return _Alignof(struct type);      \
	}
#define OFFSET(type, field)                              \
	size_t oracle_##type##_offset_##field(void)      \
	{                                                \
		return offsetof(struct type, field);     \
	}

LAYOUT(strref)
OFFSET(strref, array)
OFFSET(strref, len)

LAYOUT(base_token)
OFFSET(base_token, text)
OFFSET(base_token, type)
OFFSET(base_token, passed_whitespace)

LAYOUT(error_item)
OFFSET(error_item, error)
OFFSET(error_item, file)
OFFSET(error_item, row)
OFFSET(error_item, column)
OFFSET(error_item, level)

LAYOUT(error_data)
OFFSET(error_data, errors)

LAYOUT(lexer)
OFFSET(lexer, text)
OFFSET(lexer, offset)

size_t oracle_base_token_type_size(void)
{
	return sizeof(enum base_token_type);
}
