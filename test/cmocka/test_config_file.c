#include <stdarg.h>
#include <stddef.h>
#include <setjmp.h>
#include <string.h>
#include <cmocka.h>

#include <util/config-file.h>
#include <util/platform.h>
#include <util/bmem.h>

/*
 * Characterization tests for libobs/util/config-file.c: they pin CURRENT
 * behavior, including odd behavior. Cases marked "characterized, not endorsed"
 * are quirks of the C implementation that a port must reproduce, not
 * behavior anyone designed.
 */

#define TMP_DIR "test_config_file.tmp"

#define OPEN_PATH TMP_DIR "/open.ini"
#define MISSING_PATH TMP_DIR "/missing.ini"
#define ALWAYS_PATH TMP_DIR "/always.ini"
#define CREATE_PATH TMP_DIR "/create.ini"
#define EMPTY_PATH TMP_DIR "/empty.ini"
#define DEFSAVE_PATH TMP_DIR "/defsave.ini"
#define ROUNDTRIP_PATH TMP_DIR "/roundtrip.ini"
#define ESCAPE_PATH TMP_DIR "/escape.ini"
#define DEFAULTS_PATH TMP_DIR "/defaults.ini"
#define SAFE1_PATH TMP_DIR "/safe1.ini"
#define SAFE2_PATH TMP_DIR "/safe2.ini"
#define SAFE3_PATH TMP_DIR "/safe3.ini"
#define SAFE4_PATH TMP_DIR "/safe4.ini"
#define BADDIR_PATH TMP_DIR "/nodir/x.ini"

static const char *const test_files[] = {
	OPEN_PATH,      MISSING_PATH,      ALWAYS_PATH,       CREATE_PATH, EMPTY_PATH,        DEFSAVE_PATH,
	ROUNDTRIP_PATH, ESCAPE_PATH,       DEFAULTS_PATH,     SAFE1_PATH,  SAFE1_PATH ".tmp", SAFE1_PATH ".bak",
	SAFE2_PATH,     SAFE2_PATH ".tmp", SAFE2_PATH ".bak", SAFE3_PATH,  SAFE3_PATH ".tmp", SAFE3_PATH ".bak",
	SAFE4_PATH,     SAFE4_PATH ".tmp", SAFE4_PATH ".bak",
};

static void remove_test_files(void)
{
	for (size_t i = 0; i < sizeof(test_files) / sizeof(test_files[0]); i++)
		os_unlink(test_files[i]);
}

static int group_setup(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	return os_mkdirs(TMP_DIR) == MKDIR_ERROR ? -1 : 0;
}

static int group_teardown(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	os_rmdir(TMP_DIR);
	return 0;
}

static config_t *open_str(const char *str)
{
	config_t *config = NULL;
	int ret = config_open_string(&config, str);

	assert_int_equal(ret, CONFIG_SUCCESS);
	assert_non_null(config);
	return config;
}

static void expect_str(config_t *config, const char *section, const char *name, const char *expected)
{
	const char *value = config_get_string(config, section, name);

	assert_non_null(value);
	assert_string_equal(value, expected);
}

static void expect_missing(config_t *config, const char *section, const char *name)
{
	assert_null(config_get_string(config, section, name));
}

static void expect_section(config_t *config, size_t idx, const char *expected)
{
	const char *name = config_get_section(config, idx);

	assert_non_null(name);
	assert_string_equal(name, expected);
}

static void write_file(const char *path, const char *text)
{
	assert_true(os_quick_write_utf8_file(path, text, strlen(text), false));
}

static void expect_file(const char *path, const char *expected)
{
	char *text = os_quick_read_utf8_file(path);

	assert_non_null(text);
	assert_string_equal(text, expected);
	bfree(text);
}

static void expect_file_empty(const char *path)
{
	/* an empty (or BOM-only) file reads back as NULL */
	assert_true(os_file_exists(path));
	assert_null(os_quick_read_utf8_file(path));
}

/* ------------------------------------------------------------------------- */
/* parsing */

static void test_parse_basic(void **state)
{
	UNUSED_PARAMETER(state);

	/* sections keep file order, they are not sorted */
	config_t *c = open_str("[General]\nname=OBS\ncount=42\n\n[Video]\nfps=60\n[Audio]\nrate=48000\n");

	assert_int_equal(config_num_sections(c), 3);
	expect_section(c, 0, "General");
	expect_section(c, 1, "Video");
	expect_section(c, 2, "Audio");
	assert_null(config_get_section(c, 3));
	assert_null(config_get_section(c, 100));

	expect_str(c, "General", "name", "OBS");
	expect_str(c, "General", "count", "42");
	expect_str(c, "Video", "fps", "60");
	expect_str(c, "Audio", "rate", "48000");

	/* names are case sensitive */
	expect_missing(c, "general", "name");
	expect_missing(c, "General", "Name");
	expect_missing(c, "General", "fps");
	expect_missing(c, "Nope", "name");

	config_close(c);
}

