#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <math.h>
#include <stdio.h>
#include <string.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <media-io/video-io.h>

/* Characterization of the exported functions in media-io/video-matrices.c. */

/* Matrix entries are compared with a tolerance: a compiler that fuses
 * multiply-adds may change their last bits. Range limits are single
 * divisions and are compared exactly. */
#define MATRIX_EPSILON 1e-5f

struct params {
	bool success;
	float matrix[16];
	float range_min[3];
	float range_max[3];
};

static struct params get_for_format(enum video_colorspace cs, enum video_range_type range, enum video_format format)
{
	struct params p;
	p.success = video_format_get_parameters_for_format(cs, range, format, p.matrix, p.range_min, p.range_max);
	return p;
}

static struct params get(enum video_colorspace cs, enum video_range_type range)
{
	struct params p;
	p.success = video_format_get_parameters(cs, range, p.matrix, p.range_min, p.range_max);
	return p;
}

static void assert_matrix(const float actual[16], const float expected[16])
{
	for (size_t i = 0; i < 16; i++) {
		if (fabsf(actual[i] - expected[i]) > MATRIX_EPSILON) {
			/* fail_msg() expands to cm_print_error(), which the
			 * Windows cmocka library does not export. */
			fprintf(stderr, "matrix[%zu]: %.9g, expected %.9g\n", i, actual[i], expected[i]);
			fail();
		}
	}
}

static void assert_floats_equal(const float *actual, const float *expected, size_t count)
{
	for (size_t i = 0; i < count; i++) {
		if (actual[i] != expected[i]) {
			fprintf(stderr, "[%zu]: %.9g, expected %.9g\n", i, actual[i], expected[i]);
			fail();
		}
	}
}

static void assert_params_equal(const struct params *actual, const struct params *expected)
{
	assert_int_equal(actual->success, expected->success);
	assert_memory_equal(actual->matrix, expected->matrix, sizeof(actual->matrix));
	assert_memory_equal(actual->range_min, expected->range_min, sizeof(actual->range_min));
	assert_memory_equal(actual->range_max, expected->range_max, sizeof(actual->range_max));
}

static void assert_partial_range(const struct params *p, float range)
{
	const float max = range - 1.f;
	const float min_expected[3] = {16.f * range / 256.f / max, 16.f * range / 256.f / max,
				       16.f * range / 256.f / max};
	const float max_expected[3] = {235.f * range / 256.f / max, 240.f * range / 256.f / max,
				       240.f * range / 256.f / max};
	assert_floats_equal(p->range_min, min_expected, 3);
	assert_floats_equal(p->range_max, max_expected, 3);
}

static void test_709_partial_8bit(void **state)
{
	UNUSED_PARAMETER(state);

	static const float expected[16] = {
		1.16438353f, 0.f,          1.79274106f,   -0.972945154f, /* R */
		1.16438353f, -0.21324861f, -0.532909334f, 0.301482677f,  /* G */
		1.16438353f, 2.11240172f,  0.f,           -1.13340223f,  /* B */
		0.f,         0.f,          0.f,           1.f,
	};

	struct params p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_NV12);
	assert_true(p.success);
	assert_matrix(p.matrix, expected);
	assert_partial_range(&p, 256.f);
}

static void test_709_full_8bit(void **state)
{
	UNUSED_PARAMETER(state);

	static const float expected[16] = {
		1.f, 0.f,           1.57480001f,  -0.790487885f, /* R */
		1.f, -0.187324256f, -0.46812427f, 0.329009473f,  /* G */
		1.f, 1.8556f,       0.f,          -0.931438506f, /* B */
		0.f, 0.f,           0.f,          1.f,
	};
	static const float zeros[3] = {0.f, 0.f, 0.f};
	static const float ones[3] = {1.f, 1.f, 1.f};

	struct params p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_FULL, VIDEO_FORMAT_NV12);
	assert_true(p.success);
	assert_matrix(p.matrix, expected);
	assert_floats_equal(p.range_min, zeros, 3);
	assert_floats_equal(p.range_max, ones, 3);
}

static void test_601_partial_8bit(void **state)
{
	UNUSED_PARAMETER(state);

	static const float expected[16] = {
		1.16438353f, 0.f,           1.59602666f,   -0.874202192f, /* R */
		1.16438353f, -0.391762227f, -0.812967539f, 0.531667709f,  /* G */
		1.16438353f, 2.01723194f,   0.f,           -1.08563066f,  /* B */
		0.f,         0.f,           0.f,           1.f,
	};

	struct params p = get_for_format(VIDEO_CS_601, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_NV12);
	assert_true(p.success);
	assert_matrix(p.matrix, expected);
	assert_partial_range(&p, 256.f);
}

static void test_2100_pq_partial_10bit(void **state)
{
	UNUSED_PARAMETER(state);

	static const float expected[16] = {
		1.16780818f, 0.f,           1.68361139f,   -0.915687978f, /* R */
		1.16780818f, -0.187877059f, -0.652337372f, 0.347458512f,  /* G */
		1.16780818f, 2.14807153f,   0.f,           -1.14814496f,  /* B */
		0.f,         0.f,           0.f,           1.f,
	};

	struct params p = get_for_format(VIDEO_CS_2100_PQ, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_P010);
	assert_true(p.success);
	assert_matrix(p.matrix, expected);
	assert_partial_range(&p, 1024.f);
}

