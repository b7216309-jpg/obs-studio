#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <util/c99defs.h>
#include <graphics/quat.h>

/* The header-inline setters in graphics/quat.h. */

static void test_quat_set_stores_xyzw_in_order(void **state)
{
	UNUSED_PARAMETER(state);

	struct quat q;

	quat_set(&q, 1.0f, 2.0f, 3.0f, 4.0f);
	assert_true(q.x == 1.0f);
	assert_true(q.y == 2.0f);
	assert_true(q.z == 3.0f);
	assert_true(q.w == 4.0f);

	/* ptr[] aliases the same lanes */
	assert_true(q.ptr[0] == 1.0f);
	assert_true(q.ptr[3] == 4.0f);
}

static void test_quat_set_matches_quat_identity(void **state)
{
	UNUSED_PARAMETER(state);

	struct quat a, b;

	quat_identity(&a);
	quat_set(&b, 0.0f, 0.0f, 0.0f, 1.0f);
	assert_true(a.x == b.x && a.y == b.y && a.z == b.z && a.w == b.w);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_quat_set_stores_xyzw_in_order),
		cmocka_unit_test(test_quat_set_matches_quat_identity),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
