#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <obs-nal.h>

/* Characterization of obs_nal_find_startcode in obs-nal.c. It returns the
 * first 00 00 01 start code, moved back one byte when a zero precedes it
 * (a 4-byte 00 00 00 01 code), or `end` when there is none. */

static size_t find(const uint8_t *data, size_t size)
{
	return (size_t)(obs_nal_find_startcode(data, data + size) - data);
}

static void test_nal_three_byte_code_at_start(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {0, 0, 1, 0x65, 0xaa};
	assert_int_equal(find(data, sizeof(data)), 0);
}

/* The 4-byte form is found at its first zero, including at the very start. */
static void test_nal_four_byte_code_at_start(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {0, 0, 0, 1, 0x67, 0xaa};
	assert_int_equal(find(data, sizeof(data)), 0);
}

static void test_nal_code_after_payload(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t three[] = {0x11, 0x22, 0x33, 0, 0, 1, 0x41, 0x42};
	assert_int_equal(find(three, sizeof(three)), 3);

	const uint8_t four[] = {0x11, 0x22, 0, 0, 0, 1, 0x41, 0x42};
	assert_int_equal(find(four, sizeof(four)), 2);
}

/* Only one preceding zero is folded in: the zero run starts at index 1, but
 * the result is 2, the zero just before 00 00 01. */
static void test_nal_long_zero_run(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {0x11, 0, 0, 0, 0, 1, 0x41, 0x42};
	assert_int_equal(find(data, sizeof(data)), 2);
}

static void test_nal_no_code_returns_end(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {0x11, 0, 0x22, 0, 0, 0x33, 1, 0};
	assert_int_equal(find(data, sizeof(data)), sizeof(data));
	assert_int_equal(find(data, 0), 0);
	assert_int_equal(find(data, 2), 2);
}

/* A start code needs at least one byte after it: one in the last three bytes
 * is not reported. */
static void test_nal_code_must_not_end_the_buffer(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {0x11, 0x22, 0, 0, 1, 0x65};
	assert_int_equal(find(data, sizeof(data)), 2);
	assert_int_equal(find(data, sizeof(data) - 1), sizeof(data) - 1);
}

/* The search reads a word at a time after an alignment prologue; the result
 * must not depend on where the buffer starts. */
static void test_nal_every_alignment(void **state)
{
	UNUSED_PARAMETER(state);

	uint8_t buf[64];

	for (size_t shift = 0; shift < 8; shift++) {
		for (size_t pos = 0; pos + 4 <= 40; pos++) {
			uint8_t *data = buf + shift;
			memset(buf, 0x11, sizeof(buf));
			data[pos] = 0;
			data[pos + 1] = 0;
			data[pos + 2] = 1;
			assert_int_equal(find(data, 40), pos);
		}
	}
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_nal_three_byte_code_at_start),
		cmocka_unit_test(test_nal_four_byte_code_at_start),
		cmocka_unit_test(test_nal_code_after_payload),
		cmocka_unit_test(test_nal_long_zero_run),
		cmocka_unit_test(test_nal_no_code_returns_end),
		cmocka_unit_test(test_nal_code_must_not_end_the_buffer),
		cmocka_unit_test(test_nal_every_alignment),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