static void test_parse_empty_string(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("");

	assert_int_equal(config_num_sections(c), 0);
	assert_null(config_get_section(c, 0));
	config_close(c);

	c = open_str("\n\n   \n\t\n");
	assert_int_equal(config_num_sections(c), 0);
	config_close(c);
}

static void test_parse_whitespace_is_kept(void **state)
{
	UNUSED_PARAMETER(state);

	/*
	 * Characterized, not endorsed: only whitespace at the start of a line is
	 * skipped. Whitespace inside [ ], around '=' and at the end of a value is
	 * part of the section name, key name and value.
	 */
	config_t *c = open_str("[ Sp ace ]\n  key one = value two \n\tk2\t=\tv2\n  lead=x\n");

	assert_int_equal(config_num_sections(c), 1);
	expect_section(c, 0, " Sp ace ");
	expect_str(c, " Sp ace ", "key one ", " value two ");
	expect_str(c, " Sp ace ", "k2\t", "\tv2");
	expect_str(c, " Sp ace ", "lead", "x");

	expect_missing(c, " Sp ace ", "key one");
	expect_missing(c, "Sp ace", "key one ");

	config_close(c);
}

static void test_parse_crlf_line_endings(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[A]\r\nk=v\r\n\r\n[B]\r\nj=w\r\n");

	assert_int_equal(config_num_sections(c), 2);
	expect_section(c, 0, "A");
	expect_section(c, 1, "B");
	expect_str(c, "A", "k", "v");
	expect_str(c, "B", "j", "w");

	config_close(c);
}

static void test_parse_comments(void **state)
{
	UNUSED_PARAMETER(state);

	/* only '#' starts a comment (and only at the start of a line) */
	config_t *c = open_str("# top comment\n; top semicolon\n[S]\n# hash comment\n#x=2\n; semi=1\nreal=3\n"
			       "v=a # not\n");

	assert_int_equal(config_num_sections(c), 1);
	expect_section(c, 0, "S");
	expect_missing(c, "S", "x");
	expect_missing(c, "S", "#x");
	expect_str(c, "S", "real", "3");
	/* characterized, not endorsed: ';' is not a comment, it becomes part of the key name */
	expect_str(c, "S", "; semi", "1");
	expect_missing(c, "S", "semi");
	/* a '#' after the value is part of the value */
	expect_str(c, "S", "v", "a # not");

	config_close(c);
}

static void test_parse_empty_values(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[S]\nempty=\nafter=x\neof=");

	expect_str(c, "S", "empty", "");
	expect_str(c, "S", "after", "x");
	expect_str(c, "S", "eof", "");

	config_close(c);
}

static void test_parse_no_trailing_newline(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[S]\nk=v");

	expect_str(c, "S", "k", "v");
	config_close(c);
}

static void test_parse_keys_before_section(void **state)
{
	UNUSED_PARAMETER(state);

	/* lines before the first section header are dropped */
	config_t *c = open_str("orphan=1\n; semi\nmore stuff\n[S]\nk=v\n");

	assert_int_equal(config_num_sections(c), 1);
	expect_section(c, 0, "S");
	expect_str(c, "S", "k", "v");
	expect_missing(c, "S", "orphan");
	expect_missing(c, "", "orphan");

	config_close(c);

	/* with no section at all nothing is kept */
	c = open_str("a=1\nb=2\n");
	assert_int_equal(config_num_sections(c), 0);
	config_close(c);
}

static void test_parse_lines_without_equals(void **state)
{
	UNUSED_PARAMETER(state);

	/* a line without '=' is dropped, parsing resumes on the next line */
	config_t *c = open_str("[S]\nnoequals\nk=v\njust some words\nk2=v2\n=novalue\n");

	assert_int_equal(config_num_sections(c), 1);
	expect_str(c, "S", "k", "v");
	expect_str(c, "S", "k2", "v2");
	expect_missing(c, "S", "noequals");
	expect_missing(c, "S", "just some words");
	expect_missing(c, "S", "");
	expect_missing(c, "S", "=novalue");

	config_close(c);
}

static void test_parse_last_line_without_equals_at_eof(void **state)
{
	UNUSED_PARAMETER(state);

	/*
	 * Characterized, not endorsed: a line without '=' is dropped only if a
	 * newline follows it. At end of input it becomes a key with an empty value.
	 */
	config_t *c = open_str("[S]\nk=v\nlast words");

	expect_str(c, "S", "k", "v");
	expect_str(c, "S", "last words", "");

	config_close(c);
}

