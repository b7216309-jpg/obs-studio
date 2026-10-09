#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <cmocka.h>

#include <string.h>

#include <util/bmem.h>
#include <util/buffered-file-serializer.h>
#include <util/file-serializer.h>
#include <util/platform.h>
#include <util/serializer.h>

/*
 * Characterization tests for libobs/util/file-serializer.c and
 * libobs/util/buffered-file-serializer.c. ctest runs with cwd = build/test/cmocka,
 * so all files live in a relative scratch directory created by the group setup
 * and removed by the group teardown.
 */

#define TMP_DIR "test_file_serializer.tmp"

#define PATH_A TMP_DIR "/a.bin"
#define PATH_SAFE TMP_DIR "/safe.bin"
#define PATH_SAFE_TMP TMP_DIR "/safe.bin.tmp"
#define PATH_SAFE_DOT_TMP TMP_DIR "/safe.bin.tmp"
#define PATH_BUF TMP_DIR "/buf.bin"
#define PATH_MISSING TMP_DIR "/missing.bin"
#define PATH_NO_SUBDIR TMP_DIR "/no_such_subdir/x.bin"

/* every file any test may create; the group teardown removes them all */
static const char *const tmp_files[] = {
	PATH_A, PATH_SAFE, PATH_SAFE_TMP, PATH_SAFE_DOT_TMP, PATH_BUF, PATH_MISSING, PATH_NO_SUBDIR,
};

static int group_setup(void **state)
{
	UNUSED_PARAMETER(state);

	return os_mkdirs(TMP_DIR) == MKDIR_ERROR ? -1 : 0;
}

static int group_teardown(void **state)
{
	UNUSED_PARAMETER(state);

	for (size_t i = 0; i < sizeof(tmp_files) / sizeof(tmp_files[0]); i++)
		os_unlink(tmp_files[i]);
	os_rmdir(TMP_DIR);
	return 0;
}

static void clean_all(void)
{
	for (size_t i = 0; i < sizeof(tmp_files) / sizeof(tmp_files[0]); i++)
		os_unlink(tmp_files[i]);
}

/* write whole file through the plain file output serializer */
static void write_file(const char *path, const void *data, size_t size)
{
	struct serializer s = {0};

	assert_true(file_output_serializer_init(&s, path));
	assert_int_equal(s_write(&s, data, size), size);
	file_output_serializer_free(&s);
}

/* read whole file with stdio; returns bytes read */
static size_t read_file(const char *path, void *buf, size_t cap)
{
	FILE *f = os_fopen(path, "rb");
	size_t n;

	assert_non_null(f);
	n = fread(buf, 1, cap, f);
	fclose(f);
	return n;
}

static void test_output_input_roundtrip(void **state)
{
	UNUSED_PARAMETER(state);

	static const uint8_t expected[] = {
		0xAA,                   /* s_w8 */
		0x01, 0x02, 0x03, 0x04, /* s_wl32(0x04030201) */
		0x12, 0x34,             /* s_wb16(0x1234) */
		'h',  'e',  'l',  'l',  'o', /* s_write */
	};
	uint8_t back[sizeof(expected)] = {0};
	struct serializer s = {0};

	clean_all();

	assert_true(file_output_serializer_init(&s, PATH_A));
	assert_non_null(s.write);
	assert_null(s.read);
	s_w8(&s, 0xAA);
	s_wl32(&s, 0x04030201);
	s_wb16(&s, 0x1234);
	assert_int_equal(s_write(&s, "hello", 5), 5);
	/* s_write ignores empty and NULL data */
	assert_int_equal(s_write(&s, "x", 0), 0);
	assert_int_equal(s_write(&s, NULL, 3), 0);
	/* reading from an output serializer yields nothing */
	assert_int_equal(s_read(&s, back, 1), 0);
	assert_int_equal(serializer_get_pos(&s), sizeof(expected));
	file_output_serializer_free(&s);

	assert_int_equal(os_get_file_size(PATH_A), (int64_t)sizeof(expected));

	memset(&s, 0, sizeof(s));
	assert_true(file_input_serializer_init(&s, PATH_A));
	assert_non_null(s.read);
	assert_null(s.write);
	assert_int_equal(s_read(&s, back, sizeof(back)), sizeof(back));
	assert_memory_equal(back, expected, sizeof(expected));
	/* at EOF */
	assert_int_equal(s_read(&s, back, 1), 0);
	assert_int_equal(serializer_get_pos(&s), sizeof(expected));
	/* writing to an input serializer writes nothing */
	assert_int_equal(s_write(&s, "x", 1), 0);
	file_input_serializer_free(&s);

	clean_all();
}