static void test_709_partial_12bit_and_16bit(void **state)
{
	UNUSED_PARAMETER(state);

	static const float expected_12[16] = {
		1.16866434f, 0.f,          1.79933202f,   -0.972945154f, /* R */
		1.16866434f, -0.21403262f, -0.534868479f, 0.301482648f,  /* G */
		1.16866434f, 2.12016797f,  0.f,           -1.13340223f,  /* B */
		0.f,         0.f,          0.f,           1.f,
	};
	static const float expected_16[16] = {
		1.16893196f, 0.f,           1.79974389f,   -0.972945035f, /* R */
		1.16893196f, -0.214081615f, -0.534990966f, 0.301482648f,  /* G */
		1.16893196f, 2.12065339f,   0.f,           -1.13340223f,  /* B */
		0.f,         0.f,           0.f,           1.f,
	};

	struct params p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_I412);
	assert_true(p.success);
	assert_matrix(p.matrix, expected_12);
	assert_partial_range(&p, 4096.f);

	p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_P216);
	assert_true(p.success);
	assert_matrix(p.matrix, expected_16);
	assert_partial_range(&p, 65536.f);
}

static void test_colorspace_aliases(void **state)
{
	UNUSED_PARAMETER(state);

	static const enum video_range_type ranges[] = {VIDEO_RANGE_PARTIAL, VIDEO_RANGE_FULL};

	for (size_t i = 0; i < sizeof(ranges) / sizeof(ranges[0]); i++) {
		/* DEFAULT and sRGB use the Rec. 709 matrices, HLG the PQ ones. */
		struct params cs_709 = get_for_format(VIDEO_CS_709, ranges[i], VIDEO_FORMAT_NV12);
		struct params cs_default = get_for_format(VIDEO_CS_DEFAULT, ranges[i], VIDEO_FORMAT_NV12);
		struct params cs_srgb = get_for_format(VIDEO_CS_SRGB, ranges[i], VIDEO_FORMAT_NV12);
		assert_params_equal(&cs_default, &cs_709);
		assert_params_equal(&cs_srgb, &cs_709);

		struct params cs_pq = get_for_format(VIDEO_CS_2100_PQ, ranges[i], VIDEO_FORMAT_P010);
		struct params cs_hlg = get_for_format(VIDEO_CS_2100_HLG, ranges[i], VIDEO_FORMAT_P010);
		assert_params_equal(&cs_hlg, &cs_pq);
	}
}

static void test_unknown_colorspace(void **state)
{
	UNUSED_PARAMETER(state);

	static const int values[] = {-1, VIDEO_CS_2100_HLG + 1, 100};

	for (size_t i = 0; i < sizeof(values) / sizeof(values[0]); i++) {
		float matrix[16], range_min[3], range_max[3];
		memset(matrix, 0x5a, sizeof(matrix));
		memset(range_min, 0x5a, sizeof(range_min));
		memset(range_max, 0x5a, sizeof(range_max));

		float matrix_before[16], range_min_before[3], range_max_before[3];
		memcpy(matrix_before, matrix, sizeof(matrix));
		memcpy(range_min_before, range_min, sizeof(range_min));
		memcpy(range_max_before, range_max, sizeof(range_max));

		/* Returns false and writes nothing. */
		assert_false(video_format_get_parameters_for_format((enum video_colorspace)values[i],
								    VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_NV12, matrix,
								    range_min, range_max));
		assert_false(video_format_get_parameters((enum video_colorspace)values[i], VIDEO_RANGE_PARTIAL, matrix,
							 range_min, range_max));
		assert_memory_equal(matrix, matrix_before, sizeof(matrix));
		assert_memory_equal(range_min, range_min_before, sizeof(range_min));
		assert_memory_equal(range_max, range_max_before, sizeof(range_max));
	}
}

static void test_null_range_pointers(void **state)
{
	UNUSED_PARAMETER(state);

	struct params expected = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_NV12);

	float matrix[16];
	assert_true(video_format_get_parameters_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_NV12, matrix,
							   NULL, NULL));
	assert_memory_equal(matrix, expected.matrix, sizeof(matrix));

	float range_min[3];
	assert_true(video_format_get_parameters(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, matrix, range_min, NULL));
	assert_memory_equal(range_min, expected.range_min, sizeof(range_min));

	float range_max[3];
	assert_true(video_format_get_parameters(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, matrix, NULL, range_max));
	assert_memory_equal(range_max, expected.range_max, sizeof(range_max));
}