static void test_parse_unterminated_header(void **state)
{
	UNUSED_PARAMETER(state);

	/* characterized, not endorsed: a header without ']' still creates the section */
	config_t *c = open_str("[bad\nk=v\n");

	assert_int_equal(config_num_sections(c), 1);
	expect_section(c, 0, "bad");
	expect_str(c, "bad", "k", "v");
	config_close(c);

	c = open_str("[S]\nk=v\n[unterminated");
	assert_int_equal(config_num_sections(c), 2);
	expect_section(c, 0, "S");
	expect_section(c, 1, "unterminated");
	config_close(c);
}

static void test_parse_empty_header_stops_parsing(void **state)
{
	UNUSED_PARAMETER(state);

	/* characterized, not endorsed: "[]" aborts the whole parse; everything after it is dropped */
	config_t *c = open_str("[S]\nk=v\n[]\n[T]\nj=w\n");

	assert_int_equal(config_num_sections(c), 1);
	expect_section(c, 0, "S");
	expect_str(c, "S", "k", "v");
	expect_missing(c, "T", "j");

	config_close(c);

	c = open_str("[]\n[X]\nk=v\n");
	assert_int_equal(config_num_sections(c), 0);
	config_close(c);
}

static void test_parse_trailing_text_after_header(void **state)
{
	UNUSED_PARAMETER(state);

	/* text after ']' on the header line is parsed like a normal line (no '=' so it is dropped) */
	config_t *c = open_str("[S] junk\nk=v\n");

	assert_int_equal(config_num_sections(c), 1);
	expect_str(c, "S", "k", "v");
	expect_missing(c, "S", "junk");

	config_close(c);
}

static void test_parse_duplicates(void **state)
{
	UNUSED_PARAMETER(state);

	/*
	 * Characterized, not endorsed: duplicates are not merged. Both entries are
	 * kept in order, lookups find the one added last.
	 */
	config_t *c = open_str("[S]\nk=1\nk=2\n[S]\nb=3\n[T]\nk=4\n");

	assert_int_equal(config_num_sections(c), 3);
	expect_section(c, 0, "S");
	expect_section(c, 1, "S");
	expect_section(c, 2, "T");

	/* lookups resolve to the later [S], which has no "k" */
	expect_str(c, "S", "b", "3");
	expect_missing(c, "S", "k");
	expect_str(c, "T", "k", "4");

	config_close(c);

	c = open_str("[S]\nk=1\nk=2\n");
	assert_int_equal(config_num_sections(c), 1);
	expect_str(c, "S", "k", "2");
	config_close(c);
}

static void test_parse_escapes(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[S]\nnl=a\\nb\ncr=a\\rb\nbs=a\\\\b\nunk=a\\tb\ntrail=a\\\n");

	expect_str(c, "S", "nl", "a\nb");
	expect_str(c, "S", "cr", "a\rb");
	expect_str(c, "S", "bs", "a\\b");
	/* unknown escapes are left as written */
	expect_str(c, "S", "unk", "a\\tb");
	/* a lone trailing backslash is kept */
	expect_str(c, "S", "trail", "a\\");

	config_close(c);
}

/* ------------------------------------------------------------------------- */
/* typed getters */

static void test_get_int_parsing(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[N]\nhex=0x1F\nbigx=0X1F\njunk=12abc\nalpha=abc\nneg=-5\nempty=\nsp=  7\nneguint=-1\n"
			       "dec=3.9\nzero=0\n");

	assert_int_equal(config_get_int(c, "N", "hex"), 31);
	/* only a lowercase "0x" prefix selects base 16 */
	assert_int_equal(config_get_int(c, "N", "bigx"), 0);
	assert_int_equal(config_get_int(c, "N", "junk"), 12);
	assert_int_equal(config_get_int(c, "N", "alpha"), 0);
	assert_int_equal(config_get_int(c, "N", "neg"), -5);
	assert_int_equal(config_get_int(c, "N", "empty"), 0);
	assert_int_equal(config_get_int(c, "N", "sp"), 7);
	assert_int_equal(config_get_int(c, "N", "dec"), 3);
	assert_int_equal(config_get_int(c, "N", "zero"), 0);
	/* missing keys read as 0 */
	assert_int_equal(config_get_int(c, "N", "missing"), 0);
	assert_int_equal(config_get_int(c, "Missing", "hex"), 0);

	assert_true(config_get_uint(c, "N", "hex") == 31);
	assert_true(config_get_uint(c, "N", "junk") == 12);
	assert_true(config_get_uint(c, "N", "alpha") == 0);
	assert_true(config_get_uint(c, "N", "missing") == 0);
	/* characterized, not endorsed: a negative value wraps (strtoull semantics) */
	assert_true(config_get_uint(c, "N", "neguint") == UINT64_MAX);

	config_close(c);
}

