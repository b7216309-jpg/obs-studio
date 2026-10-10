#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <stdlib.h>
#include <math.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <graphics/vec3.h>
#include <graphics/vec4.h>
#include <graphics/plane.h>
#include <graphics/matrix3.h>
#include <graphics/matrix4.h>

/* Characterization of the exported functions in graphics/vec3.c. */

static void assert_vec3(const struct vec3 *v, float x, float y, float z)
{
	assert_true(v->x == x);
	assert_true(v->y == y);
	assert_true(v->z == z);
	assert_true(v->w == 0.0f);
}

/* Axes that cycle the components, and a translation of (10, 20, 30). */
static void cycling_axes(struct matrix3 *m)
{
	vec3_set(&m->x, 0.0f, 1.0f, 0.0f);
	vec3_set(&m->y, 0.0f, 0.0f, 1.0f);
	vec3_set(&m->z, 1.0f, 0.0f, 0.0f);
	vec3_set(&m->t, 10.0f, 20.0f, 30.0f);
}

static void floor_plane(struct plane *p)
{
	vec3_set(&p->dir, 0.0f, 1.0f, 0.0f);
	p->dist = 2.0f;
}

static void test_vec3_from_vec4(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec4 v;
	struct vec3 dst;

	vec4_set(&v, 1.0f, 2.0f, 3.0f, 4.0f);
	vec3_from_vec4(&dst, &v);
	assert_vec3(&dst, 1.0f, 2.0f, 3.0f);
}

static void test_vec3_plane_dist(void **state)
{
	UNUSED_PARAMETER(state);

	struct plane p;
	struct vec3 v;

	floor_plane(&p);
	vec3_set(&v, 4.0f, 5.0f, 6.0f);
	assert_true(vec3_plane_dist(&v, &p) == 3.0f);
	vec3_set(&v, 4.0f, 2.0f, 6.0f);
	assert_true(vec3_plane_dist(&v, &p) == 0.0f);
}

static void test_vec3_rotate(void **state)
{
	UNUSED_PARAMETER(state);

	struct matrix3 m;
	struct vec3 v, dst;

	cycling_axes(&m);
	vec3_set(&v, 1.0f, 2.0f, 3.0f);
	vec3_rotate(&dst, &v, &m);
	assert_vec3(&dst, 2.0f, 3.0f, 1.0f);

	/* in place */
	vec3_rotate(&v, &v, &m);
	assert_vec3(&v, 2.0f, 3.0f, 1.0f);
}

static void test_vec3_transform3x4(void **state)
{
	UNUSED_PARAMETER(state);

	struct matrix3 m;
	struct vec3 v, dst;

	cycling_axes(&m);
	vec3_set(&v, 1.0f, 2.0f, 3.0f);
	vec3_transform3x4(&dst, &v, &m);
	assert_vec3(&dst, -18.0f, -27.0f, -9.0f);
}

static void test_vec3_transform(void **state)
{
	UNUSED_PARAMETER(state);

	struct matrix4 m;
	struct vec3 v, dst;

	matrix4_identity(&m);
	vec4_set(&m.t, 10.0f, 20.0f, 30.0f, 1.0f);
	vec3_set(&v, 1.0f, 2.0f, 3.0f);
	vec3_transform(&dst, &v, &m);
	assert_vec3(&dst, 11.0f, 22.0f, 33.0f);

	/* in place */
	vec3_transform(&v, &v, &m);
	assert_vec3(&v, 11.0f, 22.0f, 33.0f);
}

static void test_vec3_mirror(void **state)
{
	UNUSED_PARAMETER(state);

	struct plane p;
	struct vec3 v, dst;

	floor_plane(&p);
	vec3_set(&v, 4.0f, 5.0f, 6.0f);
	vec3_mirror(&dst, &v, &p);
	assert_vec3(&dst, 4.0f, -1.0f, 6.0f);
}

static void test_vec3_mirrorv(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec3 v, dir, dst;

	vec3_set(&v, 1.0f, 2.0f, 3.0f);
	vec3_set(&dir, 1.0f, 0.0f, 0.0f);
	vec3_mirrorv(&dst, &v, &dir);
	assert_vec3(&dst, -1.0f, 2.0f, 3.0f);
}

static void test_vec3_rand(void **state)
{
	UNUSED_PARAMETER(state);

	struct vec3 a, b;

	srand(42);
	vec3_rand(&a, 1);
	srand(42);
	vec3_rand(&b, 1);
	assert_true(a.x == b.x && a.y == b.y && a.z == b.z);
	assert_true(a.w == 0.0f);
	assert_true(a.x >= 0.0f && a.x <= 1.0f);
	assert_true(a.y >= 0.0f && a.y <= 1.0f);
	assert_true(a.z >= 0.0f && a.z <= 1.0f);

	vec3_rand(&a, 0);
	assert_true(a.x >= -1.0f && a.x <= 1.0f);
	assert_true(a.y >= -1.0f && a.y <= 1.0f);
	assert_true(a.z >= -1.0f && a.z <= 1.0f);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_vec3_from_vec4), cmocka_unit_test(test_vec3_plane_dist),
		cmocka_unit_test(test_vec3_rotate),    cmocka_unit_test(test_vec3_transform3x4),
		cmocka_unit_test(test_vec3_transform), cmocka_unit_test(test_vec3_mirror),
		cmocka_unit_test(test_vec3_mirrorv),   cmocka_unit_test(test_vec3_rand),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
