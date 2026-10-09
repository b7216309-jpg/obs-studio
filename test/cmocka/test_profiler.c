#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <util/bmem.h>
#include <util/platform.h>
#include <util/profiler.h>

/*
 * The profiler keys names by pointer, so every name used with profile_start /
 * profile_end / profile_register_root is a single const char * object reused
 * for all calls. Measured times are never asserted on, only structure and
 * counts. The profiler state is global and profiler_free() destroys the global
 * root mutex, so the tests run in a fixed order: the disabled-profiler test
 * first, the full lifecycle last.
 */

#define ITERATIONS 5
#define TMP_DIR "test_profiler.tmp"
#define CSV_FILE TMP_DIR "/out.csv"

static const char root_name[] = "test_root";
static const char child_name[] = "test_child";
static const char other_name[] = "test_other";

struct collected {
	profiler_snapshot_entry_t *entries[8];
	const char *names[8];
	size_t num;
};

static bool collect(void *context, profiler_snapshot_entry_t *entry)
{
	struct collected *c = context;
	c->entries[c->num] = entry;
	c->names[c->num] = profiler_snapshot_entry_name(entry);
	c->num++;
	return true;
}

static bool collect_first_only(void *context, profiler_snapshot_entry_t *entry)
{
	collect(context, entry);
	return false;
}

static bool remove_other(void *data, const char *name, bool *remove)
{
	(*(size_t *)data)++;
	*remove = strcmp(name, other_name) == 0;
	return true;
}

static void test_profiler_name_store(void **state)
{
	UNUSED_PARAMETER(state);

	profiler_name_store_t *store = profiler_name_store_create();
	assert_non_null(store);

	const char *a = profile_store_name(store, "%s-%d", "alpha", 1);
	const char *b = profile_store_name(store, "%s-%d", "beta", 22);
	const char *c = profile_store_name(store, "plain");

	assert_non_null(a);
	assert_non_null(b);
	assert_non_null(c);

	/* every call yields a distinct pointer, even for equal strings */
	const char *a2 = profile_store_name(store, "%s-%d", "alpha", 1);
	assert_ptr_not_equal(a, a2);
	assert_ptr_not_equal(a, b);

	/* earlier pointers stay valid after later stores */
	assert_string_equal(a, "alpha-1");
	assert_string_equal(b, "beta-22");
	assert_string_equal(c, "plain");
	assert_string_equal(a2, "alpha-1");

	profiler_name_store_free(store);

	/* freeing NULL is a no-op */
	profiler_name_store_free(NULL);
}

static void test_profiler_disabled(void **state)
{
	UNUSED_PARAMETER(state);

	/* profiling is disabled until profiler_start(): nothing is recorded */
	profile_register_root(root_name, 1000000);
	profile_start(root_name);
	profile_start(child_name);
	profile_end(child_name);
	profile_end(root_name);

	profiler_snapshot_t *snap = profile_snapshot_create();
	assert_non_null(snap);
	assert_int_equal(profiler_snapshot_num_roots(snap), 0);

	struct collected c = {0};
	profiler_snapshot_enumerate_roots(snap, collect, &c);
	assert_int_equal(c.num, 0);

	profile_snapshot_free(snap);

	/* stopping a profiler that is not running, then reenabling, is harmless */
	profiler_stop();
	profile_reenable_thread();

	/* NULL-safe accessors */
	assert_int_equal(profiler_snapshot_num_roots(NULL), 0);
	assert_int_equal(profiler_snapshot_num_children(NULL), 0);
	assert_null(profiler_snapshot_entry_name(NULL));
	assert_null(profiler_snapshot_entry_times(NULL));
	assert_null(profiler_snapshot_entry_times_between_calls(NULL));
	assert_int_equal(profiler_snapshot_entry_overall_count(NULL), 0);
	assert_int_equal(profiler_snapshot_entry_min_time(NULL), 0);
	assert_int_equal(profiler_snapshot_entry_max_time(NULL), 0);
	assert_int_equal(profiler_snapshot_entry_expected_time_between_calls(NULL), 0);
	assert_int_equal(profiler_snapshot_entry_min_time_between_calls(NULL), 0);
	assert_int_equal(profiler_snapshot_entry_max_time_between_calls(NULL), 0);
	assert_int_equal(profiler_snapshot_entry_overall_between_calls_count(NULL), 0);
	profiler_snapshot_enumerate_roots(NULL, collect, &c);
	profiler_snapshot_enumerate_children(NULL, collect, &c);
	assert_int_equal(c.num, 0);
	profile_snapshot_free(NULL);
}

