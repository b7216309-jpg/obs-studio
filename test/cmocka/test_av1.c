#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <obs.h>
#include <obs-av1.h>

/* Characterization of obs-av1.c, plus regressions for OBUs that claim more
 * bytes than the buffer holds.
 *
 * OBU header byte: forbidden(1) type(4) extension_flag(1) has_size_field(1)
 * reserved(1). The first payload byte of a frame (header) OBU starts with
 * show_existing_frame(1) frame_type(2); frame_type 0 is a key frame. */

#define OBU(type, ext, has_size) (uint8_t)(((type) << 3) | ((ext) << 2) | ((has_size) << 1))

static void split(const uint8_t *data, size_t size, uint8_t **packet, size_t *packet_size, uint8_t **header,
		  size_t *header_size)
{
	obs_extract_av1_headers(data, size, packet, packet_size, header, header_size);
}

static void test_av1_keyframe(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t key[] = {OBU(OBS_OBU_TEMPORAL_DELIMITER, 0, 1),
			       0,
			       OBU(OBS_OBU_SEQUENCE_HEADER, 0, 1),
			       2,
			       0xaa,
			       0xbb,
			       OBU(OBS_OBU_FRAME, 0, 1),
			       2,
			       0x10,
			       0xcc};
	const uint8_t inter[] = {OBU(OBS_OBU_FRAME, 0, 1), 2, 0x30, 0xcc};
	const uint8_t shown[] = {OBU(OBS_OBU_FRAME_HEADER, 0, 1), 1, 0x80};
	const uint8_t header_key[] = {OBU(OBS_OBU_FRAME_HEADER, 0, 1), 1, 0x10};
	const uint8_t no_frame[] = {OBU(OBS_OBU_TEMPORAL_DELIMITER, 0, 1), 0, OBU(OBS_OBU_PADDING, 0, 1), 1, 0};

	assert_true(obs_av1_keyframe(key, sizeof(key)));
	assert_false(obs_av1_keyframe(inter, sizeof(inter)));
	/* show_existing_frame is never a key frame */
	assert_false(obs_av1_keyframe(shown, sizeof(shown)));
	assert_true(obs_av1_keyframe(header_key, sizeof(header_key)));
	assert_false(obs_av1_keyframe(no_frame, sizeof(no_frame)));
}

static void test_av1_extract_headers(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {OBU(OBS_OBU_TEMPORAL_DELIMITER, 0, 1),
				0,
				OBU(OBS_OBU_SEQUENCE_HEADER, 0, 1),
				1,
				0xaa,
				OBU(OBS_OBU_METADATA, 0, 1),
				1,
				0xbb,
				OBU(OBS_OBU_FRAME, 0, 1),
				1,
				0x10};
	uint8_t *packet, *header;
	size_t packet_size, header_size;

	split(data, sizeof(data), &packet, &packet_size, &header, &header_size);

	/* sequence header and metadata OBUs are headers; every OBU stays in
	 * the packet */
	assert_int_equal(header_size, 6);
	assert_memory_equal(header, data + 2, 6);
	assert_int_equal(packet_size, sizeof(data));
	assert_memory_equal(packet, data, sizeof(data));
	bfree(packet);
	bfree(header);

	/* empty input: both outputs NULL */
	split(data, 0, &packet, &packet_size, &header, &header_size);
	assert_null(packet);
	assert_null(header);
	assert_int_equal(packet_size + header_size, 0);
}