static void test_get_bool_parsing(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[B]\nt=true\nT=TRUE\none=1\ntwo=2\nzero=0\nf=false\nyes=yes\nhex=0x1\nneg=-1\nempty=\n"
			       "sp= true\n");

	assert_true(config_get_bool(c, "B", "t"));
	/* "true" is compared case-insensitively */
	assert_true(config_get_bool(c, "B", "T"));
	assert_true(config_get_bool(c, "B", "one"));
	/* any non-zero number is true */
	assert_true(config_get_bool(c, "B", "two"));
	assert_true(config_get_bool(c, "B", "hex"));
	assert_true(config_get_bool(c, "B", "neg"));

	assert_false(config_get_bool(c, "B", "zero"));
	assert_false(config_get_bool(c, "B", "f"));
	assert_false(config_get_bool(c, "B", "yes"));
	assert_false(config_get_bool(c, "B", "empty"));
	/* whitespace is not trimmed, so " true" is neither "true" nor a number */
	assert_false(config_get_bool(c, "B", "sp"));
	assert_false(config_get_bool(c, "B", "missing"));

	config_close(c);
}

static void test_get_double_parsing(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[D]\na=1.5\nb=abc\nc=1e3\nd=-0.5\ne= 2.5\nf=\n");

	assert_true(config_get_double(c, "D", "a") == 1.5);
	assert_true(config_get_double(c, "D", "b") == 0.0);
	assert_true(config_get_double(c, "D", "c") == 1000.0);
	assert_true(config_get_double(c, "D", "d") == -0.5);
	assert_true(config_get_double(c, "D", "e") == 2.5);
	assert_true(config_get_double(c, "D", "f") == 0.0);
	assert_true(config_get_double(c, "D", "missing") == 0.0);

	config_close(c);
}

/* ------------------------------------------------------------------------- */
/* setters */

static void test_set_get_types(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("");

	config_set_string(c, "S", "str", "hello");
	expect_str(c, "S", "str", "hello");

	/* a NULL string is stored as "" */
	config_set_string(c, "S", "null", NULL);
	expect_str(c, "S", "null", "");

	config_set_int(c, "S", "int", -42);
	expect_str(c, "S", "int", "-42");
	assert_int_equal(config_get_int(c, "S", "int"), -42);

	config_set_uint(c, "S", "uint", UINT64_MAX);
	expect_str(c, "S", "uint", "18446744073709551615");
	assert_true(config_get_uint(c, "S", "uint") == UINT64_MAX);

	config_set_bool(c, "S", "yes", true);
	config_set_bool(c, "S", "no", false);
	expect_str(c, "S", "yes", "true");
	expect_str(c, "S", "no", "false");
	assert_true(config_get_bool(c, "S", "yes"));
	assert_false(config_get_bool(c, "S", "no"));

	/* doubles are written with 17 significant digits and always contain '.' or 'e' */
	config_set_double(c, "S", "d1", 1.5);
	config_set_double(c, "S", "d2", 2.0);
	config_set_double(c, "S", "d3", 0.1);
	expect_str(c, "S", "d1", "1.5");
	expect_str(c, "S", "d2", "2.0");
	expect_str(c, "S", "d3", "0.10000000000000001");
	assert_true(config_get_double(c, "S", "d1") == 1.5);
	assert_true(config_get_double(c, "S", "d2") == 2.0);
	assert_true(config_get_double(c, "S", "d3") == 0.1);

	/* overwriting replaces the value and keeps one entry */
	config_set_string(c, "S", "str", "again");
	expect_str(c, "S", "str", "again");
	assert_int_equal(config_num_sections(c), 1);

	config_close(c);
}

static void test_set_adds_sections_in_order(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[B]\nk=1\n[A]\nk=2\n");

	config_set_string(c, "Z", "k", "3");
	config_set_string(c, "B", "j", "4");

	assert_int_equal(config_num_sections(c), 3);
	expect_section(c, 0, "B");
	expect_section(c, 1, "A");
	expect_section(c, 2, "Z");
	expect_str(c, "B", "j", "4");

	config_close(c);
}

static void test_remove_value(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("[S]\na=1\nb=2\n");

	assert_true(config_remove_value(c, "S", "a"));
	expect_missing(c, "S", "a");
	expect_str(c, "S", "b", "2");
	assert_false(config_has_user_value(c, "S", "a"));

	/* removing twice, or a name or section that does not exist, fails */
	assert_false(config_remove_value(c, "S", "a"));
	assert_false(config_remove_value(c, "S", "nope"));
	assert_false(config_remove_value(c, "Nope", "b"));

	/* the section stays even when it has no values left */
	assert_true(config_remove_value(c, "S", "b"));
	assert_int_equal(config_num_sections(c), 1);
	expect_section(c, 0, "S");

	config_close(c);
}

/* ------------------------------------------------------------------------- */
/* defaults */

