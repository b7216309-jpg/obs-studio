#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <util/text-lookup.h>

#define INI_PATH "test_text_lookup.ini"
#define INI2_PATH "test_text_lookup2.ini"
#define MISSING_PATH "test_text_lookup_does_not_exist.ini"

static void write_file(const char *path, const char *data)
{
	FILE *f = fopen(path, "wb");
	assert_non_null(f);
	assert_int_equal(fwrite(data, 1, strlen(data), f), strlen(data));
	fclose(f);
}

static void assert_lookup(lookup_t *lookup, const char *key, const char *expected)
{
	const char *out = NULL;
	assert_true(text_lookup_getstr(lookup, key, &out));
	assert_non_null(out);
	assert_string_equal(out, expected);
}

static void assert_missing(lookup_t *lookup, const char *key)
{
	const char *out = NULL;
	assert_false(text_lookup_getstr(lookup, key, &out));
	assert_null(out);
}

static const char ini1[] = "# a comment line\n"
			   "Key=\"Value\"\n"
			   "Plain=hello\n"
			   "Esc=\"a\\nb\\tc\"\n"
			   "Quote=\"say \\\"hi\\\"\"\n"
			   "Dup=\"first\"\n"
			   "Dup=\"second\"\n"
			   "Last=\"end\"";

static const char ini2[] = "Key=\"Override\"\n"
			   "Extra=\"more\"";

static void create_and_getstr_test(void **state)
{
	UNUSED_PARAMETER(state);

	write_file(INI_PATH, ini1);
	lookup_t *lookup = text_lookup_create(INI_PATH);
	remove(INI_PATH);
	assert_non_null(lookup);

	assert_lookup(lookup, "Key", "Value");
	assert_lookup(lookup, "Plain", "hello");
	assert_lookup(lookup, "Esc", "a\nb\tc");
	assert_lookup(lookup, "Quote", "say \"hi\"");
	/* later duplicate replaces the earlier one */
	assert_lookup(lookup, "Dup", "second");
	/* final line without trailing newline is still read */
	assert_lookup(lookup, "Last", "end");

	assert_missing(lookup, "Missing");
	assert_missing(lookup, "key"); /* lookup is case sensitive */

	text_lookup_destroy(lookup);
}

static void add_overrides_test(void **state)
{
	UNUSED_PARAMETER(state);

	write_file(INI_PATH, ini1);
	write_file(INI2_PATH, ini2);

	lookup_t *lookup = text_lookup_create(INI_PATH);
	assert_non_null(lookup);
	assert_lookup(lookup, "Key", "Value");
	assert_missing(lookup, "Extra");

	assert_true(text_lookup_add(lookup, INI2_PATH));
	remove(INI_PATH);
	remove(INI2_PATH);

	assert_lookup(lookup, "Key", "Override");
	assert_lookup(lookup, "Extra", "more");
	/* keys absent from the second file survive */
	assert_lookup(lookup, "Plain", "hello");

	/* adding a missing file fails and leaves existing entries intact */
	assert_false(text_lookup_add(lookup, MISSING_PATH));
	assert_lookup(lookup, "Key", "Override");

	text_lookup_destroy(lookup);
}

static void missing_file_test(void **state)
{
	UNUSED_PARAMETER(state);

	assert_null(text_lookup_create(MISSING_PATH));
}

static void null_lookup_test(void **state)
{
	UNUSED_PARAMETER(state);

	const char *out = NULL;
	assert_false(text_lookup_getstr(NULL, "Key", &out));
	assert_null(out);

	text_lookup_destroy(NULL);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(create_and_getstr_test),
		cmocka_unit_test(add_overrides_test),
		cmocka_unit_test(missing_file_test),
		cmocka_unit_test(null_lookup_test),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
