#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <stdint.h>
#include <string.h>
#include <wchar.h>
#include <cmocka.h>

#include <util/bmem.h>

static void alignment_constant_test(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(base_get_alignment(), 32);
}

static void bmalloc_alignment_test(void **state)
{
	UNUSED_PARAMETER(state);

	static const size_t sizes[] = {1, 2, 7, 16, 31, 32, 33, 100, 4096, 100000};

	for (size_t i = 0; i < sizeof(sizes) / sizeof(sizes[0]); i++) {
		void *p = bmalloc(sizes[i]);
		assert_non_null(p);
		assert_int_equal((uintptr_t)p % 32, 0);
		/* memory must be writable across the whole block */
		memset(p, 0xAB, sizes[i]);
		bfree(p);
	}
}

static void alloc_count_test(void **state)
{
	UNUSED_PARAMETER(state);

	long base = bnum_allocs();

	void *p = bmalloc(16);
	assert_int_equal(bnum_allocs(), base + 1);

	void *q = brealloc(NULL, 16);
	assert_non_null(q);
	assert_int_equal(bnum_allocs(), base + 2);

	/* growing an existing block does not change the count */
	memcpy(p, "0123456789abcde", 16);
	p = brealloc(p, 4096);
	assert_non_null(p);
	assert_int_equal(bnum_allocs(), base + 2);
	assert_memory_equal(p, "0123456789abcde", 16);

	bfree(p);
	assert_int_equal(bnum_allocs(), base + 1);

	bfree(NULL);
	assert_int_equal(bnum_allocs(), base + 1);

	bfree(q);
	assert_int_equal(bnum_allocs(), base);
}

static void bmemdup_test(void **state)
{
	UNUSED_PARAMETER(state);

	static const unsigned char src[] = {1, 2, 3, 4, 5, 0, 7, 8};
	long base = bnum_allocs();

	void *dup = bmemdup(src, sizeof(src));
	assert_non_null(dup);
	assert_ptr_not_equal(dup, (const void *)src);
	assert_int_equal(bnum_allocs(), base + 1);
	assert_memory_equal(dup, src, sizeof(src));

	bfree(dup);
	assert_int_equal(bnum_allocs(), base);
}

static void bzalloc_test(void **state)
{
	UNUSED_PARAMETER(state);

	long base = bnum_allocs();
	unsigned char *p = bzalloc(257);
	assert_non_null(p);
	assert_int_equal(bnum_allocs(), base + 1);

	for (size_t i = 0; i < 257; i++)
		assert_int_equal(p[i], 0);

	bfree(p);
	assert_int_equal(bnum_allocs(), base);
}

static void bstrdup_test(void **state)
{
	UNUSED_PARAMETER(state);

	long base = bnum_allocs();

	assert_null(bstrdup(NULL));
	assert_null(bstrdup_n(NULL, 5));
	assert_null(bwstrdup(NULL));
	assert_null(bwstrdup_n(NULL, 5));
	assert_int_equal(bnum_allocs(), base);

	char *s = bstrdup("hello");
	assert_non_null(s);
	assert_string_equal(s, "hello");
	assert_int_equal(bnum_allocs(), base + 1);
	bfree(s);

	/* empty string still allocates one byte for the terminator */
	s = bstrdup("");
	assert_non_null(s);
	assert_int_equal(s[0], 0);
	assert_int_equal(bnum_allocs(), base + 1);
	bfree(s);

	/* bstrdup_n copies exactly n bytes and terminates */
	s = bstrdup_n("abcdef", 3);
	assert_non_null(s);
	assert_string_equal(s, "abc");
	bfree(s);

	wchar_t *w = bwstrdup(L"wide");
	assert_non_null(w);
	assert_int_equal(wcscmp(w, L"wide"), 0);
	assert_int_equal(bnum_allocs(), base + 1);
	bfree(w);

	w = bwstrdup_n(L"abcdef", 2);
	assert_non_null(w);
	assert_int_equal(wcscmp(w, L"ab"), 0);
	bfree(w);

	assert_int_equal(bnum_allocs(), base);
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(alignment_constant_test),
		cmocka_unit_test(bmalloc_alignment_test),
		cmocka_unit_test(alloc_count_test),
		cmocka_unit_test(bmemdup_test),
		cmocka_unit_test(bzalloc_test),
		cmocka_unit_test(bstrdup_test),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
