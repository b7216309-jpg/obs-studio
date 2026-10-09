#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <util/crc32.h>

static void test_crc32_check_value(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal((uint32_t)calc_crc32(0, "123456789", 9), (uint32_t)0xCBF43926);
}

static void test_crc32_empty(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal((uint32_t)calc_crc32(0, "", 0), (uint32_t)0);
	/* size 0 returns the crc unchanged */
	assert_int_equal((uint32_t)calc_crc32(0x12345678, "", 0), (uint32_t)0x12345678);
}

static void test_crc32_single_bytes(void **state)
{
	UNUSED_PARAMETER(state);

	uint8_t z = 0;

	assert_int_equal((uint32_t)calc_crc32(0, "a", 1), (uint32_t)0xE8B7BE43);
	assert_int_equal((uint32_t)calc_crc32(0, &z, 1), (uint32_t)0xD202EF8D);
}

static void test_crc32_fox(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal((uint32_t)calc_crc32(0, "The quick brown fox jumps over the lazy dog", 43),
			 (uint32_t)0x414FA339);
}

static void test_crc32_chaining(void **state)
{
	UNUSED_PARAMETER(state);

	uint32_t part = calc_crc32(0, "1234", 4);

	assert_int_equal((uint32_t)calc_crc32(part, "56789", 5), (uint32_t)0xCBF43926);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_crc32_check_value),  cmocka_unit_test(test_crc32_empty),
		cmocka_unit_test(test_crc32_single_bytes), cmocka_unit_test(test_crc32_fox),
		cmocka_unit_test(test_crc32_chaining),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
