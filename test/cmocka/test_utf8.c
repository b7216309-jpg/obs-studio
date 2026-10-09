#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <wchar.h>
#include <string.h>

#include <util/c99defs.h>
#include <util/utf8.h>

static void test_utf8_ascii_round_trip(void **state)
{
	UNUSED_PARAMETER(state);

	const char *in = "hello";
	wchar_t out[8] = {0};

	assert_int_equal(utf8_to_wchar(in, 0, NULL, 0, 0), 5);
	assert_int_equal(utf8_to_wchar(in, 0, out, 8, 0), 5);
	assert_int_equal(wmemcmp(out, L"hello", 5), 0);

	char back[8] = {0};
	assert_int_equal(wchar_to_utf8(out, 0, NULL, 0, 0), 5);
	assert_int_equal(wchar_to_utf8(out, 0, back, 8, 0), 5);
	assert_string_equal(back, "hello");
}

static void test_utf8_to_wchar_multibyte(void **state)
{
	UNUSED_PARAMETER(state);

	wchar_t out[4] = {0};

	assert_int_equal(utf8_to_wchar("\xc3\xa9", 2, out, 4, 0), 1);
	assert_int_equal((uint32_t)out[0], 0xE9);

	assert_int_equal(utf8_to_wchar("\xe2\x82\xac", 3, out, 4, 0), 1);
	assert_int_equal((uint32_t)out[0], 0x20AC);

	assert_int_equal(utf8_to_wchar("\xf0\x9f\x98\x80", 4, out, 4, 0), 1);
	assert_int_equal((uint32_t)out[0], 0x1F600);
}

static void test_utf8_wchar_to_utf8_multibyte(void **state)
{
	UNUSED_PARAMETER(state);

	char out[8];
	wchar_t in[1];

	in[0] = 0xE9;
	memset(out, 0, sizeof(out));
	assert_int_equal(wchar_to_utf8(in, 1, out, 8, 0), 2);
	assert_memory_equal(out, "\xc3\xa9", 2);

	in[0] = 0x20AC;
	memset(out, 0, sizeof(out));
	assert_int_equal(wchar_to_utf8(in, 1, out, 8, 0), 3);
	assert_memory_equal(out, "\xe2\x82\xac", 3);

	in[0] = 0x1F600;
	memset(out, 0, sizeof(out));
	assert_int_equal(wchar_to_utf8(in, 1, out, 8, 0), 4);
	assert_memory_equal(out, "\xf0\x9f\x98\x80", 4);
}

static void test_utf8_errors(void **state)
{
	UNUSED_PARAMETER(state);

	wchar_t out[8];
	char cout[8];
	wchar_t sur[1] = {0xD800};

	/* forbidden octets */
	assert_int_equal(utf8_to_wchar("\xc0\x80", 2, out, 8, 0), 0);
	assert_int_equal(utf8_to_wchar("\xff", 1, out, 8, 0), 0);
	/* truncated sequence */
	assert_int_equal(utf8_to_wchar("\xe2\x82", 2, out, 8, 0), 0);
	/* bad continuation */
	assert_int_equal(utf8_to_wchar("\xe2\x41\x41", 3, out, 8, 0), 0);
	/* surrogate */
	assert_int_equal(wchar_to_utf8(sur, 1, cout, 8, 0), 0);
}

static void test_utf8_ignore_error(void **state)
{
	UNUSED_PARAMETER(state);

	wchar_t out[8] = {0};
	char cout[8] = {0};
	wchar_t in[3] = {0x61, 0xD800, 0x62};

	assert_int_equal(utf8_to_wchar("a\xff"
				       "b",
				       3, out, 8, UTF8_IGNORE_ERROR),
			 2);
	assert_int_equal(wmemcmp(out, L"ab", 2), 0);

	assert_int_equal(wchar_to_utf8(in, 3, cout, 8, UTF8_IGNORE_ERROR), 2);
	assert_memory_equal(cout, "ab", 2);
}

static void test_utf8_skip_bom(void **state)
{
	UNUSED_PARAMETER(state);

	wchar_t out[8] = {0};

	assert_int_equal(utf8_to_wchar("\xef\xbb\xbf"
				       "x",
				       4, out, 8, UTF8_SKIP_BOM),
			 1);
	assert_int_equal((uint32_t)out[0], (uint32_t)L'x');

	memset(out, 0, sizeof(out));
	assert_int_equal(utf8_to_wchar("\xef\xbb\xbf"
				       "x",
				       4, out, 8, 0),
			 2);
	assert_int_equal((uint32_t)out[0], 0xFEFF);
	assert_int_equal((uint32_t)out[1], (uint32_t)L'x');
}

static void test_utf8_invalid_arguments(void **state)
{
	UNUSED_PARAMETER(state);

	wchar_t out[8];
	char cout[8];

	assert_int_equal(utf8_to_wchar(NULL, 0, out, 8, 0), 0);
	assert_int_equal(utf8_to_wchar("abc", 3, out, 0, 0), 0);
	/* output buffer too small */
	assert_int_equal(utf8_to_wchar("abc", 0, out, 2, 0), 0);

	assert_int_equal(wchar_to_utf8(NULL, 0, cout, 8, 0), 0);
	assert_int_equal(wchar_to_utf8(L"abc", 3, cout, 0, 0), 0);
	assert_int_equal(wchar_to_utf8(L"abc", 0, cout, 2, 0), 0);
}

static void test_utf8_embedded_nul(void **state)
{
	UNUSED_PARAMETER(state);

	wchar_t out[8] = {0};

	/* insize 0 stops at NUL */
	assert_int_equal(utf8_to_wchar("a\0b", 0, NULL, 0, 0), 1);
	/* insize > 0 translates NUL as a regular symbol */
	assert_int_equal(utf8_to_wchar("a\0b", 3, out, 8, 0), 3);
	assert_int_equal((uint32_t)out[0], (uint32_t)L'a');
	assert_int_equal((uint32_t)out[1], 0);
	assert_int_equal((uint32_t)out[2], (uint32_t)L'b');
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_utf8_ascii_round_trip),
		cmocka_unit_test(test_utf8_to_wchar_multibyte),
		cmocka_unit_test(test_utf8_wchar_to_utf8_multibyte),
		cmocka_unit_test(test_utf8_errors),
		cmocka_unit_test(test_utf8_ignore_error),
		cmocka_unit_test(test_utf8_skip_bom),
		cmocka_unit_test(test_utf8_invalid_arguments),
		cmocka_unit_test(test_utf8_embedded_nul),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