static void test_serialize_helper(void **state)
{
	UNUSED_PARAMETER(state);

	char data[4] = {'a', 'b', 'c', 'd'};
	char back[4] = {0};
	struct serializer s = {0};

	clean_all();

	assert_true(file_output_serializer_init(&s, PATH_A));
	assert_int_equal(serialize(&s, data, sizeof(data)), sizeof(data));
	file_output_serializer_free(&s);

	memset(&s, 0, sizeof(s));
	assert_true(file_input_serializer_init(&s, PATH_A));
	assert_int_equal(serialize(&s, back, sizeof(back)), sizeof(back));
	assert_memory_equal(back, data, sizeof(data));
	file_input_serializer_free(&s);

	/* NULL serializer */
	assert_int_equal(serialize(NULL, data, 4), 0);
	assert_int_equal(serializer_get_pos(NULL), -1);
	assert_int_equal(serializer_seek(NULL, 0, SERIALIZE_SEEK_START), -1);

	clean_all();
}

static void test_output_seek_overwrite(void **state)
{
	UNUSED_PARAMETER(state);

	static const char expected[] = "0123X5678Z";
	char back[16] = {0};
	struct serializer s = {0};

	clean_all();

	assert_true(file_output_serializer_init(&s, PATH_A));
	assert_int_equal(serializer_get_pos(&s), 0);
	assert_int_equal(s_write(&s, "0123456789", 10), 10);
	assert_int_equal(serializer_get_pos(&s), 10);

	assert_int_equal(serializer_seek(&s, 4, SERIALIZE_SEEK_START), 4);
	assert_int_equal(serializer_get_pos(&s), 4);
	s_w8(&s, 'X');
	assert_int_equal(serializer_get_pos(&s), 5);

	assert_int_equal(serializer_seek(&s, 2, SERIALIZE_SEEK_CURRENT), 7);
	assert_int_equal(serializer_seek(&s, -1, SERIALIZE_SEEK_END), 9);
	s_w8(&s, 'Z');
	assert_int_equal(serializer_get_pos(&s), 10);
	file_output_serializer_free(&s);

	assert_int_equal(os_get_file_size(PATH_A), 10);
	assert_int_equal(read_file(PATH_A, back, sizeof(back)), 10);
	assert_memory_equal(back, expected, 10);

	clean_all();
}

