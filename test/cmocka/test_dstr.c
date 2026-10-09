#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <util/bmem.h>
#include <util/dstr.h>
#include <util/lexer.h>

/* Characterization tests: every assertion pins what libobs/util/dstr.c does
 * today. Each test is bracketed by an allocation-count check to catch leaks. */

static long allocs_at_start;

static int leak_setup(void **state)
{
	UNUSED_PARAMETER(state);
	allocs_at_start = bnum_allocs();
	return 0;
}

static int leak_teardown(void **state)
{
	UNUSED_PARAMETER(state);
	assert_int_equal(bnum_allocs(), allocs_at_start);
	return 0;
}

static void test_astrcmpi(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(astrcmpi("abc", "abc"), 0);
	assert_int_equal(astrcmpi("abc", "ABC"), 0);
	assert_int_equal(astrcmpi("aBc", "AbC"), 0);
	assert_int_equal(astrcmpi("a", "b"), -1);
	assert_int_equal(astrcmpi("B", "a"), 1);
	assert_int_equal(astrcmpi("ab", "abc"), -1);
	assert_int_equal(astrcmpi("abc", "ab"), 1);
	assert_int_equal(astrcmpi("", ""), 0);

	/* NULL is treated as the empty string */
	assert_int_equal(astrcmpi(NULL, NULL), 0);
	assert_int_equal(astrcmpi(NULL, ""), 0);
	assert_int_equal(astrcmpi("", NULL), 0);
	assert_int_equal(astrcmpi(NULL, "a"), -1);
	assert_int_equal(astrcmpi("a", NULL), 1);
}

static void test_astrcmp_n(void **state)
{
	UNUSED_PARAMETER(state);

	/* n == 0 always equal */
	assert_int_equal(astrcmp_n("a", "b", 0), 0);
	assert_int_equal(astrcmp_n(NULL, "b", 0), 0);

	assert_int_equal(astrcmp_n("abcd", "abce", 3), 0);
	assert_int_equal(astrcmp_n("abcd", "abce", 4), -1);
	assert_int_equal(astrcmp_n("abce", "abcd", 4), 1);
	assert_int_equal(astrcmp_n("abc", "abc", 100), 0);
	assert_int_equal(astrcmp_n("ab", "abc", 100), -1);

	/* case sensitive */
	assert_int_equal(astrcmp_n("A", "a", 1), -1);
	assert_int_equal(astrcmp_n("a", "A", 1), 1);

	/* NULL is treated as the empty string */
	assert_int_equal(astrcmp_n(NULL, NULL, 5), 0);
	assert_int_equal(astrcmp_n(NULL, "a", 1), -1);
	assert_int_equal(astrcmp_n("a", NULL, 1), 1);
}

static void test_astrcmpi_n(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(astrcmpi_n("a", "b", 0), 0);
	assert_int_equal(astrcmpi_n(NULL, "b", 0), 0);

	assert_int_equal(astrcmpi_n("ABCD", "abce", 3), 0);
	assert_int_equal(astrcmpi_n("ABCD", "abce", 4), -1);
	assert_int_equal(astrcmpi_n("abce", "ABCD", 4), 1);
	assert_int_equal(astrcmpi_n("ABC", "abc", 100), 0);
	assert_int_equal(astrcmpi_n("AB", "abc", 100), -1);

	assert_int_equal(astrcmpi_n(NULL, NULL, 5), 0);
	assert_int_equal(astrcmpi_n(NULL, "a", 1), -1);
	assert_int_equal(astrcmpi_n("a", NULL, 1), 1);
}

static void test_astrstri(void **state)
{
	UNUSED_PARAMETER(state);

	const char *str = "Hello World";

	assert_ptr_equal(astrstri(str, "WORLD"), str + 6);
	assert_ptr_equal(astrstri(str, "llo"), str + 2);
	assert_ptr_equal(astrstri(str, "hello world"), str);
	assert_ptr_equal(astrstri(str, "d"), str + 10);
	assert_null(astrstri(str, "xyz"));
	assert_null(astrstri(str, "Hello World!"));
	assert_null(astrstri("", "a"));

	/* empty needle matches at the start */
	assert_ptr_equal(astrstri(str, ""), str);

	assert_null(astrstri(NULL, "a"));
	assert_null(astrstri(str, NULL));
	assert_null(astrstri(NULL, NULL));
}

