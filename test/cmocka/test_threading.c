#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <errno.h>

#include <util/threading.h>

/* The return codes asserted here are identical in threading-posix.c and
 * threading-windows.c (0 / EAGAIN / ETIMEDOUT), and the atomics have the same
 * contract in threading-posix.h and threading-windows.h. */

#define PING_PONG_COUNT 1000
#define SEM_POST_COUNT 500
#define ATOMIC_THREADS 4
#define ATOMIC_ITERATIONS 10000

static void test_event_init_auto_manual(void **state)
{
	UNUSED_PARAMETER(state);

	os_event_t *ev = NULL;

	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_AUTO), 0);
	assert_non_null(ev);
	os_event_destroy(ev);

	ev = NULL;
	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_MANUAL), 0);
	assert_non_null(ev);
	os_event_destroy(ev);

	/* destroying NULL is a no-op */
	os_event_destroy(NULL);
}

static void test_event_try_unsignaled(void **state)
{
	UNUSED_PARAMETER(state);

	os_event_t *ev = NULL;

	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_AUTO), 0);
	assert_int_equal(os_event_try(ev), EAGAIN);
	os_event_destroy(ev);

	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_MANUAL), 0);
	assert_int_equal(os_event_try(ev), EAGAIN);
	os_event_destroy(ev);
}

static void test_event_auto_semantics(void **state)
{
	UNUSED_PARAMETER(state);

	os_event_t *ev = NULL;

	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_AUTO), 0);
	assert_int_equal(os_event_signal(ev), 0);
	assert_int_equal(os_event_try(ev), 0);
	/* an auto-reset event is consumed by the first successful try */
	assert_int_equal(os_event_try(ev), EAGAIN);

	/* wait returns immediately on a signaled event and consumes it */
	assert_int_equal(os_event_signal(ev), 0);
	assert_int_equal(os_event_wait(ev), 0);
	assert_int_equal(os_event_try(ev), EAGAIN);

	/* signaling twice still only releases one waiter */
	assert_int_equal(os_event_signal(ev), 0);
	assert_int_equal(os_event_signal(ev), 0);
	assert_int_equal(os_event_try(ev), 0);
	assert_int_equal(os_event_try(ev), EAGAIN);

	/* reset clears a pending signal */
	assert_int_equal(os_event_signal(ev), 0);
	os_event_reset(ev);
	assert_int_equal(os_event_try(ev), EAGAIN);

	os_event_destroy(ev);
}

static void test_event_manual_semantics(void **state)
{
	UNUSED_PARAMETER(state);

	os_event_t *ev = NULL;

	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_MANUAL), 0);
	assert_int_equal(os_event_signal(ev), 0);
	/* a manual-reset event stays signaled until reset */
	assert_int_equal(os_event_try(ev), 0);
	assert_int_equal(os_event_try(ev), 0);
	assert_int_equal(os_event_wait(ev), 0);
	assert_int_equal(os_event_timedwait(ev, 1), 0);
	assert_int_equal(os_event_try(ev), 0);

	os_event_reset(ev);
	assert_int_equal(os_event_try(ev), EAGAIN);

	/* reset on an unsignaled event is harmless */
	os_event_reset(ev);
	assert_int_equal(os_event_try(ev), EAGAIN);

	os_event_destroy(ev);
}

static void test_event_timedwait(void **state)
{
	UNUSED_PARAMETER(state);

	os_event_t *ev = NULL;

	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_AUTO), 0);
	assert_int_equal(os_event_timedwait(ev, 10), ETIMEDOUT);

	/* signaled: returns 0 immediately and consumes the auto event */
	assert_int_equal(os_event_signal(ev), 0);
	assert_int_equal(os_event_timedwait(ev, 10), 0);
	assert_int_equal(os_event_timedwait(ev, 10), ETIMEDOUT);
	os_event_destroy(ev);

	assert_int_equal(os_event_init(&ev, OS_EVENT_TYPE_MANUAL), 0);
	assert_int_equal(os_event_timedwait(ev, 10), ETIMEDOUT);
	os_event_destroy(ev);
}