static void test_input_seek(void **state)
{
	UNUSED_PARAMETER(state);

	char buf[4] = {0};
	struct serializer s = {0};

	clean_all();
	write_file(PATH_A, "ABCDEFGHIJ", 10);

	assert_true(file_input_serializer_init(&s, PATH_A));
	assert_int_equal(serializer_get_pos(&s), 0);

	assert_int_equal(s_read(&s, buf, 3), 3);
	assert_memory_equal(buf, "ABC", 3);
	assert_int_equal(serializer_get_pos(&s), 3);

	assert_int_equal(serializer_seek(&s, 6, SERIALIZE_SEEK_START), 6);
	assert_int_equal(s_read(&s, buf, 2), 2);
	assert_memory_equal(buf, "GH", 2);

	assert_int_equal(serializer_seek(&s, -4, SERIALIZE_SEEK_CURRENT), 4);
	assert_int_equal(s_read(&s, buf, 1), 1);
	assert_memory_equal(buf, "E", 1);

	assert_int_equal(serializer_seek(&s, -2, SERIALIZE_SEEK_END), 8);
	assert_int_equal(s_read(&s, buf, 2), 2);
	assert_memory_equal(buf, "IJ", 2);

	assert_int_equal(serializer_seek(&s, 0, SERIALIZE_SEEK_END), 10);
	assert_int_equal(s_read(&s, buf, 1), 0);

	/* a short read returns the bytes that were available */
	assert_int_equal(serializer_seek(&s, 8, SERIALIZE_SEEK_START), 8);
	assert_int_equal(s_read(&s, buf, 4), 2);

	/* seeking before the start of the file fails */
	assert_int_equal(serializer_seek(&s, -1, SERIALIZE_SEEK_START), -1);

	file_input_serializer_free(&s);
	clean_all();
}

static void test_input_missing_file(void **state)
{
	UNUSED_PARAMETER(state);

	struct serializer s = {0};

	clean_all();

	assert_false(os_file_exists(PATH_MISSING));
	assert_false(file_input_serializer_init(&s, PATH_MISSING));
	assert_null(s.data);
	assert_null(s.read);
}

static void test_output_unwritable_path(void **state)
{
	UNUSED_PARAMETER(state);

	struct serializer s = {0};

	clean_all();

	assert_false(file_output_serializer_init(&s, PATH_NO_SUBDIR));
	assert_null(s.data);
	assert_false(file_output_serializer_init_safe(&s, PATH_NO_SUBDIR, "tmp"));
	assert_null(s.data);
	assert_false(buffered_file_serializer_init_defaults(&s, PATH_NO_SUBDIR));
	assert_null(s.data);
	assert_false(buffered_file_serializer_init(&s, PATH_NO_SUBDIR, 64, 8));
	assert_null(s.data);
	assert_false(os_file_exists(PATH_NO_SUBDIR));
}

static void test_safe_rejects_bad_temp_ext(void **state)
{
	UNUSED_PARAMETER(state);

	struct serializer s = {0};

	clean_all();

	assert_false(file_output_serializer_init_safe(&s, PATH_SAFE, NULL));
	assert_false(file_output_serializer_init_safe(&s, PATH_SAFE, ""));
	assert_null(s.data);
	assert_false(os_file_exists(PATH_SAFE));
	assert_false(os_file_exists(PATH_SAFE_TMP));
}

static void safe_new_target(const char *temp_ext)
{
	char back[8] = {0};
	struct serializer s = {0};

	clean_all();

	assert_true(file_output_serializer_init_safe(&s, PATH_SAFE, temp_ext));
	assert_int_equal(s_write(&s, "payload", 7), 7);

	/* data goes to path + ".tmp" (a dot is inserted only when missing) */
	assert_true(os_file_exists(PATH_SAFE_TMP));
	assert_false(os_file_exists(PATH_SAFE));

	file_output_serializer_free(&s);

	assert_false(os_file_exists(PATH_SAFE_TMP));
	assert_true(os_file_exists(PATH_SAFE));
	assert_int_equal(os_get_file_size(PATH_SAFE), 7);
	assert_int_equal(read_file(PATH_SAFE, back, sizeof(back)), 7);
	assert_memory_equal(back, "payload", 7);

	clean_all();
}

static void test_safe_temp_ext_without_dot(void **state)
{
	UNUSED_PARAMETER(state);

	safe_new_target("tmp");
}

static void test_safe_temp_ext_with_dot(void **state)
{
	UNUSED_PARAMETER(state);

	safe_new_target(".tmp");
}

