#include <stdarg.h>
#include <stddef.h>
#include <stdio.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <util/base.h>

struct log_record {
	int calls;
	int level;
	char message[256];
};

static void record_handler(int log_level, const char *format, va_list args, void *param)
{
	struct log_record *rec = param;

	rec->calls++;
	rec->level = log_level;
	vsnprintf(rec->message, sizeof(rec->message), format, args);
}

static void dummy_crash_handler(const char *format, va_list args, void *param)
{
	UNUSED_PARAMETER(format);
	UNUSED_PARAMETER(args);
	UNUSED_PARAMETER(param);
	/* fail_msg() expands to cm_print_error(), which the Windows cmocka
	 * library does not export; report via stdio and use plain fail(). */
	fprintf(stderr, "crash handler must never be called\n");
	fail();
}

static void blog_delivers_level_and_message_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct log_record rec;
	memset(&rec, 0, sizeof(rec));

	log_handler_t prev = NULL;
	void *prev_param = NULL;
	base_get_log_handler(&prev, &prev_param);
	assert_non_null(prev);

	base_set_log_handler(record_handler, &rec);

	blog(LOG_INFO, "x=%d", 5);
	assert_int_equal(rec.calls, 1);
	assert_int_equal(rec.level, LOG_INFO);
	assert_string_equal(rec.message, "x=5");

	blog(LOG_WARNING, "%s-%s", "a", "b");
	assert_int_equal(rec.calls, 2);
	assert_int_equal(rec.level, LOG_WARNING);
	assert_string_equal(rec.message, "a-b");

	blog(LOG_ERROR, "plain");
	assert_int_equal(rec.calls, 3);
	assert_int_equal(rec.level, LOG_ERROR);
	assert_string_equal(rec.message, "plain");

	base_set_log_handler(prev, prev_param);
}

static void get_log_handler_test(void **state)
{
	UNUSED_PARAMETER(state);

	int marker = 0;
	log_handler_t prev = NULL;
	void *prev_param = NULL;
	base_get_log_handler(&prev, &prev_param);

	base_set_log_handler(record_handler, &marker);

	log_handler_t cur = NULL;
	void *cur_param = NULL;
	base_get_log_handler(&cur, &cur_param);
	assert_ptr_equal(cur, record_handler);
	assert_ptr_equal(cur_param, &marker);

	/* NULL out-pointers are accepted */
	base_get_log_handler(NULL, NULL);

	/* a NULL handler falls back to the default one, which is not ours */
	base_set_log_handler(NULL, &marker);
	base_get_log_handler(&cur, &cur_param);
	assert_non_null(cur);
	assert_ptr_not_equal(cur, record_handler);
	assert_ptr_equal(cur_param, &marker);

	base_set_log_handler(prev, prev_param);
	base_get_log_handler(&cur, &cur_param);
	assert_ptr_equal(cur, prev);
	assert_ptr_equal(cur_param, prev_param);
}

static void call_blogva(int log_level, const char *format, ...)
{
	va_list args;

	va_start(args, format);
	blogva(log_level, format, args);
	va_end(args);
}

static void blogva_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct log_record rec;
	memset(&rec, 0, sizeof(rec));

	log_handler_t prev = NULL;
	void *prev_param = NULL;
	base_get_log_handler(&prev, &prev_param);
	base_set_log_handler(record_handler, &rec);

	/* blogva is called directly through a local variadic wrapper */
	call_blogva(LOG_DEBUG, "%s:%03d", "id", 7);
	assert_int_equal(rec.calls, 1);
	assert_int_equal(rec.level, LOG_DEBUG);
	assert_string_equal(rec.message, "id:007");

	base_set_log_handler(prev, prev_param);
}

static void set_crash_handler_once_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct log_record rec;
	memset(&rec, 0, sizeof(rec));

	log_handler_t prev = NULL;
	void *prev_param = NULL;
	base_get_log_handler(&prev, &prev_param);
	base_set_log_handler(record_handler, &rec);

	/* first set succeeds silently */
	base_set_crash_handler(dummy_crash_handler, NULL);
	assert_int_equal(rec.calls, 0);

	/* second set is rejected with a warning */
	base_set_crash_handler(dummy_crash_handler, NULL);
	assert_int_equal(rec.calls, 1);
	assert_int_equal(rec.level, LOG_WARNING);
	assert_string_equal(rec.message, "Tried to set a crash handler when one already exists.");

	base_set_log_handler(prev, prev_param);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(blog_delivers_level_and_message_test),
		cmocka_unit_test(get_log_handler_test),
		cmocka_unit_test(blogva_test),
		cmocka_unit_test(set_crash_handler_once_test),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
