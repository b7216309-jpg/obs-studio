#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <util/task.h>
#include <util/threading.h>

/*
 * Characterization tests for libobs/util/task.c. Deterministic: no sleeps;
 * synchronization only through os_task_queue_wait, os_event and os_sem.
 *
 * Note on os_task_queue_wait's return value: it is true only if a task other
 * than the wait marker ran after the wait call set its "waiting" flag. Whether
 * earlier tasks are still pending at that moment is a race, so the true case
 * is not asserted here (characterized, not endorsed). The false case on an
 * idle queue is deterministic and is pinned.
 */

#define NUM_TASKS 100

static int g_order[NUM_TASKS];
static int g_order_count;

struct idx_param {
	int idx;
};

static void append_task(void *param)
{
	struct idx_param *p = param;
	g_order[g_order_count++] = p->idx;
}

static void test_task_create_destroy_idle(void **state)
{
	UNUSED_PARAMETER(state);

	os_task_queue_t *tq = os_task_queue_create();
	assert_non_null(tq);
	os_task_queue_destroy(tq);
}

static void test_task_fifo_order(void **state)
{
	UNUSED_PARAMETER(state);

	struct idx_param params[NUM_TASKS];

	g_order_count = 0;

	os_task_queue_t *tq = os_task_queue_create();
	assert_non_null(tq);

	for (int i = 0; i < NUM_TASKS; i++) {
		params[i].idx = i;
		assert_true(os_task_queue_queue_task(tq, append_task, &params[i]));
	}

	/* the return value is racy (see header comment); only ordering is pinned */
	(void)os_task_queue_wait(tq);

	assert_int_equal(g_order_count, NUM_TASKS);
	for (int i = 0; i < NUM_TASKS; i++)
		assert_int_equal(g_order[i], i);

	os_task_queue_destroy(tq);
}

static void test_task_wait_idle_returns_false(void **state)
{
	UNUSED_PARAMETER(state);

	os_task_queue_t *tq = os_task_queue_create();
	assert_non_null(tq);

	/* nothing queued: only the wait marker runs, so no task was "processed" */
	assert_false(os_task_queue_wait(tq));
	assert_false(os_task_queue_wait(tq));

	os_task_queue_destroy(tq);
}

static void test_task_wait_runs_earlier_tasks(void **state)
{
	UNUSED_PARAMETER(state);

	struct idx_param a = {1};
	struct idx_param b = {2};

	g_order_count = 0;

	os_task_queue_t *tq = os_task_queue_create();
	assert_non_null(tq);

	assert_true(os_task_queue_queue_task(tq, append_task, &a));
	assert_true(os_task_queue_queue_task(tq, append_task, &b));
	(void)os_task_queue_wait(tq);

	assert_int_equal(g_order_count, 2);
	assert_int_equal(g_order[0], 1);
	assert_int_equal(g_order[1], 2);

	/* a second wait with an idle queue returns false */
	assert_false(os_task_queue_wait(tq));
	assert_int_equal(g_order_count, 2);

	os_task_queue_destroy(tq);
}

struct inside_param {
	os_task_queue_t *tq;
	os_task_queue_t *other;
	bool inside_own;
	bool inside_other;
};

static void inside_task(void *param)
{
	struct inside_param *p = param;
	p->inside_own = os_task_queue_inside(p->tq);
	p->inside_other = os_task_queue_inside(p->other);
}

static void test_task_inside(void **state)
{
	UNUSED_PARAMETER(state);

	os_task_queue_t *tq = os_task_queue_create();
	os_task_queue_t *other = os_task_queue_create();
	assert_non_null(tq);
	assert_non_null(other);

	/* the test thread is not a worker of either queue */
	assert_false(os_task_queue_inside(tq));
	assert_false(os_task_queue_inside(other));

	struct inside_param p = {tq, other, false, true};
	assert_true(os_task_queue_queue_task(tq, inside_task, &p));
	(void)os_task_queue_wait(tq);

	assert_true(p.inside_own);
	/* a worker of one queue is not inside another queue */
	assert_false(p.inside_other);

	os_task_queue_destroy(other);
	os_task_queue_destroy(tq);
}

struct nested_param {
	os_task_queue_t *tq;
	struct idx_param inner;
};

static void nested_outer_task(void *param)
{
	struct nested_param *p = param;

	/* queuing from inside a task on the same queue does not deadlock */
	assert_true(os_task_queue_queue_task(p->tq, append_task, &p->inner));
	g_order[g_order_count++] = 10;
}

static void test_task_queue_from_task(void **state)
{
	UNUSED_PARAMETER(state);

	g_order_count = 0;

	os_task_queue_t *tq = os_task_queue_create();
	assert_non_null(tq);

	struct nested_param p = {tq, {20}};
	assert_true(os_task_queue_queue_task(tq, nested_outer_task, &p));

	/*
	 * The first wait may return before the nested task is queued, so the
	 * nested task is flushed by a second wait (FIFO guarantees it ran).
	 * Both waits' return values are racy and ignored.
	 */
	(void)os_task_queue_wait(tq);
	(void)os_task_queue_wait(tq);

	/* the nested task runs after the task that queued it */
	assert_int_equal(g_order_count, 2);
	assert_int_equal(g_order[0], 10);
	assert_int_equal(g_order[1], 20);

	os_task_queue_destroy(tq);
}

struct block_param {
	os_event_t *started;
	os_event_t *gate;
};

static void block_task(void *param)
{
	struct block_param *p = param;
	os_event_signal(p->started);
	os_event_wait(p->gate);
}

static int g_counter;

static void count_task(void *param)
{
	UNUSED_PARAMETER(param);
	g_counter++;
}

static void test_task_destroy_runs_pending(void **state)
{
	UNUSED_PARAMETER(state);

	struct block_param bp;
	g_counter = 0;

	assert_int_equal(os_event_init(&bp.started, OS_EVENT_TYPE_MANUAL), 0);
	assert_int_equal(os_event_init(&bp.gate, OS_EVENT_TYPE_MANUAL), 0);

	os_task_queue_t *tq = os_task_queue_create();
	assert_non_null(tq);

	assert_true(os_task_queue_queue_task(tq, block_task, &bp));
	/* the worker is now provably inside block_task */
	os_event_wait(bp.started);

	for (int i = 0; i < 5; i++)
		assert_true(os_task_queue_queue_task(tq, count_task, NULL));

	assert_int_equal(g_counter, 0);

	os_event_signal(bp.gate);
	os_task_queue_destroy(tq);

	/*
	 * Characterized, not endorsed: destroy does not discard pending tasks.
	 * The stop marker is queued behind them and the worker is joined, so
	 * all of them run before destroy returns.
	 */
	assert_int_equal(g_counter, 5);

	os_event_destroy(bp.started);
	os_event_destroy(bp.gate);
}

static void test_task_null_args(void **state)
{
	UNUSED_PARAMETER(state);

	/* these entry points check for NULL; os_task_queue_inside does not */
	assert_false(os_task_queue_queue_task(NULL, count_task, NULL));
	assert_false(os_task_queue_wait(NULL));
	os_task_queue_destroy(NULL);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_task_create_destroy_idle),
		cmocka_unit_test(test_task_fifo_order),
		cmocka_unit_test(test_task_wait_idle_returns_false),
		cmocka_unit_test(test_task_wait_runs_earlier_tasks),
		cmocka_unit_test(test_task_inside),
		cmocka_unit_test(test_task_queue_from_task),
		cmocka_unit_test(test_task_destroy_runs_pending),
		cmocka_unit_test(test_task_null_args),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