static void test_safe_replaces_existing(void **state)
{
	UNUSED_PARAMETER(state);

	char back[8] = {0};
	struct serializer s = {0};

	clean_all();
	write_file(PATH_SAFE, "old", 3);

	assert_true(file_output_serializer_init_safe(&s, PATH_SAFE, "tmp"));
	assert_int_equal(s_write(&s, "newdata", 7), 7);

	/* the original is untouched until free */
	assert_true(os_file_exists(PATH_SAFE_TMP));
	assert_int_equal(os_get_file_size(PATH_SAFE), 3);

	file_output_serializer_free(&s);

	assert_false(os_file_exists(PATH_SAFE_TMP));
	assert_int_equal(os_get_file_size(PATH_SAFE), 7);
	assert_int_equal(read_file(PATH_SAFE, back, sizeof(back)), 7);
	assert_memory_equal(back, "newdata", 7);

	clean_all();
}

static void test_safe_seek_and_pos(void **state)
{
	UNUSED_PARAMETER(state);

	char back[8] = {0};
	struct serializer s = {0};

	clean_all();

	assert_true(file_output_serializer_init_safe(&s, PATH_SAFE, "tmp"));
	assert_int_equal(s_write(&s, "abcdef", 6), 6);
	assert_int_equal(serializer_get_pos(&s), 6);
	assert_int_equal(serializer_seek(&s, 1, SERIALIZE_SEEK_START), 1);
	s_w8(&s, 'Z');
	file_output_serializer_free(&s);

	assert_int_equal(read_file(PATH_SAFE, back, sizeof(back)), 6);
	assert_memory_equal(back, "aZcdef", 6);

	clean_all();
}

static void test_buffered_defaults(void **state)
{
	UNUSED_PARAMETER(state);

	uint8_t data[20];
	uint8_t back[32] = {0};
	struct serializer s = {0};

	clean_all();
	for (size_t i = 0; i < sizeof(data); i++)
		data[i] = (uint8_t)(i * 3 + 1);

	assert_true(buffered_file_serializer_init_defaults(&s, PATH_BUF));
	assert_non_null(s.write);
	assert_null(s.read);
	assert_int_equal(serializer_get_pos(&s), 0);
	assert_int_equal(s_write(&s, data, sizeof(data)), sizeof(data));
	assert_int_equal(serializer_get_pos(&s), sizeof(data));
	/* zero-length write is accepted and does nothing */
	assert_int_equal(s.write(s.data, data, 0), 0);
	assert_int_equal(serializer_get_pos(&s), sizeof(data));
	buffered_file_serializer_free(&s);

	assert_int_equal(os_get_file_size(PATH_BUF), (int64_t)sizeof(data));
	assert_int_equal(read_file(PATH_BUF, back, sizeof(back)), sizeof(data));
	assert_memory_equal(back, data, sizeof(data));

	clean_all();
}

static void test_buffered_small_chunks(void **state)
{
	UNUSED_PARAMETER(state);

	/* 1000 bytes through an 8 byte chunk size and a 64 byte max buffer */
	uint8_t data[1000];
	uint8_t back[1024] = {0};
	struct serializer s = {0};

	clean_all();
	for (size_t i = 0; i < sizeof(data); i++)
		data[i] = (uint8_t)(i ^ (i >> 8) ^ 0x5A);

	assert_true(buffered_file_serializer_init(&s, PATH_BUF, 64, 8));
	assert_int_equal(s_write(&s, data, sizeof(data)), sizeof(data));
	assert_int_equal(serializer_get_pos(&s), (int64_t)sizeof(data));
	buffered_file_serializer_free(&s);

	assert_int_equal(os_get_file_size(PATH_BUF), (int64_t)sizeof(data));
	assert_int_equal(read_file(PATH_BUF, back, sizeof(back)), sizeof(data));
	assert_memory_equal(back, data, sizeof(data));

	clean_all();
}

