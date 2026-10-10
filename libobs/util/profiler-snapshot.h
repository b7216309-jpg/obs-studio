#pragma once

#include "profiler.h"

/* Snapshot layout, shared by profiler.c, which builds snapshots, and
 * profiler-snapshot.c, which reads and frees them. Private to libobs. */

struct profiler_snapshot {
	DARRAY(profiler_snapshot_entry_t) roots;
};

struct profiler_snapshot_entry {
	const char *name;
	profiler_time_entries_t times;
	uint64_t min_time;
	uint64_t max_time;
	uint64_t overall_count;
	profiler_time_entries_t times_between_calls;
	uint64_t expected_time_between_calls;
	uint64_t min_time_between_calls;
	uint64_t max_time_between_calls;
	uint64_t overall_between_calls_count;
	DARRAY(profiler_snapshot_entry_t) children;
};
