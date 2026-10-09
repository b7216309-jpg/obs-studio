#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <math.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <graphics/vec2.h>

/* Characterization of the exported functions in graphics/vec2.c. */

static void test_vec2_abs(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v, dst;

	vec2_set(&v, -1.5f, 2.0f);
	vec2_abs(&dst, &v);
	assert_true(dst.x == 1.5f);
	assert_true(dst.y == 2.0f);
}

static void test_vec2_abs_negative_zero(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v, dst;

	vec2_set(&v, -0.0f, 0.0f);
	vec2_abs(&dst, &v);
	assert_true(dst.x == 0.0f);
	assert_true(dst.y == 0.0f);
	assert_true(signbit(dst.x) == 0);
	assert_true(signbit(dst.y) == 0);
}

static void test_vec2_floor_ceil(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v, dst;

	vec2_set(&v, 1.5f, -1.5f);

	vec2_floor(&dst, &v);
	assert_true(dst.x == 1.0f);
	assert_true(dst.y == -2.0f);

	vec2_ceil(&dst, &v);
	assert_true(dst.x == 2.0f);
	assert_true(dst.y == -1.0f);
}

static void test_vec2_floor_ceil_integral(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v, dst;

	vec2_set(&v, 3.0f, -4.0f);

	vec2_floor(&dst, &v);
	assert_true(dst.x == 3.0f);
	assert_true(dst.y == -4.0f);

	vec2_ceil(&dst, &v);
	assert_true(dst.x == 3.0f);
	assert_true(dst.y == -4.0f);
}

static void test_vec2_close(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 a, b;

	vec2_set(&a, 1.25f, -3.5f);
	vec2_set(&b, 1.25f, -3.5f);
	assert_int_equal(vec2_close(&a, &b, 0.0f), 1);
	assert_int_equal(vec2_close(&a, &b, 0.5f), 1);

	/* a difference exactly equal to epsilon is close (close_float uses <=) */
	vec2_set(&a, 0.0f, 0.0f);
	vec2_set(&b, 0.5f, 0.0f);
	assert_int_equal(vec2_close(&a, &b, 0.5f), 1);

	/* a larger difference in y only is not close */
	vec2_set(&b, 0.0f, 0.75f);
	assert_int_equal(vec2_close(&a, &b, 0.5f), 0);
}

static void test_vec2_close_nan(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 a, b;

	vec2_set(&a, 0.0f, 0.0f);

	vec2_set(&b, NAN, 0.0f);
	assert_int_equal(vec2_close(&a, &b, 1.0f), 0);
	assert_int_equal(vec2_close(&b, &a, 1.0f), 0);

	vec2_set(&b, 0.0f, NAN);
	assert_int_equal(vec2_close(&a, &b, 1.0f), 0);
	assert_int_equal(vec2_close(&b, &a, 1.0f), 0);
}

static void test_vec2_norm(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v, dst;

	vec2_set(&v, 3.0f, 4.0f);
	vec2_norm(&dst, &v);
	assert_true(fabsf(dst.x - 0.6f) <= 1e-6f);
	assert_true(fabsf(dst.y - 0.8f) <= 1e-6f);
}

static void test_vec2_norm_aliased(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v;

	vec2_set(&v, 3.0f, 4.0f);
	vec2_norm(&v, &v);
	assert_true(fabsf(v.x - 0.6f) <= 1e-6f);
	assert_true(fabsf(v.y - 0.8f) <= 1e-6f);
}

static void test_vec2_norm_zero_leaves_dst(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v, dst;

	/* characterized, not endorsed: a zero vector leaves dst unchanged */
	vec2_set(&v, 0.0f, 0.0f);
	vec2_set(&dst, 7.0f, 9.0f);
	vec2_norm(&dst, &v);
	assert_true(dst.x == 7.0f);
	assert_true(dst.y == 9.0f);
}

static void test_vec2_norm_nan_leaves_dst(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec2 v, dst;

	/* characterized, not endorsed: len is NaN, so (len > 0.0f) is false and dst is unchanged */
	vec2_set(&v, NAN, 0.0f);
	vec2_set(&dst, 7.0f, 9.0f);
	vec2_norm(&dst, &v);
	assert_true(dst.x == 7.0f);
	assert_true(dst.y == 9.0f);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_vec2_abs),
		cmocka_unit_test(test_vec2_abs_negative_zero),
		cmocka_unit_test(test_vec2_floor_ceil),
		cmocka_unit_test(test_vec2_floor_ceil_integral),
		cmocka_unit_test(test_vec2_close),
		cmocka_unit_test(test_vec2_close_nan),
		cmocka_unit_test(test_vec2_norm),
		cmocka_unit_test(test_vec2_norm_aliased),
		cmocka_unit_test(test_vec2_norm_zero_leaves_dst),
		cmocka_unit_test(test_vec2_norm_nan_leaves_dst),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