static void test_buffered_seek_overwrite(void **state)
{
	UNUSED_PARAMETER(state);

	uint8_t expected[100];
	uint8_t back[128] = {0};
	struct serializer s = {0};

	clean_all();
	for (size_t i = 0; i < sizeof(expected); i++)
		expected[i] = (uint8_t)i;

	assert_true(buffered_file_serializer_init(&s, PATH_BUF, 64, 8));
	assert_int_equal(s_write(&s, expected, sizeof(expected)), sizeof(expected));
	assert_int_equal(serializer_get_pos(&s), 100);

	/* absolute seek, then overwrite 3 bytes in the middle */
	assert_int_equal(serializer_seek(&s, 10, SERIALIZE_SEEK_START), 10);
	assert_int_equal(serializer_get_pos(&s), 10);
	assert_int_equal(s_write(&s, "XYZ", 3), 3);
	assert_int_equal(serializer_get_pos(&s), 13);
	memcpy(expected + 10, "XYZ", 3);

	/* relative seek forward by 7 -> 20 */
	assert_int_equal(serializer_seek(&s, 7, SERIALIZE_SEEK_CURRENT), 20);
	s_w8(&s, 'Q');
	expected[20] = 'Q';
	assert_int_equal(serializer_get_pos(&s), 21);

	/*
	 * characterized, not endorsed: SERIALIZE_SEEK_END is relative to the
	 * current write position, subtracting the offset (it does not use the
	 * file size, unlike the plain file serializer): 21 - 5 = 16.
	 */
	assert_int_equal(serializer_seek(&s, 5, SERIALIZE_SEEK_END), 16);
	assert_int_equal(serializer_get_pos(&s), 16);
	s_w8(&s, 'E');
	expected[16] = 'E';
	assert_int_equal(serializer_get_pos(&s), 17);

	buffered_file_serializer_free(&s);

	assert_int_equal(os_get_file_size(PATH_BUF), 100);
	assert_int_equal(read_file(PATH_BUF, back, sizeof(back)), 100);
	assert_memory_equal(back, expected, sizeof(expected));

	clean_all();
}

static void test_buffered_seek_past_end(void **state)
{
	UNUSED_PARAMETER(state);

	uint8_t back[16] = {0};
	struct serializer s = {0};

	clean_all();

	assert_true(buffered_file_serializer_init(&s, PATH_BUF, 64, 8));
	assert_int_equal(s_write(&s, "ab", 2), 2);
	assert_int_equal(serializer_seek(&s, 6, SERIALIZE_SEEK_START), 6);
	assert_int_equal(s_write(&s, "cd", 2), 2);
	assert_int_equal(serializer_get_pos(&s), 8);
	buffered_file_serializer_free(&s);

	/* the gap bytes are whatever the filesystem fills; check the ends */
	assert_int_equal(os_get_file_size(PATH_BUF), 8);
	assert_int_equal(read_file(PATH_BUF, back, sizeof(back)), 8);
	assert_memory_equal(back, "ab", 2);
	assert_memory_equal(back + 6, "cd", 2);

	clean_all();
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_output_input_roundtrip),
		cmocka_unit_test(test_serialize_helper),
		cmocka_unit_test(test_output_seek_overwrite),
		cmocka_unit_test(test_input_seek),
		cmocka_unit_test(test_input_missing_file),
		cmocka_unit_test(test_output_unwritable_path),
		cmocka_unit_test(test_safe_rejects_bad_temp_ext),
		cmocka_unit_test(test_safe_temp_ext_without_dot),
		cmocka_unit_test(test_safe_temp_ext_with_dot),
		cmocka_unit_test(test_safe_replaces_existing),
		cmocka_unit_test(test_safe_seek_and_pos),
		cmocka_unit_test(test_buffered_defaults),
		cmocka_unit_test(test_buffered_small_chunks),
		cmocka_unit_test(test_buffered_seek_overwrite),
		cmocka_unit_test(test_buffered_seek_past_end),
	};

	return cmocka_run_group_tests(tests, group_setup, group_teardown);
}
