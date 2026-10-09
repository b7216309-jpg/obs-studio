#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <obs.h>
#include <util/bmem.h>
#include <util/dstr.h>
#include <util/platform.h>

/* platform.c is compiled into this test (see CMakeLists.txt), so this fixture
 * replaces libobs's obs_get_video_info and no obs core needs to be started. */
bool obs_get_video_info(struct obs_video_info *ovi)
{
	memset(ovi, 0, sizeof(*ovi));
	ovi->fps_num = 30;
	ovi->fps_den = 1;
	return true;
}

/* Strict UTF-8 check: rejects truncated, overlong and surrogate sequences. */
static bool is_valid_utf8(const unsigned char *s, size_t len)
{
	size_t i = 0;
	while (i < len) {
		unsigned char c = s[i];
		size_t n;
		unsigned int cp;

		if (c < 0x80) {
			i++;
			continue;
		} else if ((c & 0xE0) == 0xC0) {
			n = 1;
			cp = c & 0x1F;
		} else if ((c & 0xF0) == 0xE0) {
			n = 2;
			cp = c & 0x0F;
		} else if ((c & 0xF8) == 0xF0) {
			n = 3;
			cp = c & 0x07;
		} else {
			return false;
		}

		if (i + n >= len)
			return false;
		for (size_t k = 1; k <= n; k++) {
			if ((s[i + k] & 0xC0) != 0x80)
				return false;
			cp = (cp << 6) | (s[i + k] & 0x3F);
		}

		if ((n == 1 && cp < 0x80) || (n == 2 && cp < 0x800) || (n == 3 && cp < 0x10000) || cp > 0x10FFFF ||
		    (cp >= 0xD800 && cp <= 0xDFFF))
			return false;
		i += n + 1;
	}
	return true;
}

/* The upstream report (obsproject/obs-studio#11211): a 23-byte prefix
 * followed by U+200B (3 bytes) puts the 255-byte cut inside a character. */
static void zero_width_space_report_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr fmt = {0};
	dstr_copy(&fmt, "%CCYY-%MM-%DD %hh-%mm-%ss foo");
	for (int i = 0; i < 100; i++)
		dstr_cat(&fmt, "\xE2\x80\x8B");
	dstr_cat(&fmt, "bar");

	char *name = os_generate_formatted_filename("mkv", true, fmt.array);
	size_t len = strlen(name);

	/* "YYYY-MM-DD hh-mm-ss foo" is 23 bytes; 77 full U+200B fit in 254. */
	assert_int_equal(len, 254);
	assert_true(is_valid_utf8((const unsigned char *)name, len));

	bfree(name);
	dstr_free(&fmt);
}

/* Every interior cut of a 2-, 3- and 4-byte character at the 255-byte limit
 * must drop the whole character; a character ending exactly at byte 255 must
 * be kept. */
static void every_interior_cut_test(void **state)
{
	UNUSED_PARAMETER(state);

	static const char *chars[] = {"\xC3\xA9", "\xE2\x80\x8B", "\xF0\x9F\x98\x80"};

	for (size_t c = 0; c < sizeof(chars) / sizeof(chars[0]); c++) {
		size_t clen = strlen(chars[c]);

		/* offset = bytes of the character that fit before the limit */
		for (size_t fit = 0; fit <= clen; fit++) {
			size_t prefix = 255 - fit;
			struct dstr fmt = {0};
			for (size_t i = 0; i < prefix; i++)
				dstr_cat_ch(&fmt, 'a');
			dstr_cat(&fmt, chars[c]);
			dstr_cat(&fmt, "tail");

			char *name = os_generate_formatted_filename(NULL, true, fmt.array);
			size_t len = strlen(name);
			size_t expected = fit == clen ? 255 : prefix;

			printf("char %zu-byte, %zu byte(s) before limit: len %zu\n", clen, fit, len);
			assert_int_equal(len, expected);
			assert_true(is_valid_utf8((const unsigned char *)name, len));

			bfree(name);
			dstr_free(&fmt);
		}
	}
}

static void ascii_truncation_unchanged_test(void **state)
{
	UNUSED_PARAMETER(state);

	struct dstr fmt = {0};
	for (int i = 0; i < 300; i++)
		dstr_cat_ch(&fmt, 'x');

	char *name = os_generate_formatted_filename("mp4", false, fmt.array);
	assert_int_equal(strlen(name), 255);
	assert_memory_equal(name, fmt.array, 255);

	bfree(name);
	dstr_free(&fmt);
}

static void short_name_unchanged_test(void **state)
{
	UNUSED_PARAMETER(state);

	char *name = os_generate_formatted_filename("mkv", false, "caf\xC3\xA9 %FPS");
	assert_string_equal(name, "caf\xC3\xA9_30.mkv");
	bfree(name);
}

int main()
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(zero_width_space_report_test),
		cmocka_unit_test(every_interior_cut_test),
		cmocka_unit_test(ascii_truncation_unchanged_test),
		cmocka_unit_test(short_name_unchanged_test),
	};

	return cmocka_run_group_tests(tests, NULL, NULL);
}