struct ping_pong_ctx {
	os_event_t *ping;
	os_event_t *pong;
	int rounds;
	int failures;
};

static void *ping_pong_worker(void *param)
{
	struct ping_pong_ctx *ctx = param;

	for (int i = 0; i < ctx->rounds; i++) {
		if (os_event_wait(ctx->ping) != 0)
			ctx->failures++;
		if (os_event_signal(ctx->pong) != 0)
			ctx->failures++;
	}

	return NULL;
}

static void test_event_ping_pong(void **state)
{
	UNUSED_PARAMETER(state);

	struct ping_pong_ctx ctx = {0};
	pthread_t thread;

	ctx.rounds = PING_PONG_COUNT;
	assert_int_equal(os_event_init(&ctx.ping, OS_EVENT_TYPE_AUTO), 0);
	assert_int_equal(os_event_init(&ctx.pong, OS_EVENT_TYPE_AUTO), 0);

	assert_int_equal(pthread_create(&thread, NULL, ping_pong_worker, &ctx), 0);

	for (int i = 0; i < PING_PONG_COUNT; i++) {
		assert_int_equal(os_event_signal(ctx.ping), 0);
		assert_int_equal(os_event_wait(ctx.pong), 0);
	}

	assert_int_equal(pthread_join(thread, NULL), 0);
	assert_int_equal(ctx.failures, 0);
	assert_int_equal(os_event_try(ctx.ping), EAGAIN);
	assert_int_equal(os_event_try(ctx.pong), EAGAIN);

	os_event_destroy(ctx.ping);
	os_event_destroy(ctx.pong);
}

static void test_sem_initial_value(void **state)
{
	UNUSED_PARAMETER(state);

	os_sem_t *sem = NULL;

	assert_int_equal(os_sem_init(&sem, 2), 0);
	assert_non_null(sem);

	/* two waits do not block with an initial value of 2 */
	assert_int_equal(os_sem_wait(sem), 0);
	assert_int_equal(os_sem_wait(sem), 0);

	/* post then wait round-trips */
	assert_int_equal(os_sem_post(sem), 0);
	assert_int_equal(os_sem_wait(sem), 0);

	os_sem_destroy(sem);
	os_sem_destroy(NULL);
}

static void test_sem_null(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(os_sem_post(NULL), -1);
	assert_int_equal(os_sem_wait(NULL), -1);
}

struct sem_ctx {
	os_sem_t *sem;
	int count;
	int failures;
};

static void *sem_producer(void *param)
{
	struct sem_ctx *ctx = param;

	for (int i = 0; i < ctx->count; i++) {
		if (os_sem_post(ctx->sem) != 0)
			ctx->failures++;
	}

	return NULL;
}

static void test_sem_producer_consumer(void **state)
{
	UNUSED_PARAMETER(state);

	struct sem_ctx ctx = {0};
	pthread_t thread;

	ctx.count = SEM_POST_COUNT;
	assert_int_equal(os_sem_init(&ctx.sem, 0), 0);
	assert_int_equal(pthread_create(&thread, NULL, sem_producer, &ctx), 0);

	for (int i = 0; i < SEM_POST_COUNT; i++)
		assert_int_equal(os_sem_wait(ctx.sem), 0);

	assert_int_equal(pthread_join(thread, NULL), 0);
	assert_int_equal(ctx.failures, 0);

	os_sem_destroy(ctx.sem);
}