static void test_strdepad(void **state)
{
	UNUSED_PARAMETER(state);

	char buf[64];

	strcpy(buf, "  \t hi there \r\n");
	assert_ptr_equal(strdepad(buf), buf);
	assert_string_equal(buf, "hi there");

	strcpy(buf, "nopad");
	assert_ptr_equal(strdepad(buf), buf);
	assert_string_equal(buf, "nopad");

	strcpy(buf, "lead");
	memmove(buf + 3, buf, 5);
	memset(buf, ' ', 3);
	strdepad(buf);
	assert_string_equal(buf, "lead");

	strcpy(buf, "trail \t\n");
	strdepad(buf);
	assert_string_equal(buf, "trail");

	strcpy(buf, " \t\r\n ");
	assert_ptr_equal(strdepad(buf), buf);
	assert_string_equal(buf, "");

	strcpy(buf, "");
	assert_ptr_equal(strdepad(buf), buf);
	assert_string_equal(buf, "");

	assert_null(strdepad(NULL));
}

static void test_strlist_split(void **state)
{
	UNUSED_PARAMETER(state);

	char **list;

	list = strlist_split("a,b,,c", ',', true);
	assert_non_null(list);
	assert_string_equal(list[0], "a");
	assert_string_equal(list[1], "b");
	assert_string_equal(list[2], "");
	assert_string_equal(list[3], "c");
	assert_null(list[4]);
	strlist_free(list);

	list = strlist_split("a,b,,c", ',', false);
	assert_non_null(list);
	assert_string_equal(list[0], "a");
	assert_string_equal(list[1], "b");
	assert_string_equal(list[2], "c");
	assert_null(list[3]);
	strlist_free(list);

	/* trailing separator */
	list = strlist_split("a,b,", ',', true);
	assert_non_null(list);
	assert_string_equal(list[0], "a");
	assert_string_equal(list[1], "b");
	assert_string_equal(list[2], "");
	assert_null(list[3]);
	strlist_free(list);

	list = strlist_split("a,b,", ',', false);
	assert_non_null(list);
	assert_string_equal(list[0], "a");
	assert_string_equal(list[1], "b");
	assert_null(list[2]);
	strlist_free(list);

	/* leading separator */
	list = strlist_split(",a", ',', true);
	assert_non_null(list);
	assert_string_equal(list[0], "");
	assert_string_equal(list[1], "a");
	assert_null(list[2]);
	strlist_free(list);

	/* no separator present */
	list = strlist_split("abc", ',', false);
	assert_non_null(list);
	assert_string_equal(list[0], "abc");
	assert_null(list[1]);
	strlist_free(list);

	/* empty input */
	list = strlist_split("", ',', true);
	assert_non_null(list);
	assert_string_equal(list[0], "");
	assert_null(list[1]);
	strlist_free(list);

	list = strlist_split("", ',', false);
	assert_non_null(list);
	assert_null(list[0]);
	strlist_free(list);

	/* NULL input */
	assert_null(strlist_split(NULL, ',', true));
}

static void test_dstr_copy(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	dstr_copy(&d, "hello");
	assert_string_equal(d.array, "hello");
	assert_int_equal(d.len, 5);
	assert_true(d.capacity > d.len);

	/* copy shorter over longer */
	dstr_copy(&d, "hi");
	assert_string_equal(d.array, "hi");
	assert_int_equal(d.len, 2);
	assert_true(d.capacity > d.len);

	/* copy longer */
	dstr_copy(&d, "a considerably longer string");
	assert_string_equal(d.array, "a considerably longer string");
	assert_int_equal(d.len, 28);
	assert_true(d.capacity > d.len);

	/* NULL and empty free the destination */
	dstr_copy(&d, NULL);
	assert_null(d.array);
	assert_int_equal(d.len, 0);
	assert_int_equal(d.capacity, 0);

	dstr_copy(&d, "x");
	assert_string_equal(d.array, "x");
	dstr_copy(&d, "");
	assert_null(d.array);
	assert_int_equal(d.len, 0);
	assert_int_equal(d.capacity, 0);

	dstr_free(&d);
}