static void test_profiler_lifecycle(void **state)
{
	UNUSED_PARAMETER(state);

	profiler_start();
	profile_reenable_thread();

	/* 16000000 ns is stored in microseconds, rounded: (ns + 500) / 1000 */
	profile_register_root(root_name, 16000000);
	profile_register_root(other_name, 0);

	for (int i = 0; i < ITERATIONS; i++) {
		profile_start(root_name);
		profile_start(child_name);
		profile_end(child_name);
		profile_end(root_name);
	}

	profile_start(other_name);
	profile_end(other_name);

	profiler_snapshot_t *snap = profile_snapshot_create();
	assert_non_null(snap);
	assert_int_equal(profiler_snapshot_num_roots(snap), 2);

	/* roots are enumerated in registration order */
	struct collected roots = {0};
	profiler_snapshot_enumerate_roots(snap, collect, &roots);
	assert_int_equal(roots.num, 2);
	assert_ptr_equal(roots.names[0], root_name);
	assert_ptr_equal(roots.names[1], other_name);

	/* a callback returning false stops the enumeration */
	struct collected first = {0};
	profiler_snapshot_enumerate_roots(snap, collect_first_only, &first);
	assert_int_equal(first.num, 1);
	assert_ptr_equal(first.names[0], root_name);

	profiler_snapshot_entry_t *root = roots.entries[0];
	profiler_snapshot_entry_t *other = roots.entries[1];

	/* root: ITERATIONS calls, one child, registered expected time */
	assert_int_equal(profiler_snapshot_entry_overall_count(root), ITERATIONS);
	assert_true(profiler_snapshot_entry_min_time(root) <= profiler_snapshot_entry_max_time(root));
	assert_int_equal(profiler_snapshot_entry_expected_time_between_calls(root), 16000);
	assert_int_equal(profiler_snapshot_entry_overall_between_calls_count(root), ITERATIONS - 1);
	assert_true(profiler_snapshot_entry_min_time_between_calls(root) <=
		    profiler_snapshot_entry_max_time_between_calls(root));
	assert_non_null(profiler_snapshot_entry_times(root));
	assert_non_null(profiler_snapshot_entry_times_between_calls(root));
	assert_int_equal(profiler_snapshot_num_children(root), 1);

	struct collected children = {0};
	profiler_snapshot_enumerate_children(root, collect, &children);
	assert_int_equal(children.num, 1);
	assert_ptr_equal(children.names[0], child_name);

	profiler_snapshot_entry_t *child = children.entries[0];
	assert_int_equal(profiler_snapshot_entry_overall_count(child), ITERATIONS);
	assert_true(profiler_snapshot_entry_min_time(child) <= profiler_snapshot_entry_max_time(child));
	assert_int_equal(profiler_snapshot_num_children(child), 0);
	assert_int_equal(profiler_snapshot_entry_expected_time_between_calls(child), 0);
	assert_int_equal(profiler_snapshot_entry_overall_between_calls_count(child), 0);

	/* the unregistered-expected root: one call, no time-between-calls data */
	assert_int_equal(profiler_snapshot_entry_overall_count(other), 1);
	assert_int_equal(profiler_snapshot_entry_expected_time_between_calls(other), 0);
	assert_int_equal(profiler_snapshot_entry_overall_between_calls_count(other), 0);
	assert_int_equal(profiler_snapshot_num_children(other), 0);

	/* the recorded time entries add up to the overall count */
	profiler_time_entries_t *times = profiler_snapshot_entry_times(root);
	uint64_t total = 0;
	for (size_t i = 0; i < times->num; i++)
		total += times->array[i].count;
	assert_int_equal(total, ITERATIONS);

	/* csv dump: header line is fixed */
	os_unlink(CSV_FILE);
	assert_true(os_mkdirs(TMP_DIR) != MKDIR_ERROR);
	assert_true(profiler_snapshot_dump_csv(snap, CSV_FILE));

	char *csv = os_quick_read_utf8_file(CSV_FILE);
	assert_non_null(csv);
	const char *header = "id,parent_id,name_id,parent_name_id,name,time_between_calls,time_delta_\xc2\xb5s,count\n";
	assert_int_equal(strncmp(csv, header, strlen(header)), 0);
	bfree(csv);
	os_unlink(CSV_FILE);

	/* an unwritable path fails */
	assert_false(profiler_snapshot_dump_csv(snap, TMP_DIR "/no_such_dir/out.csv"));
	os_rmdir(TMP_DIR);

	/* filtering removes roots by name; the callback sees every root */
	size_t visited = 0;
	profiler_snapshot_filter_roots(snap, remove_other, &visited);
	assert_int_equal(visited, 2);
	assert_int_equal(profiler_snapshot_num_roots(snap), 1);

	struct collected left = {0};
	profiler_snapshot_enumerate_roots(snap, collect, &left);
	assert_int_equal(left.num, 1);
	assert_ptr_equal(left.names[0], root_name);

	profile_snapshot_free(snap);

	/* after profiler_stop, new calls are dropped and the counts do not move */
	profiler_stop();

	profile_start(root_name);
	profile_start(child_name);
	profile_end(child_name);
	profile_end(root_name);

	snap = profile_snapshot_create();
	assert_non_null(snap);
	assert_int_equal(profiler_snapshot_num_roots(snap), 2);

	struct collected after = {0};
	profiler_snapshot_enumerate_roots(snap, collect, &after);
	assert_int_equal(after.num, 2);
	assert_int_equal(profiler_snapshot_entry_overall_count(after.entries[0]), ITERATIONS);
	assert_int_equal(profiler_snapshot_entry_overall_count(after.entries[1]), 1);

	profile_snapshot_free(snap);

	profiler_free();
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_profiler_name_store),
		cmocka_unit_test(test_profiler_disabled),
		cmocka_unit_test(test_profiler_lifecycle),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