static void test_defaults_become_user_values(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("");

	assert_false(config_has_default_value(c, "D", "i"));
	assert_false(config_has_user_value(c, "D", "i"));

	/*
	 * Characterized, not endorsed: setting a default also creates a user value
	 * (and the section) when none exists.
	 */
	config_set_default_int(c, "D", "i", 7);
	assert_true(config_has_default_value(c, "D", "i"));
	assert_true(config_has_user_value(c, "D", "i"));
	assert_int_equal(config_num_sections(c), 1);
	assert_int_equal(config_get_int(c, "D", "i"), 7);
	assert_int_equal(config_get_default_int(c, "D", "i"), 7);

	/* the user value now wins over the default and does not change it */
	config_set_int(c, "D", "i", 9);
	assert_int_equal(config_get_int(c, "D", "i"), 9);
	assert_int_equal(config_get_default_int(c, "D", "i"), 7);

	/* setting the default again does not overwrite an existing user value */
	config_set_default_int(c, "D", "i", 8);
	assert_int_equal(config_get_int(c, "D", "i"), 9);
	assert_int_equal(config_get_default_int(c, "D", "i"), 8);

	config_close(c);
}

static void test_defaults_user_value_wins(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("");

	config_set_string(c, "D", "s", "user");
	config_set_default_string(c, "D", "s", "dflt");

	expect_str(c, "D", "s", "user");
	assert_string_equal(config_get_default_string(c, "D", "s"), "dflt");
	assert_true(config_has_user_value(c, "D", "s"));
	assert_true(config_has_default_value(c, "D", "s"));

	/* removing the user value exposes the default */
	assert_true(config_remove_value(c, "D", "s"));
	assert_false(config_has_user_value(c, "D", "s"));
	assert_true(config_has_default_value(c, "D", "s"));
	expect_str(c, "D", "s", "dflt");
	assert_false(config_remove_value(c, "D", "s"));

	config_close(c);
}

static void test_defaults_typed(void **state)
{
	UNUSED_PARAMETER(state);

	config_t *c = open_str("");

	config_set_default_uint(c, "T", "u", 5);
	config_set_default_bool(c, "T", "yes", true);
	config_set_default_bool(c, "T", "no", false);
	config_set_default_double(c, "T", "whole", 3.0);
	config_set_default_double(c, "T", "frac", 0.25);
	config_set_default_string(c, "T", "null", NULL);

	assert_true(config_get_default_uint(c, "T", "u") == 5);
	assert_true(config_get_default_bool(c, "T", "yes"));
	assert_false(config_get_default_bool(c, "T", "no"));
	assert_string_equal(config_get_default_string(c, "T", "yes"), "true");
	assert_string_equal(config_get_default_string(c, "T", "no"), "false");
	/* default doubles use "%g", unlike config_set_double */
	assert_string_equal(config_get_default_string(c, "T", "whole"), "3");
	assert_string_equal(config_get_default_string(c, "T", "frac"), "0.25");
	assert_true(config_get_default_double(c, "T", "frac") == 0.25);
	assert_int_equal(config_get_default_int(c, "T", "whole"), 3);
	assert_string_equal(config_get_default_string(c, "T", "null"), "");

	/* absent defaults */
	assert_null(config_get_default_string(c, "T", "missing"));
	assert_null(config_get_default_string(c, "Missing", "u"));
	assert_int_equal(config_get_default_int(c, "T", "missing"), 0);
	assert_true(config_get_default_uint(c, "T", "missing") == 0);
	assert_false(config_get_default_bool(c, "T", "missing"));
	assert_true(config_get_default_double(c, "T", "missing") == 0.0);
	assert_false(config_has_default_value(c, "T", "missing"));
	assert_false(config_has_user_value(c, "Missing", "u"));

	config_close(c);
}

static void test_open_defaults_file(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(DEFAULTS_PATH, "[Def]\na=1\nb=true\n[Other]\nc=x\n");

	config_t *c = open_str("");

	assert_int_equal(config_open_defaults(c, DEFAULTS_PATH), CONFIG_SUCCESS);

	/* defaults are not user values and are not sections of the user config */
	assert_int_equal(config_num_sections(c), 0);
	assert_true(config_has_default_value(c, "Def", "a"));
	assert_false(config_has_user_value(c, "Def", "a"));
	expect_str(c, "Def", "a", "1");
	assert_int_equal(config_get_default_int(c, "Def", "a"), 1);
	assert_true(config_get_default_bool(c, "Def", "b"));
	expect_str(c, "Other", "c", "x");

	config_set_string(c, "Def", "a", "9");
	assert_int_equal(config_get_int(c, "Def", "a"), 9);
	assert_int_equal(config_get_default_int(c, "Def", "a"), 1);
	assert_int_equal(config_num_sections(c), 1);

	config_close(c);
}