static void test_dstr_ncopy(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d, src;
	dstr_init(&d);
	dstr_init(&src);

	dstr_ncopy(&d, "hello world", 5);
	assert_string_equal(d.array, "hello");
	assert_int_equal(d.len, 5);
	assert_int_equal(d.capacity, 6);

	/* previous contents are replaced */
	dstr_ncopy(&d, "abcdef", 2);
	assert_string_equal(d.array, "ab");
	assert_int_equal(d.len, 2);
	assert_int_equal(d.capacity, 3);

	/* zero length frees */
	dstr_ncopy(&d, "abc", 0);
	assert_null(d.array);
	assert_int_equal(d.len, 0);
	assert_int_equal(d.capacity, 0);

	dstr_copy(&src, "abcdef");
	dstr_ncopy_dstr(&d, &src, 3);
	assert_string_equal(d.array, "abc");
	assert_int_equal(d.len, 3);
	assert_int_equal(d.capacity, 4);

	/* length is clamped to the source length */
	dstr_ncopy_dstr(&d, &src, 100);
	assert_string_equal(d.array, "abcdef");
	assert_int_equal(d.len, 6);
	assert_int_equal(d.capacity, 7);

	dstr_ncopy_dstr(&d, &src, 0);
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	dstr_free(&d);
	dstr_free(&src);
}

static void test_dstr_cat(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d, other;
	dstr_init(&d);
	dstr_init(&other);

	/* NULL / empty are no-ops and do not allocate */
	dstr_cat(&d, NULL);
	dstr_cat(&d, "");
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	dstr_cat(&d, "foo");
	assert_string_equal(d.array, "foo");
	assert_int_equal(d.len, 3);
	assert_true(d.capacity > d.len);

	dstr_cat(&d, "bar");
	assert_string_equal(d.array, "foobar");
	assert_int_equal(d.len, 6);
	assert_true(d.capacity > d.len);

	dstr_cat_ch(&d, '!');
	assert_string_equal(d.array, "foobar!");
	assert_int_equal(d.len, 7);
	assert_true(d.capacity > d.len);

	dstr_ncat(&d, "123456", 3);
	assert_string_equal(d.array, "foobar!123");
	assert_int_equal(d.len, 10);
	assert_true(d.capacity > d.len);

	/* zero length / NULL / empty are no-ops */
	dstr_ncat(&d, "zzz", 0);
	dstr_ncat(&d, NULL, 3);
	dstr_ncat(&d, "", 3);
	assert_string_equal(d.array, "foobar!123");
	assert_int_equal(d.len, 10);

	dstr_copy(&other, "-tail");
	dstr_cat_dstr(&d, &other);
	assert_string_equal(d.array, "foobar!123-tail");
	assert_int_equal(d.len, 15);
	assert_true(d.capacity > d.len);

	/* empty source is a no-op */
	dstr_free(&other);
	dstr_cat_dstr(&d, &other);
	assert_string_equal(d.array, "foobar!123-tail");
	assert_int_equal(d.len, 15);

	dstr_copy(&other, "abcdef");
	dstr_ncat_dstr(&d, &other, 2);
	assert_string_equal(d.array, "foobar!123-tailab");
	assert_int_equal(d.len, 17);

	/* length is clamped to the source length */
	dstr_ncat_dstr(&d, &other, 100);
	assert_string_equal(d.array, "foobar!123-tailababcdef");
	assert_int_equal(d.len, 23);
	assert_true(d.capacity > d.len);

	dstr_free(&d);
	dstr_free(&other);
}

static void test_dstr_insert(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d, other;
	dstr_init(&d);
	dstr_init(&other);

	/* inserting at len (0 here) is a cat */
	dstr_insert(&d, 0, "ad");
	assert_string_equal(d.array, "ad");
	assert_int_equal(d.len, 2);

	dstr_insert(&d, 1, "bc");
	assert_string_equal(d.array, "abcd");
	assert_int_equal(d.len, 4);
	assert_true(d.capacity > d.len);

	dstr_insert(&d, 4, "ef");
	assert_string_equal(d.array, "abcdef");
	assert_int_equal(d.len, 6);

	dstr_insert(&d, 0, "X");
	assert_string_equal(d.array, "Xabcdef");
	assert_int_equal(d.len, 7);
	assert_true(d.capacity > d.len);

	/* NULL / empty are no-ops */
	dstr_insert(&d, 2, NULL);
	dstr_insert(&d, 2, "");
	assert_string_equal(d.array, "Xabcdef");
	assert_int_equal(d.len, 7);

	dstr_copy(&other, "--");
	dstr_insert_dstr(&d, 3, &other);
	assert_string_equal(d.array, "Xab--cdef");
	assert_int_equal(d.len, 9);
	assert_true(d.capacity > d.len);

	dstr_insert_dstr(&d, 9, &other);
	assert_string_equal(d.array, "Xab--cdef--");
	assert_int_equal(d.len, 11);

	dstr_free(&other);
	dstr_insert_dstr(&d, 1, &other);
	assert_string_equal(d.array, "Xab--cdef--");
	assert_int_equal(d.len, 11);

	dstr_free(&d);
}

