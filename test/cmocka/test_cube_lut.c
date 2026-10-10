#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <graphics/half.h>
#include <util/bmem.h>
#include <util/dstr.h>
#include <util/platform.h>

#include "cube-lut.h"

/* load_cube_file from the Color Grade filter: RGBA half floats, sRGB
 * entries linearized, alpha 1.0.
 *
 * Regressions: the 1D loader allocated 4 * width^3 halves in uint32_t, so
 * a LUT_1D_SIZE of 1024 wrapped to bmalloc(0) and crashed, and sizes near
 * 1000 asked for gigabytes. Sizes past the .cube spec (1D: 65536, 3D: 256)
 * are rejected before allocating. */

#define CUBE_PATH "test_cube_lut.cube"
#define HALF_ONE 0x3c00

static void write_cube(const char *text)
{
	assert_true(os_quick_write_utf8_file(CUBE_PATH, text, strlen(text), false));
}

/* header, then `entries` lines alternating black and white */
static void write_cube_entries(const char *header, size_t entries)
{
	struct dstr text = {0};
	dstr_copy(&text, header);
	for (size_t i = 0; i < entries; i++)
		dstr_cat(&text, i % 2 ? "1.0 1.0 1.0\n" : "0.0 0.0 0.0\n");
	write_cube(text.array);
	dstr_free(&text);
}

static struct half *load(uint32_t *width, enum clut_dimension *dim, struct vec3 *min, struct vec3 *max)
{
	vec3_set(min, 0.0f, 0.0f, 0.0f);
	vec3_set(max, 1.0f, 1.0f, 1.0f);
	*width = 0;
	*dim = CLUT_3D;
	return load_cube_file(CUBE_PATH, width, min, max, dim);
}

static void expect_entry(const struct half *values, size_t i, uint16_t rgb)
{
	assert_int_equal(values[i * 4 + 0].u, rgb);
	assert_int_equal(values[i * 4 + 1].u, rgb);
	assert_int_equal(values[i * 4 + 2].u, rgb);
	assert_int_equal(values[i * 4 + 3].u, HALF_ONE);
}

static int teardown(void **state)
{
	UNUSED_PARAMETER(state);
	os_unlink(CUBE_PATH);
	return 0;
}

static void test_3d_lut(void **state)
{
	UNUSED_PARAMETER(state);

	uint32_t width;
	enum clut_dimension dim;
	struct vec3 min, max;

	write_cube_entries("TITLE \"test\"\n# comment\nLUT_3D_SIZE 2\n", 8);
	struct half *values = load(&width, &dim, &min, &max);
	assert_non_null(values);
	assert_int_equal(width, 2);
	assert_int_equal(dim, CLUT_3D);
	for (size_t i = 0; i < 8; i++)
		expect_entry(values, i, i % 2 ? HALF_ONE : 0);
	bfree(values);

	/* one entry short */
	write_cube_entries("LUT_3D_SIZE 2\n", 7);
	assert_null(load(&width, &dim, &min, &max));
	assert_int_equal(width, 0);
}

static void test_1d_lut_and_domain(void **state)
{
	UNUSED_PARAMETER(state);

	uint32_t width;
	enum clut_dimension dim;
	struct vec3 min, max;

	write_cube_entries("DOMAIN_MIN 0.0 0.0 0.0\nDOMAIN_MAX 2.0 4.0 8.0\nLUT_1D_SIZE 3\n", 3);
	struct half *values = load(&width, &dim, &min, &max);
	assert_non_null(values);
	assert_int_equal(width, 3);
	assert_int_equal(dim, CLUT_1D);
	assert_true(max.x == 2.0f && max.y == 4.0f && max.z == 8.0f);
	expect_entry(values, 0, 0);
	expect_entry(values, 1, HALF_ONE);
	expect_entry(values, 2, 0);
	bfree(values);

	/* an empty domain is refused */
	write_cube_entries("DOMAIN_MIN 1.0 0.0 0.0\nDOMAIN_MAX 1.0 1.0 1.0\nLUT_1D_SIZE 3\n", 3);
	assert_null(load(&width, &dim, &min, &max));

	/* no size line: no LUT */
	write_cube_entries("TITLE \"none\"\n", 8);
	assert_null(load(&width, &dim, &min, &max));
}

/* 4 * 1024^3 is 2^32: the old size wrapped to bmalloc(0). */
static void test_large_1d_lut(void **state)
{
	UNUSED_PARAMETER(state);

	uint32_t width;
	enum clut_dimension dim;
	struct vec3 min, max;

	write_cube_entries("LUT_1D_SIZE 1024\n", 1024);
	struct half *values = load(&width, &dim, &min, &max);
	assert_non_null(values);
	assert_int_equal(width, 1024);
	assert_int_equal(dim, CLUT_1D);
	expect_entry(values, 0, 0);
	expect_entry(values, 1023, HALF_ONE);
	bfree(values);
}

/* Sizes past the spec are refused before anything is allocated, whatever
 * follows them. */
static void test_oversized_luts(void **state)
{
	UNUSED_PARAMETER(state);

	uint32_t width;
	enum clut_dimension dim;
	struct vec3 min, max;

	write_cube_entries("LUT_3D_SIZE 1024\n", 4);
	assert_null(load(&width, &dim, &min, &max));
	write_cube_entries("LUT_3D_SIZE 257\n", 4);
	assert_null(load(&width, &dim, &min, &max));
	write_cube_entries("LUT_3D_SIZE 4294967295\n", 4);
	assert_null(load(&width, &dim, &min, &max));
	write_cube_entries("LUT_1D_SIZE 65537\n", 4);
	assert_null(load(&width, &dim, &min, &max));
	assert_int_equal(width, 0);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test_teardown(test_3d_lut, teardown),
		cmocka_unit_test_teardown(test_1d_lut_and_domain, teardown),
		cmocka_unit_test_teardown(test_large_1d_lut, teardown),
		cmocka_unit_test_teardown(test_oversized_luts, teardown),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