static void test_open_defaults_errors(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = open_str("");

	assert_int_equal(config_open_defaults(c, MISSING_PATH), CONFIG_FILENOTFOUND);
	assert_int_equal(config_open_defaults(NULL, MISSING_PATH), CONFIG_ERROR);
	assert_false(os_file_exists(MISSING_PATH));

	config_close(c);
}

/* ------------------------------------------------------------------------- */
/* files */

static void test_open_null_config(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(config_open(NULL, OPEN_PATH, CONFIG_OPEN_ALWAYS), CONFIG_ERROR);
	assert_int_equal(config_open_string(NULL, "[S]\n"), CONFIG_ERROR);
	assert_false(os_file_exists(OPEN_PATH));

	/* closing NULL is a no-op */
	config_close(NULL);
}

static void test_open_existing_missing(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = (config_t *)(uintptr_t)1;

	assert_int_equal(config_open(&c, MISSING_PATH, CONFIG_OPEN_EXISTING), CONFIG_FILENOTFOUND);
	assert_null(c);
	assert_false(os_file_exists(MISSING_PATH));
}

static void test_open_existing_file(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(OPEN_PATH, "[S]\nk=v\n");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, OPEN_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	assert_non_null(c);
	assert_int_equal(config_num_sections(c), 1);
	expect_str(c, "S", "k", "v");
	config_close(c);

	/* OPEN_ALWAYS on an existing file reads it without touching it */
	c = NULL;
	assert_int_equal(config_open(&c, OPEN_PATH, CONFIG_OPEN_ALWAYS), CONFIG_SUCCESS);
	expect_str(c, "S", "k", "v");
	config_close(c);
	expect_file(OPEN_PATH, "[S]\nk=v\n");
}

static void test_open_always_creates_file(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	assert_false(os_file_exists(ALWAYS_PATH));

	config_t *c = NULL;

	assert_int_equal(config_open(&c, ALWAYS_PATH, CONFIG_OPEN_ALWAYS), CONFIG_SUCCESS);
	assert_non_null(c);
	assert_int_equal(config_num_sections(c), 0);
	assert_true(os_file_exists(ALWAYS_PATH));
	expect_file_empty(ALWAYS_PATH);

	config_close(c);
}

static void test_open_empty_file(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(EMPTY_PATH, "");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, EMPTY_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	assert_non_null(c);
	assert_int_equal(config_num_sections(c), 0);
	config_close(c);
}

static void test_create(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(CREATE_PATH, "[Old]\nk=v\n");

	/* config_create truncates an existing file and does not read it */
	config_t *c = config_create(CREATE_PATH);

	assert_non_null(c);
	assert_int_equal(config_num_sections(c), 0);
	expect_file_empty(CREATE_PATH);
	config_close(c);

	/* an unwritable path fails */
	assert_null(config_create(BADDIR_PATH));
	assert_false(os_file_exists(BADDIR_PATH));
}