static void test_dstr_insert_ch(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	/* idx == len is a cat_ch, including on an empty dstr */
	dstr_insert_ch(&d, 0, 'a');
	assert_string_equal(d.array, "a");
	assert_int_equal(d.len, 1);

	dstr_insert_ch(&d, 1, 'c');
	assert_string_equal(d.array, "ac");
	assert_int_equal(d.len, 2);

	/* A mid-string insert_ch moves one byte more than needed (it writes
	 * array[new_len + 1]), which is past the buffer when capacity is exactly
	 * new_len + 1. Reserve room so the test never relies on that
	 * out-of-bounds write (characterized, not endorsed). */
	dstr_reserve(&d, 16);

	dstr_insert_ch(&d, 1, 'b');
	assert_string_equal(d.array, "abc");
	assert_int_equal(d.len, 3);
	assert_true(d.capacity > d.len);

	dstr_insert_ch(&d, 0, 'X');
	assert_string_equal(d.array, "Xabc");
	assert_int_equal(d.len, 4);
	assert_true(d.capacity > d.len);

	dstr_insert_ch(&d, 4, 'Z');
	assert_string_equal(d.array, "XabcZ");
	assert_int_equal(d.len, 5);

	dstr_free(&d);
}

static void test_dstr_remove(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	dstr_copy(&d, "abcdef");

	dstr_remove(&d, 1, 0);
	assert_string_equal(d.array, "abcdef");
	assert_int_equal(d.len, 6);

	/* middle */
	dstr_remove(&d, 1, 2);
	assert_string_equal(d.array, "adef");
	assert_int_equal(d.len, 4);
	assert_true(d.capacity > d.len);

	/* tail */
	dstr_remove(&d, 2, 2);
	assert_string_equal(d.array, "ad");
	assert_int_equal(d.len, 2);
	assert_true(d.capacity > d.len);

	/* head */
	dstr_remove(&d, 0, 1);
	assert_string_equal(d.array, "d");
	assert_int_equal(d.len, 1);

	/* removing everything frees */
	dstr_remove(&d, 0, 1);
	assert_null(d.array);
	assert_int_equal(d.len, 0);
	assert_int_equal(d.capacity, 0);

	dstr_free(&d);
}

static void test_dstr_replace(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	/* empty dstr is a no-op */
	dstr_replace(&d, "a", "b");
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	/* replacement shorter than find */
	dstr_copy(&d, "foo bar foo");
	dstr_replace(&d, "foo", "x");
	assert_string_equal(d.array, "x bar x");
	assert_int_equal(d.len, 7);
	assert_true(d.capacity > d.len);

	/* empty replacement */
	dstr_copy(&d, "foo bar foo");
	dstr_replace(&d, "foo", "");
	assert_string_equal(d.array, " bar ");
	assert_int_equal(d.len, 5);

	/* NULL replacement behaves like empty */
	dstr_copy(&d, "foo bar foo");
	dstr_replace(&d, "foo", NULL);
	assert_string_equal(d.array, " bar ");
	assert_int_equal(d.len, 5);

	/* replacement longer than find */
	dstr_copy(&d, "a-b-c");
	dstr_replace(&d, "-", "--");
	assert_string_equal(d.array, "a--b--c");
	assert_int_equal(d.len, 7);
	assert_true(d.capacity > d.len);

	dstr_copy(&d, "-start and end-");
	dstr_replace(&d, "-", "<longer>");
	assert_string_equal(d.array, "<longer>start and end<longer>");
	assert_int_equal(d.len, 29);
	assert_true(d.capacity > d.len);

	/* replacement same length as find */
	dstr_copy(&d, "abcabc");
	dstr_replace(&d, "abc", "xyz");
	assert_string_equal(d.array, "xyzxyz");
	assert_int_equal(d.len, 6);

	/* no match, every branch */
	dstr_copy(&d, "hello");
	dstr_replace(&d, "zzz", "y");
	assert_string_equal(d.array, "hello");
	assert_int_equal(d.len, 5);
	dstr_replace(&d, "zzz", "yyyyyy");
	assert_string_equal(d.array, "hello");
	assert_int_equal(d.len, 5);
	dstr_replace(&d, "zzz", "yyy");
	assert_string_equal(d.array, "hello");
	assert_int_equal(d.len, 5);

	/* an empty find string is deliberately not tested: the C code loops
	 * forever because strstr(temp, "") never advances temp. */

	dstr_free(&d);
}

