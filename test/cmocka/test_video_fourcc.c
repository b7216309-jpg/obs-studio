#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <media-io/video-io.h>

/* Characterization of the exported function in media-io/video-fourcc.c. */

/* Little-endian packing, as video-fourcc.c's MAKE_FOURCC: the first
 * character is the low byte. */
static uint32_t fourcc(const char code[4])
{
	return (uint32_t)(uint8_t)code[0] | (uint32_t)(uint8_t)code[1] << 8 | (uint32_t)(uint8_t)code[2] << 16 |
	       (uint32_t)(uint8_t)code[3] << 24;
}

static void assert_codes(const char *const *codes, size_t count, enum video_format expected)
{
	for (size_t i = 0; i < count; i++) {
		if (video_format_from_fourcc(fourcc(codes[i])) != expected)
			fail_msg("fourcc '%.4s'", codes[i]);
	}
}

static void test_uyvy_codes(void **state)
{
	UNUSED_PARAMETER(state);

	static const char *const codes[] = {"UYVY", "HDYC", "UYNV", "UYNY", "uyv1", "2vuy", "2Vuy"};
	assert_codes(codes, sizeof(codes) / sizeof(codes[0]), VIDEO_FORMAT_UYVY);
}

static void test_yuy2_codes(void **state)
{
	UNUSED_PARAMETER(state);

	static const char *const codes[] = {"YUY2", "Y422", "V422", "VYUY", "YUNV", "yuv2", "yuvs"};
	assert_codes(codes, sizeof(codes) / sizeof(codes[0]), VIDEO_FORMAT_YUY2);
}

static void test_yvyu_and_y800_codes(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(video_format_from_fourcc(fourcc("YVYU")), VIDEO_FORMAT_YVYU);
	assert_int_equal(video_format_from_fourcc(fourcc("Y800")), VIDEO_FORMAT_Y800);
}

static void test_unknown_codes(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(video_format_from_fourcc(0), VIDEO_FORMAT_NONE);
	assert_int_equal(video_format_from_fourcc(UINT32_MAX), VIDEO_FORMAT_NONE);

	/* Characterized, not endorsed: fourccs of other formats the enum has
	 * are not mapped. */
	static const char *const codes[] = {"I420", "NV12", "RGBA", "BGRA", "I444", "P010", "v210", "AYUV"};
	assert_codes(codes, sizeof(codes) / sizeof(codes[0]), VIDEO_FORMAT_NONE);
}

static void test_case_sensitive(void **state)
{
	UNUSED_PARAMETER(state);

	/* Only the exact spellings in the table match. */
	static const char *const codes[] = {"uyvy", "yuy2", "yvyu", "y800", "2VUY", "UYV1", "YUV2", "YUVS"};
	assert_codes(codes, sizeof(codes) / sizeof(codes[0]), VIDEO_FORMAT_NONE);
}

static void test_byte_order(void **state)
{
	UNUSED_PARAMETER(state);

	/* The first character is the low byte. Read the other way round,
	 * "UYVY" is "YVYU" and "YUY2" is "2YUY". */
	assert_int_equal(video_format_from_fourcc(0x55595659u), VIDEO_FORMAT_YVYU);
	assert_int_equal(video_format_from_fourcc(0x59555932u), VIDEO_FORMAT_NONE);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_uyvy_codes),          cmocka_unit_test(test_yuy2_codes),
		cmocka_unit_test(test_yvyu_and_y800_codes), cmocka_unit_test(test_unknown_codes),
		cmocka_unit_test(test_case_sensitive),      cmocka_unit_test(test_byte_order),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