static void test_av1_metadata_obu(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t payload[] = {1, 2, 3};
	const uint8_t want[] = {OBU(OBS_OBU_METADATA, 0, 1), 5, METADATA_TYPE_ITUT_T35, 1, 2, 3, 0x80};
	uint8_t *out = NULL;
	size_t out_size = 0;

	metadata_obu(payload, sizeof(payload), &out, &out_size, METADATA_TYPE_ITUT_T35);
	assert_int_equal(out_size, sizeof(want));
	assert_memory_equal(out, want, sizeof(want));
	bfree(out);

	/* the ITU-T T.35 wrapper is the same OBU */
	metadata_obu_itu_t35(payload, sizeof(payload), &out, &out_size);
	assert_int_equal(out_size, sizeof(want));
	assert_memory_equal(out, want, sizeof(want));
	bfree(out);

	/* obu_size 1 + 200 + 1 = 202 takes two leb128 bytes */
	uint8_t big[200];
	memset(big, 0x5a, sizeof(big));
	metadata_obu(big, sizeof(big), &out, &out_size, METADATA_TYPE_USER_PRIVATE_6);
	assert_int_equal(out_size, 1 + 2 + 1 + sizeof(big) + 1);
	assert_int_equal(out[1], 0xca);
	assert_int_equal(out[2], 0x01);
	assert_int_equal(out[3], METADATA_TYPE_USER_PRIVATE_6);
	assert_int_equal(out[out_size - 1], 0x80);
	bfree(out);
}

/* An OBU whose size field claims more than is left is cut at the end of the
 * buffer instead of being read past it. */
static void test_av1_truncated_obu(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {OBU(OBS_OBU_FRAME, 0, 1), 16, 0x10, 0x11};
	uint8_t *packet, *header;
	size_t packet_size, header_size;

	split(data, sizeof(data), &packet, &packet_size, &header, &header_size);
	assert_int_equal(packet_size, sizeof(data));
	assert_memory_equal(packet, data, sizeof(data));
	assert_null(header);
	bfree(packet);

	assert_true(obs_av1_keyframe(data, sizeof(data)));
}

/* A 5-byte leb128 size (1 << 28) is computed in 64 bits, then cut at the end
 * of the buffer. */
static void test_av1_long_leb128(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {OBU(OBS_OBU_FRAME, 0, 1), 0x80, 0x80, 0x80, 0x80, 0x01, 0x10};
	uint8_t *packet, *header;
	size_t packet_size, header_size;

	split(data, sizeof(data), &packet, &packet_size, &header, &header_size);
	assert_int_equal(packet_size, sizeof(data));
	bfree(packet);

	assert_true(obs_av1_keyframe(data, sizeof(data)));
}

/* Without a size field the OBU is everything after its header, extension
 * byte included (AV1 spec 5.3.1: sz - 1 - obu_extension_flag). */
static void test_av1_extension_without_size(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {OBU(OBS_OBU_FRAME, 1, 0), 0x00, 0x10, 0x11};
	uint8_t *packet, *header;
	size_t packet_size, header_size;

	split(data, sizeof(data), &packet, &packet_size, &header, &header_size);
	assert_int_equal(packet_size, sizeof(data));
	bfree(packet);

	assert_true(obs_av1_keyframe(data, sizeof(data)));
}

/* An OBU header announcing an extension byte and a size field, with nothing
 * after it, is a 1-byte OBU. */
static void test_av1_header_only_at_end(void **state)
{
	UNUSED_PARAMETER(state);

	const uint8_t data[] = {OBU(OBS_OBU_FRAME, 0, 1), 1, 0x10, OBU(OBS_OBU_FRAME, 1, 1)};
	uint8_t *packet, *header;
	size_t packet_size, header_size;

	split(data, sizeof(data), &packet, &packet_size, &header, &header_size);
	assert_int_equal(packet_size, sizeof(data));
	assert_memory_equal(packet, data, sizeof(data));
	bfree(packet);

	const uint8_t alone[] = {OBU(OBS_OBU_FRAME, 1, 1)};
	assert_false(obs_av1_keyframe(alone, sizeof(alone)));
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_av1_keyframe),           cmocka_unit_test(test_av1_extract_headers),
		cmocka_unit_test(test_av1_metadata_obu),       cmocka_unit_test(test_av1_truncated_obu),
		cmocka_unit_test(test_av1_long_leb128),        cmocka_unit_test(test_av1_extension_without_size),
		cmocka_unit_test(test_av1_header_only_at_end),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