static void test_dstr_depad(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	/* no array is a no-op */
	dstr_depad(&d);
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	dstr_copy(&d, "  \thi there \r\n");
	dstr_depad(&d);
	assert_string_equal(d.array, "hi there");
	assert_int_equal(d.len, 8);
	assert_true(d.capacity > d.len);

	dstr_copy(&d, "tight");
	dstr_depad(&d);
	assert_string_equal(d.array, "tight");
	assert_int_equal(d.len, 5);

	/* all whitespace frees */
	dstr_copy(&d, " \t\r\n ");
	dstr_depad(&d);
	assert_null(d.array);
	assert_int_equal(d.len, 0);
	assert_int_equal(d.capacity, 0);

	dstr_free(&d);
}

static void test_dstr_left_mid_right(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr src, d;
	dstr_init(&src);
	dstr_init(&d);

	dstr_copy(&src, "hello world");

	dstr_left(&d, &src, 5);
	assert_string_equal(d.array, "hello");
	assert_int_equal(d.len, 5);
	assert_true(d.capacity > d.len);

	/* in-place (dst == src) */
	dstr_left(&src, &src, 8);
	assert_string_equal(src.array, "hello wo");
	assert_int_equal(src.len, 8);
	assert_true(src.capacity > src.len);

	dstr_copy(&src, "hello world");

	dstr_mid(&d, &src, 6, 5);
	assert_string_equal(d.array, "world");
	assert_int_equal(d.len, 5);
	assert_true(d.capacity > d.len);

	dstr_mid(&d, &src, 2, 3);
	assert_string_equal(d.array, "llo");
	assert_int_equal(d.len, 3);

	/* zero count frees dst */
	dstr_mid(&d, &src, 2, 0);
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	dstr_right(&d, &src, 6);
	assert_string_equal(d.array, "world");
	assert_int_equal(d.len, 5);
	assert_true(d.capacity > d.len);

	dstr_right(&d, &src, 0);
	assert_string_equal(d.array, "hello world");
	assert_int_equal(d.len, 11);

	/* pos == len leaves dst empty */
	dstr_right(&d, &src, 11);
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	dstr_free(&d);
	dstr_free(&src);
}

#ifndef _WIN32
static void test_dstr_case(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	/* empty is a no-op */
	dstr_to_upper(&d);
	dstr_to_lower(&d);
	assert_null(d.array);

	dstr_copy(&d, "Hello, World 123");
	dstr_to_upper(&d);
	assert_string_equal(d.array, "HELLO, WORLD 123");
	assert_int_equal(d.len, 16);
	assert_true(d.capacity > d.len);

	dstr_to_lower(&d);
	assert_string_equal(d.array, "hello, world 123");
	assert_int_equal(d.len, 16);
	assert_true(d.capacity > d.len);

	dstr_free(&d);
}
#endif

static void test_dstr_printf(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	dstr_printf(&d, "%d-%s", 42, "abc");
	assert_string_equal(d.array, "42-abc");
	assert_int_equal(d.len, 6);
	assert_true(d.capacity > d.len);

	/* overwrites previous contents */
	dstr_printf(&d, "%d-%s", 7, "x");
	assert_string_equal(d.array, "7-x");
	assert_int_equal(d.len, 3);
	assert_true(d.capacity > d.len);

	dstr_catf(&d, " [%03d]", 7);
	assert_string_equal(d.array, "7-x [007]");
	assert_int_equal(d.len, 9);
	assert_true(d.capacity > d.len);

	/* printf producing an empty string frees */
	dstr_printf(&d, "%s", "");
	assert_null(d.array);
	assert_int_equal(d.len, 0);
	assert_int_equal(d.capacity, 0);

	/* catf onto a fresh dstr */
	dstr_catf(&d, "%s=%d", "k", 1);
	assert_string_equal(d.array, "k=1");
	assert_int_equal(d.len, 3);
	assert_true(d.capacity > d.len);

	dstr_free(&d);
}

