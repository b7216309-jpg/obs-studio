#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include "product-name.h"

static void upstream_build_is_obs_version(void **state)
{
	(void)state;
	char buf[64];

	assert_int_equal(obs_product_string(buf, sizeof(buf), "32.1.2", NULL), 10);
	assert_string_equal(buf, "OBS 32.1.2");
}

static void rust_build_names_itself_and_its_base(void **state)
{
	(void)state;
	char buf[96];
	const char *want = "OBS-Studio-Rust 0.3.0 (based on OBS 32.1.2)";

	assert_int_equal(obs_product_string(buf, sizeof(buf), "32.1.2", "0.3.0"), (int)strlen(want));
	assert_string_equal(buf, want);
}

static void truncates_and_terminates(void **state)
{
	(void)state;
	char buf[8];

	assert_int_equal(obs_product_string(buf, sizeof(buf), "32.1.2", "0.3.0"),
			 (int)strlen("OBS-Studio-Rust 0.3.0 (based on OBS 32.1.2)"));
	assert_string_equal(buf, "OBS-Stu");
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(upstream_build_is_obs_version),
		cmocka_unit_test(rust_build_names_itself_and_its_base),
		cmocka_unit_test(truncates_and_terminates),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
