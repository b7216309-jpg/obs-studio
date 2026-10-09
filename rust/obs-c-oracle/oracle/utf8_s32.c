/* Test-only oracle: the original libobs/util/utf8.c, unmodified, with its
 * global symbols renamed to oracle_s32_* so it can link next to the Rust
 * implementation.
 *
 * The Rust port replaces the non-Windows implementation, so that branch is
 * compiled on every platform (Windows included, where it is otherwise
 * unreachable) with wchar_t as int32_t: the wchar_t of x86 Linux and macOS (signed int). */
#include <stdint.h>
#include <wchar.h>

#undef _WIN32
#define wchar_t int32_t
#define utf8_to_wchar oracle_s32_utf8_to_wchar
#define wchar_to_utf8 oracle_s32_wchar_to_utf8

#include "util/utf8.c"