static void test_dstr_safe_printf(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	dstr_safe_printf(&d, "$1 and $2 and $3 and $4", "a", "bb", "ccc", "dddd");
	assert_string_equal(d.array, "a and bb and ccc and dddd");
	assert_int_equal(d.len, 25);
	assert_true(d.capacity > d.len);

	/* NULL values leave their placeholder untouched */
	dstr_safe_printf(&d, "x=$1 y=$2", "1", NULL, NULL, NULL);
	assert_string_equal(d.array, "x=1 y=$2");
	assert_int_equal(d.len, 8);

	/* repeated placeholder */
	dstr_safe_printf(&d, "$1$1", "ab", NULL, NULL, NULL);
	assert_string_equal(d.array, "abab");
	assert_int_equal(d.len, 4);

	/* empty value removes the placeholder */
	dstr_safe_printf(&d, "[$1]", "", NULL, NULL, NULL);
	assert_string_equal(d.array, "[]");
	assert_int_equal(d.len, 2);

	/* NULL format frees */
	dstr_safe_printf(&d, NULL, "a", "b", "c", "d");
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	dstr_free(&d);
}

static void test_dstr_strref(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	struct strref ref;
	dstr_init(&d);

	strref_set(&ref, "hello world", 5);
	dstr_copy_strref(&d, &ref);
	assert_string_equal(d.array, "hello");
	assert_int_equal(d.len, 5);
	assert_int_equal(d.capacity, 6);

	/* replaces previous contents */
	strref_set(&ref, "abcdef", 3);
	dstr_copy_strref(&d, &ref);
	assert_string_equal(d.array, "abc");
	assert_int_equal(d.len, 3);

	dstr_cat_strref(&d, &ref);
	assert_string_equal(d.array, "abcabc");
	assert_int_equal(d.len, 6);
	assert_true(d.capacity > d.len);

	/* empty strref */
	strref_set(&ref, "xyz", 0);
	dstr_cat_strref(&d, &ref);
	assert_string_equal(d.array, "abcabc");
	assert_int_equal(d.len, 6);

	dstr_copy_strref(&d, &ref);
	assert_null(d.array);
	assert_int_equal(d.len, 0);

	dstr_free(&d);

	strref_set(&ref, "init-copy", 4);
	dstr_init_copy_strref(&d, &ref);
	assert_string_equal(d.array, "init");
	assert_int_equal(d.len, 4);
	assert_int_equal(d.capacity, 5);

	dstr_free(&d);
}

static void test_dstr_inline_helpers(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr d;
	dstr_init(&d);

	assert_true(dstr_is_empty(&d));
	assert_int_equal(dstr_end(&d), 0);
	assert_int_equal(dstr_cmp(&d, NULL), 0);
	assert_int_equal(dstr_cmp(&d, ""), 0);

	dstr_copy(&d, "Hello");
	assert_false(dstr_is_empty(&d));
	assert_int_equal(dstr_end(&d), 'o');
	assert_int_equal(dstr_cmp(&d, "Hello"), 0);
	assert_true(dstr_cmp(&d, "hello") < 0);
	assert_int_equal(dstr_cmpi(&d, "hELLO"), 0);
	assert_int_equal(dstr_ncmp(&d, "Help", 3), 0);
	assert_int_equal(dstr_ncmpi(&d, "HELP", 3), 0);
	assert_ptr_equal(dstr_find(&d, "llo"), d.array + 2);
	assert_ptr_equal(dstr_find_i(&d, "LLO"), d.array + 2);
	assert_null(dstr_find(&d, "LLO"));

	dstr_free(&d);
}

int main()
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test_setup_teardown(test_astrcmpi, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_astrcmp_n, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_astrcmpi_n, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_astrstri, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_strdepad, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_strlist_split, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_copy, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_ncopy, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_cat, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_insert, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_insert_ch, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_remove, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_replace, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_depad, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_left_mid_right, leak_setup, leak_teardown),
#ifndef _WIN32
		cmocka_unit_test_setup_teardown(test_dstr_case, leak_setup, leak_teardown),
#endif
		cmocka_unit_test_setup_teardown(test_dstr_printf, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_safe_printf, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_strref, leak_setup, leak_teardown),
		cmocka_unit_test_setup_teardown(test_dstr_inline_helpers, leak_setup, leak_teardown),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