static void test_save_exact_text(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = config_create(CREATE_PATH);

	assert_non_null(c);

	/* sections and keys are written in insertion order, '\n' line endings, blank line between sections */
	config_set_string(c, "Video", "fps", "60");
	config_set_int(c, "Audio", "rate", 48000);
	config_set_string(c, "Video", "res", "1920x1080");
	config_set_bool(c, "Audio", "mono", false);
	config_set_string(c, "Video", "path", "a\\b\nc\rd");
	config_set_double(c, "Misc", "ratio", 1.5);

	assert_int_equal(config_save(c), CONFIG_SUCCESS);
	expect_file(CREATE_PATH, "[Video]\nfps=60\nres=1920x1080\npath=a\\\\b\\nc\\rd\n\n"
				 "[Audio]\nrate=48000\nmono=false\n\n"
				 "[Misc]\nratio=1.5\n");
	config_close(c);

	/* what was written reads back to the same values */
	assert_int_equal(config_open(&c, CREATE_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	assert_int_equal(config_num_sections(c), 3);
	expect_section(c, 0, "Video");
	expect_section(c, 1, "Audio");
	expect_section(c, 2, "Misc");
	expect_str(c, "Video", "path", "a\\b\nc\rd");
	assert_int_equal(config_get_int(c, "Audio", "rate"), 48000);
	assert_false(config_get_bool(c, "Audio", "mono"));
	assert_true(config_get_double(c, "Misc", "ratio") == 1.5);
	config_close(c);
}

static void test_save_overwrites_file(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(OPEN_PATH, "[S]\nk=old\nextra=gone\n");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, OPEN_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	config_set_string(c, "S", "k", "new");
	assert_true(config_remove_value(c, "S", "extra"));
	assert_int_equal(config_save(c), CONFIG_SUCCESS);
	config_close(c);

	expect_file(OPEN_PATH, "[S]\nk=new\n");
}

static void test_save_empty_config(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = config_create(CREATE_PATH);

	assert_non_null(c);

	/* saving a config with no sections writes an empty file */
	assert_int_equal(config_save(c), CONFIG_SUCCESS);
	expect_file_empty(CREATE_PATH);
	config_close(c);
}

static void test_save_empty_section(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = config_create(CREATE_PATH);

	assert_non_null(c);
	config_set_string(c, "S", "k", "v");
	assert_true(config_remove_value(c, "S", "k"));
	assert_int_equal(config_save(c), CONFIG_SUCCESS);
	expect_file(CREATE_PATH, "[S]\n");
	config_close(c);
}

static void test_save_invalid_config(void **state)
{
	UNUSED_PARAMETER(state);

	assert_int_equal(config_save(NULL), CONFIG_ERROR);

	/* a config opened from a string has no file to save to */
	config_t *c = open_str("[S]\nk=v\n");

	assert_int_equal(config_save(c), CONFIG_ERROR);
	config_close(c);
}

static void test_save_writes_defaults_as_user_values(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = config_create(DEFSAVE_PATH);

	assert_non_null(c);
	config_set_default_int(c, "A", "x", 1);
	config_set_string(c, "A", "y", "2");
	assert_int_equal(config_save(c), CONFIG_SUCCESS);

	/* characterized, not endorsed: defaults were copied into the user values, so they are saved */
	expect_file(DEFSAVE_PATH, "[A]\nx=1\ny=2\n");
	config_close(c);
}

static void test_roundtrip_malformed(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(ROUNDTRIP_PATH, "orphan=1\n# top comment\n[First]\n; semi=2\n#hash=3\nnoequals\nkey = spaced \n"
				   "k=a\nk=b\n\n[Empty]\n[First]\nlate=4\nlast");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, ROUNDTRIP_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	assert_int_equal(config_num_sections(c), 3);
	assert_int_equal(config_save(c), CONFIG_SUCCESS);
	config_close(c);

	/*
	 * Dropped: "orphan=1", the comments, "noequals". Kept as is: the ';' key,
	 * whitespace around '=', duplicate keys and sections, an empty section,
	 * and the final "last" line, which becomes "last=".
	 */
	expect_file(ROUNDTRIP_PATH, "[First]\n; semi=2\nkey = spaced \nk=a\nk=b\n\n"
				    "[Empty]\n\n"
				    "[First]\nlate=4\nlast=\n");
}

static void test_roundtrip_crlf_input(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(ROUNDTRIP_PATH, "[A]\r\nk=v\r\n\r\n[B]\r\nj=w\r\n");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, ROUNDTRIP_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	assert_int_equal(config_save(c), CONFIG_SUCCESS);
	config_close(c);

	/* output always uses '\n' */
	expect_file(ROUNDTRIP_PATH, "[A]\nk=v\n\n[B]\nj=w\n");
}

static void test_roundtrip_unknown_escape(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(ESCAPE_PATH, "[S]\nk=a\\tb\nn=a\\nb\n");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, ESCAPE_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	expect_str(c, "S", "k", "a\\tb");
	assert_int_equal(config_save(c), CONFIG_SUCCESS);
	config_close(c);

	/*
	 * Characterized, not endorsed: the reader leaves "\t" alone but the
	 * writer doubles every backslash, so saving is not idempotent for it.
	 */
	expect_file(ESCAPE_PATH, "[S]\nk=a\\\\tb\nn=a\\nb\n");
}

/* ------------------------------------------------------------------------- */
/* config_save_safe */

static void test_save_safe_with_backup(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = config_create(SAFE1_PATH);

	assert_non_null(c);
	config_set_string(c, "Old", "k", "1");
	assert_int_equal(config_save(c), CONFIG_SUCCESS);

	config_set_string(c, "Old", "k", "2");
	assert_int_equal(config_save_safe(c, "tmp", "bak"), CONFIG_SUCCESS);
	config_close(c);

	expect_file(SAFE1_PATH, "[Old]\nk=2\n");
	expect_file(SAFE1_PATH ".bak", "[Old]\nk=1\n");
	assert_false(os_file_exists(SAFE1_PATH ".tmp"));
}

static void test_save_safe_dotted_extensions(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();

	config_t *c = config_create(SAFE2_PATH);

	assert_non_null(c);
	config_set_string(c, "Old", "k", "1");
	assert_int_equal(config_save(c), CONFIG_SUCCESS);

	config_set_string(c, "Old", "k", "2");
	assert_int_equal(config_save_safe(c, ".tmp", ".bak"), CONFIG_SUCCESS);
	config_close(c);

	/* a leading '.' on an extension is not doubled */
	expect_file(SAFE2_PATH, "[Old]\nk=2\n");
	expect_file(SAFE2_PATH ".bak", "[Old]\nk=1\n");
	assert_false(os_file_exists(SAFE2_PATH ".tmp"));
}

static void test_save_safe_without_backup(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(SAFE3_PATH, "[A]\nx=1\n");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, SAFE3_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	config_set_int(c, "A", "x", 2);
	assert_int_equal(config_save_safe(c, "tmp", NULL), CONFIG_SUCCESS);
	expect_file(SAFE3_PATH, "[A]\nx=2\n");
	assert_false(os_file_exists(SAFE3_PATH ".tmp"));
	assert_false(os_file_exists(SAFE3_PATH ".bak"));

	/* an empty backup extension also means no backup */
	config_set_int(c, "A", "x", 3);
	assert_int_equal(config_save_safe(c, "tmp", ""), CONFIG_SUCCESS);
	config_close(c);

	expect_file(SAFE3_PATH, "[A]\nx=3\n");
	assert_false(os_file_exists(SAFE3_PATH ".tmp"));
	assert_false(os_file_exists(SAFE3_PATH ".bak"));
}

static void test_save_safe_invalid_temp_extension(void **state)
{
	UNUSED_PARAMETER(state);

	remove_test_files();
	write_file(SAFE4_PATH, "[A]\nx=1\n");

	config_t *c = NULL;

	assert_int_equal(config_open(&c, SAFE4_PATH, CONFIG_OPEN_EXISTING), CONFIG_SUCCESS);
	config_set_int(c, "A", "x", 2);

	assert_int_equal(config_save_safe(c, NULL, "bak"), CONFIG_ERROR);
	assert_int_equal(config_save_safe(c, "", "bak"), CONFIG_ERROR);
	config_close(c);

	/* nothing was written */
	expect_file(SAFE4_PATH, "[A]\nx=1\n");
	assert_false(os_file_exists(SAFE4_PATH ".tmp"));
	assert_false(os_file_exists(SAFE4_PATH ".bak"));
}

int main(void)
{
	const struct CMUnitTest tests[] = {
		cmocka_unit_test(test_parse_basic),
		cmocka_unit_test(test_parse_empty_string),
		cmocka_unit_test(test_parse_whitespace_is_kept),
		cmocka_unit_test(test_parse_crlf_line_endings),
		cmocka_unit_test(test_parse_comments),
		cmocka_unit_test(test_parse_empty_values),
		cmocka_unit_test(test_parse_no_trailing_newline),
		cmocka_unit_test(test_parse_keys_before_section),
		cmocka_unit_test(test_parse_lines_without_equals),
		cmocka_unit_test(test_parse_last_line_without_equals_at_eof),
		cmocka_unit_test(test_parse_unterminated_header),
		cmocka_unit_test(test_parse_empty_header_stops_parsing),
		cmocka_unit_test(test_parse_trailing_text_after_header),
		cmocka_unit_test(test_parse_duplicates),
		cmocka_unit_test(test_parse_escapes),
		cmocka_unit_test(test_get_int_parsing),
		cmocka_unit_test(test_get_bool_parsing),
		cmocka_unit_test(test_get_double_parsing),
		cmocka_unit_test(test_set_get_types),
		cmocka_unit_test(test_set_adds_sections_in_order),
		cmocka_unit_test(test_remove_value),
		cmocka_unit_test(test_defaults_become_user_values),
		cmocka_unit_test(test_defaults_user_value_wins),
		cmocka_unit_test(test_defaults_typed),
		cmocka_unit_test(test_open_defaults_file),
		cmocka_unit_test(test_open_defaults_errors),
		cmocka_unit_test(test_open_null_config),
		cmocka_unit_test(test_open_existing_missing),
		cmocka_unit_test(test_open_existing_file),
		cmocka_unit_test(test_open_always_creates_file),
		cmocka_unit_test(test_open_empty_file),
		cmocka_unit_test(test_create),
		cmocka_unit_test(test_save_exact_text),
		cmocka_unit_test(test_save_overwrites_file),
		cmocka_unit_test(test_save_empty_config),
		cmocka_unit_test(test_save_empty_section),
		cmocka_unit_test(test_save_invalid_config),
		cmocka_unit_test(test_save_writes_defaults_as_user_values),
		cmocka_unit_test(test_roundtrip_malformed),
		cmocka_unit_test(test_roundtrip_crlf_input),
		cmocka_unit_test(test_roundtrip_unknown_escape),
		cmocka_unit_test(test_save_safe_with_backup),
		cmocka_unit_test(test_save_safe_dotted_extensions),
		cmocka_unit_test(test_save_safe_without_backup),
		cmocka_unit_test(test_save_safe_invalid_temp_extension),
	};

	return cmocka_run_group_tests(tests, group_setup, group_teardown);
}
