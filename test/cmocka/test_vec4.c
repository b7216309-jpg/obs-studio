#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <math.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <graphics/vec3.h>
#include <graphics/vec4.h>
#include <graphics/matrix4.h>

/* Characterization of the exported functions in graphics/vec4.c. */

static void assert_vec4(const struct vec4 *v, float x, float y, float z, float w)
{
	assert_true(v->x == x);
	assert_true(v->y == y);
	assert_true(v->z == z);
	assert_true(v->w == w);
}

/* Rows 1..16, so every product and sum below is exact. */
static void counting_matrix(struct matrix4 *m)
{
	vec4_set(&m->x, 1.0f, 2.0f, 3.0f, 4.0f);
	vec4_set(&m->y, 5.0f, 6.0f, 7.0f, 8.0f);
	vec4_set(&m->z, 9.0f, 10.0f, 11.0f, 12.0f);
	vec4_set(&m->t, 13.0f, 14.0f, 15.0f, 16.0f);
}

static void test_vec4_from_vec3(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec3 v;
	struct vec4 dst;

	vec3_set(&v, 1.5f, -2.0f, 3.25f);
	vec4_from_vec3(&dst, &v);
	assert_vec4(&dst, 1.5f, -2.0f, 3.25f, 1.0f);
}

static void test_vec4_from_vec3_ignores_w(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec3 v;
	struct vec4 dst;

	vec3_set(&v, 1.0f, 2.0f, 3.0f);
	v.w = 7.0f;
	vec4_from_vec3(&dst, &v);
	assert_vec4(&dst, 1.0f, 2.0f, 3.0f, 1.0f);
}

static void test_vec4_transform_identity(void **state)
{
	UNUSED_PARAMETER(state);

	struct matrix4 m;
	struct vec4 v, dst;

	matrix4_identity(&m);
	vec4_set(&v, 1.5f, -2.0f, 3.25f, 0.5f);
	vec4_transform(&dst, &v, &m);
	assert_vec4(&dst, 1.5f, -2.0f, 3.25f, 0.5f);
}

/* Row vector times matrix: the result is x*m.x + y*m.y + z*m.z + w*m.t. */
static void test_vec4_transform_rows(void **state)
{
	UNUSED_PARAMETER(state);

	struct matrix4 m;
	struct vec4 v, dst;

	counting_matrix(&m);

	vec4_set(&v, 1.0f, 0.0f, 0.0f, 0.0f);
	vec4_transform(&dst, &v, &m);
	assert_vec4(&dst, 1.0f, 2.0f, 3.0f, 4.0f);

	vec4_set(&v, 0.0f, 0.0f, 0.0f, 1.0f);
	vec4_transform(&dst, &v, &m);
	assert_vec4(&dst, 13.0f, 14.0f, 15.0f, 16.0f);

	vec4_set(&v, 1.0f, 1.0f, 1.0f, 1.0f);
	vec4_transform(&dst, &v, &m);
	assert_vec4(&dst, 28.0f, 32.0f, 36.0f, 40.0f);

	vec4_set(&v, 1.0f, -1.0f, 2.0f, 0.5f);
	vec4_transform(&dst, &v, &m);
	assert_vec4(&dst, 20.5f, 23.0f, 25.5f, 28.0f);
}

static void test_vec4_transform_translation(void **state)
{
	UNUSED_PARAMETER(state);

	struct matrix4 m;
	struct vec4 v, dst;

	matrix4_identity(&m);
	vec4_set(&m.t, 10.0f, 20.0f, 30.0f, 1.0f);

	/* A point (w = 1) moves; a direction (w = 0) does not. */
	vec4_set(&v, 1.0f, 2.0f, 3.0f, 1.0f);
	vec4_transform(&dst, &v, &m);
	assert_vec4(&dst, 11.0f, 22.0f, 33.0f, 1.0f);

	vec4_set(&v, 1.0f, 2.0f, 3.0f, 0.0f);
	vec4_transform(&dst, &v, &m);
	assert_vec4(&dst, 1.0f, 2.0f, 3.0f, 0.0f);
}

static void test_vec4_transform_in_place(void **state)
{
	UNUSED_PARAMETER(state);

	struct matrix4 m;
	struct vec4 v;

	counting_matrix(&m);
	vec4_set(&v, 1.0f, 1.0f, 1.0f, 1.0f);
	vec4_transform(&v, &v, &m);
	assert_vec4(&v, 28.0f, 32.0f, 36.0f, 40.0f);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_vec4_from_vec3),
		cmocka_unit_test(test_vec4_from_vec3_ignores_w),
		cmocka_unit_test(test_vec4_transform_identity),
		cmocka_unit_test(test_vec4_transform_rows),
		cmocka_unit_test(test_vec4_transform_translation),
		cmocka_unit_test(test_vec4_transform_in_place),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