static void test_atomic_long(void **state)
{
	UNUSED_PARAMETER(state);

	volatile long v = 0;

	/* inc and dec return the new value */
	assert_int_equal(os_atomic_inc_long(&v), 1);
	assert_int_equal(os_atomic_inc_long(&v), 2);
	assert_int_equal(os_atomic_dec_long(&v), 1);
	assert_int_equal(os_atomic_dec_long(&v), 0);
	assert_int_equal(os_atomic_dec_long(&v), -1);
	assert_int_equal(os_atomic_load_long(&v), -1);

	os_atomic_store_long(&v, 5);
	assert_int_equal(os_atomic_load_long(&v), 5);

	/* set and exchange return the previous value */
	assert_int_equal(os_atomic_set_long(&v, 7), 5);
	assert_int_equal(os_atomic_load_long(&v), 7);
	assert_int_equal(os_atomic_exchange_long(&v, 9), 7);
	assert_int_equal(os_atomic_load_long(&v), 9);

	/* compare_swap only stores when the current value matches */
	assert_true(os_atomic_compare_swap_long(&v, 9, 11));
	assert_int_equal(os_atomic_load_long(&v), 11);
	assert_false(os_atomic_compare_swap_long(&v, 9, 13));
	assert_int_equal(os_atomic_load_long(&v), 11);

	/* compare_exchange updates the expected value on failure */
	long expected = 11;
	assert_true(os_atomic_compare_exchange_long(&v, &expected, 20));
	assert_int_equal(expected, 11);
	assert_int_equal(os_atomic_load_long(&v), 20);
	expected = 1;
	assert_false(os_atomic_compare_exchange_long(&v, &expected, 30));
	assert_int_equal(expected, 20);
	assert_int_equal(os_atomic_load_long(&v), 20);
}

static void test_atomic_bool(void **state)
{
	UNUSED_PARAMETER(state);

	volatile bool b = false;

	assert_false(os_atomic_load_bool(&b));

	os_atomic_store_bool(&b, true);
	assert_true(os_atomic_load_bool(&b));

	/* set and exchange return the previous value */
	assert_true(os_atomic_set_bool(&b, false));
	assert_false(os_atomic_load_bool(&b));
	assert_false(os_atomic_set_bool(&b, true));
	assert_true(os_atomic_exchange_bool(&b, false));
	assert_false(os_atomic_exchange_bool(&b, true));
	assert_true(os_atomic_load_bool(&b));
}

static volatile long concurrent_counter;

static void *atomic_incrementer(void *param)
{
	UNUSED_PARAMETER(param);

	for (int i = 0; i < ATOMIC_ITERATIONS; i++)
		os_atomic_inc_long(&concurrent_counter);

	return NULL;
}

static void test_atomic_concurrent_inc(void **state)
{
	UNUSED_PARAMETER(state);

	pthread_t threads[ATOMIC_THREADS];

	concurrent_counter = 0;

	for (int i = 0; i < ATOMIC_THREADS; i++)
		assert_int_equal(pthread_create(&threads[i], NULL, atomic_incrementer, NULL), 0);

	for (int i = 0; i < ATOMIC_THREADS; i++)
		assert_int_equal(pthread_join(threads[i], NULL), 0);

	assert_int_equal(os_atomic_load_long(&concurrent_counter), (long)ATOMIC_THREADS * ATOMIC_ITERATIONS);
}

static void *thread_name_worker(void *param)
{
	UNUSED_PARAMETER(param);

	/* short name, then one longer than the 15 character Linux limit */
	os_set_thread_name("test-thread");
	os_set_thread_name("a-very-long-thread-name-over-limit");

	return NULL;
}

static void test_set_thread_name(void **state)
{
	UNUSED_PARAMETER(state);

	pthread_t thread;

	assert_int_equal(pthread_create(&thread, NULL, thread_name_worker, NULL), 0);
	assert_int_equal(pthread_join(thread, NULL), 0);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_event_init_auto_manual),
		cmocka_unit_test(test_event_try_unsignaled),
		cmocka_unit_test(test_event_auto_semantics),
		cmocka_unit_test(test_event_manual_semantics),
		cmocka_unit_test(test_event_timedwait),
		cmocka_unit_test(test_event_ping_pong),
		cmocka_unit_test(test_sem_initial_value),
		cmocka_unit_test(test_sem_null),
		cmocka_unit_test(test_sem_producer_consumer),
		cmocka_unit_test(test_atomic_long),
		cmocka_unit_test(test_atomic_bool),
		cmocka_unit_test(test_atomic_concurrent_inc),
		cmocka_unit_test(test_set_thread_name),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
