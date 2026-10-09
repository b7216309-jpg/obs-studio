#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <obs-avc.h>
#include <util/bmem.h>

/*
 * A High-profile SPS truncated right after profile/constraints/level: every
 * Exp-Golomb read (id, chroma_format_idc, bit depths) runs past the end of
 * the buffer, where the bitstream reader returns 0 bits. get_ue_golomb then
 * counts 32 leading zeros and evaluated `1 << 32` (undefined behavior, #11).
 * Built with -fsanitize=undefined -fno-sanitize-recover=undefined where
 * available, so the UB aborts the test.
 */
static void avc_header_truncated_high_sps(void **state)
{
	UNUSED_PARAMETER(state);

	static const uint8_t data[] = {
		0x00, 0x00, 0x00, 0x01, 0x67, 0x64, 0x00, 0x1f, /* SPS: profile_idc 100 (High), level 3.1 */
		0x00, 0x00, 0x00, 0x01, 0x68, 0xee, 0x3c, 0x80, /* PPS */
	};
	const size_t sps_size = 4, pps_size = 4;

	uint8_t *header = NULL;
	size_t size = obs_parse_avc_header(&header, data, sizeof(data));

	/* avcC: version, profile/compat/level, length size, SPS count (6 bytes),
	 * SPS length + SPS, PPS count + length + PPS, then 4 bytes of
	 * High-profile extension data. */
	assert_int_equal(size, 6 + 2 + sps_size + 1 + 2 + pps_size + 4);
	assert_non_null(header);
	assert_int_equal(header[0], 0x01);
	assert_int_equal(header[1], 0x64);
	assert_memory_equal(header + 8, data + 4, sps_size);

	bfree(header);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(avc_header_truncated_high_sps),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
