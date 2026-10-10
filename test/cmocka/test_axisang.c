#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <math.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <graphics/axisang.h>
#include <graphics/quat.h>

/* Characterization of the exported function in graphics/axisang.c. */

static bool close_enough(float a, float b)
{
	return fabsf(a - b) < 1e-6f;
}

static void test_axisang_from_quat_z90(void **state)
{
	UNUSED_PARAMETER(state);

	const float half = 0.70710678f;
	struct quat q;
	struct axisang aa;

	quat_set(&q, 0.0f, 0.0f, half, half);
	axisang_from_quat(&aa, &q);
	assert_true(close_enough(aa.x, 0.0f) && close_enough(aa.y, 0.0f) && close_enough(aa.z, 1.0f));
	assert_true(close_enough(aa.w, M_PI / 2.0f));
}

static void test_axisang_from_quat_identity_is_zero(void **state)
{
	UNUSED_PARAMETER(state);

	struct quat q;
	struct axisang aa;

	quat_identity(&q);
	axisang_from_quat(&aa, &q);
	assert_true(aa.x == 0.0f && aa.y == 0.0f && aa.z == 0.0f && aa.w == 0.0f);
}

static void test_axisang_from_quat_x180(void **state)
{
	UNUSED_PARAMETER(state);

	struct quat q;
	struct axisang aa;

	quat_set(&q, 1.0f, 0.0f, 0.0f, 0.0f);
	axisang_from_quat(&aa, &q);
	assert_true(close_enough(aa.x, 1.0f) && close_enough(aa.y, 0.0f) && close_enough(aa.z, 0.0f));
	assert_true(close_enough(aa.w, M_PI));
}

/* The axis is normalized even when the quaternion is not. */
static void test_axisang_from_quat_normalizes_axis(void **state)
{
	UNUSED_PARAMETER(state);

	struct quat q;
	struct axisang aa;

	quat_set(&q, 0.0f, 3.0f, 4.0f, 0.0f);
	axisang_from_quat(&aa, &q);
	assert_true(close_enough(aa.x, 0.0f) && close_enough(aa.y, 0.6f) && close_enough(aa.z, 0.8f));
}

/* A squared vector length up to EPSILON (1e-4) counts as no rotation. */
static void test_axisang_from_quat_epsilon(void **state)
{
	UNUSED_PARAMETER(state);

	struct quat q;
	struct axisang aa;

	quat_set(&q, 0.009f, 0.0f, 0.0f, 1.0f);
	axisang_from_quat(&aa, &q);
	assert_true(aa.x == 0.0f && aa.y == 0.0f && aa.z == 0.0f && aa.w == 0.0f);

	quat_set(&q, 0.011f, 0.0f, 0.0f, 0.5f);
	axisang_from_quat(&aa, &q);
	assert_true(close_enough(aa.x, 1.0f));
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_axisang_from_quat_z90),
		cmocka_unit_test(test_axisang_from_quat_identity_is_zero),
		cmocka_unit_test(test_axisang_from_quat_x180),
		cmocka_unit_test(test_axisang_from_quat_normalizes_axis),
		cmocka_unit_test(test_axisang_from_quat_epsilon),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