static void test_only_full_range_is_full(void **state)
{
	UNUSED_PARAMETER(state);

	struct params partial = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, VIDEO_FORMAT_NV12);
	struct params full = get_for_format(VIDEO_CS_709, VIDEO_RANGE_FULL, VIDEO_FORMAT_NV12);
	assert_memory_not_equal(partial.matrix, full.matrix, sizeof(partial.matrix));

	/* Characterized, not endorsed: VIDEO_RANGE_DEFAULT and values
	 * outside the enum are partial range. */
	static const int values[] = {VIDEO_RANGE_DEFAULT, -1, VIDEO_RANGE_FULL + 1, 100};
	for (size_t i = 0; i < sizeof(values) / sizeof(values[0]); i++) {
		struct params p = get_for_format(VIDEO_CS_709, (enum video_range_type)values[i], VIDEO_FORMAT_NV12);
		assert_params_equal(&p, &partial);
	}
}

static void test_format_bit_depths(void **state)
{
	UNUSED_PARAMETER(state);

	static const enum video_format formats_10[] = {VIDEO_FORMAT_I010, VIDEO_FORMAT_P010, VIDEO_FORMAT_I210,
						       VIDEO_FORMAT_V210, VIDEO_FORMAT_R10L};
	static const enum video_format formats_12[] = {VIDEO_FORMAT_I412, VIDEO_FORMAT_YA2L};
	static const enum video_format formats_16[] = {VIDEO_FORMAT_P216, VIDEO_FORMAT_P416};
	/* Every other format, and values outside the enum, are 8 bits. */
	static const int formats_8[] = {
		VIDEO_FORMAT_NONE,     VIDEO_FORMAT_I420,
		VIDEO_FORMAT_NV12,     VIDEO_FORMAT_YVYU,
		VIDEO_FORMAT_YUY2,     VIDEO_FORMAT_UYVY,
		VIDEO_FORMAT_RGBA,     VIDEO_FORMAT_BGRA,
		VIDEO_FORMAT_BGRX,     VIDEO_FORMAT_Y800,
		VIDEO_FORMAT_I444,     VIDEO_FORMAT_BGR3,
		VIDEO_FORMAT_I422,     VIDEO_FORMAT_I40A,
		VIDEO_FORMAT_I42A,     VIDEO_FORMAT_YUVA,
		VIDEO_FORMAT_AYUV,     -1,
		VIDEO_FORMAT_R10L + 1, 100,
	};

	struct params p;
	for (size_t i = 0; i < sizeof(formats_10) / sizeof(formats_10[0]); i++) {
		p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, formats_10[i]);
		assert_partial_range(&p, 1024.f);
	}
	for (size_t i = 0; i < sizeof(formats_12) / sizeof(formats_12[0]); i++) {
		p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, formats_12[i]);
		assert_partial_range(&p, 4096.f);
	}
	for (size_t i = 0; i < sizeof(formats_16) / sizeof(formats_16[0]); i++) {
		p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, formats_16[i]);
		assert_partial_range(&p, 65536.f);
	}
	for (size_t i = 0; i < sizeof(formats_8) / sizeof(formats_8[0]); i++) {
		p = get_for_format(VIDEO_CS_709, VIDEO_RANGE_PARTIAL, (enum video_format)formats_8[i]);
		assert_partial_range(&p, 256.f);
	}
}

static void test_get_parameters_bit_depth(void **state)
{
	UNUSED_PARAMETER(state);

	static const enum video_range_type ranges[] = {VIDEO_RANGE_PARTIAL, VIDEO_RANGE_FULL};

	for (size_t i = 0; i < sizeof(ranges) / sizeof(ranges[0]); i++) {
		/* 10 bits for PQ and HLG, 8 bits otherwise. */
		static const enum video_colorspace cs_8[] = {VIDEO_CS_DEFAULT, VIDEO_CS_601, VIDEO_CS_709,
							     VIDEO_CS_SRGB};
		for (size_t j = 0; j < sizeof(cs_8) / sizeof(cs_8[0]); j++) {
			struct params p = get(cs_8[j], ranges[i]);
			struct params expected = get_for_format(cs_8[j], ranges[i], VIDEO_FORMAT_NV12);
			assert_params_equal(&p, &expected);
		}

		static const enum video_colorspace cs_10[] = {VIDEO_CS_2100_PQ, VIDEO_CS_2100_HLG};
		for (size_t j = 0; j < sizeof(cs_10) / sizeof(cs_10[0]); j++) {
			struct params p = get(cs_10[j], ranges[i]);
			struct params expected = get_for_format(cs_10[j], ranges[i], VIDEO_FORMAT_P010);
			assert_params_equal(&p, &expected);
		}
	}
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_709_partial_8bit),
		cmocka_unit_test(test_709_full_8bit),
		cmocka_unit_test(test_601_partial_8bit),
		cmocka_unit_test(test_2100_pq_partial_10bit),
		cmocka_unit_test(test_709_partial_12bit_and_16bit),
		cmocka_unit_test(test_colorspace_aliases),
		cmocka_unit_test(test_unknown_colorspace),
		cmocka_unit_test(test_null_range_pointers),
		cmocka_unit_test(test_only_full_range_is_full),
		cmocka_unit_test(test_format_bit_depths),
		cmocka_unit_test(test_get_parameters_bit_depth),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
