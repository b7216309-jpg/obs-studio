/* Test-only stand-in for os_fopen and os_fread_utf8 from libobs
 * util/platform.c, which needs the rest of platform and its OS backends.
 * Only the text-lookup oracle uses them (as oracle_os_*). os_fread_utf8 is
 * copied from platform.c, with os_ftelli64 as ftell and astrcmp_n from the
 * dstr oracle; os_fopen is its non-Windows branch, which matches the
 * Windows one for the ASCII paths the tests use. Delete when util/platform.c
 * is ported or linked into the oracle.
 *
 * Deliberately includes no libobs header: platform.h marks these EXPORT. */
#include <stdio.h>

void *bmalloc(size_t size);
void bfree(void *ptr);
int oracle_astrcmp_n(const char *str1, const char *str2, size_t n);

FILE *oracle_os_fopen(const char *path, const char *mode)
{
	return path ? fopen(path, mode) : NULL;
}

size_t oracle_os_fread_utf8(FILE *file, char **pstr)
{
	size_t size = 0;
	size_t len = 0;

	*pstr = NULL;

	fseek(file, 0, SEEK_END);
	size = (size_t)ftell(file);

	if (size > 0) {
		char bom[3];
		char *utf8str;
		long offset;

		bom[0] = 0;
		bom[1] = 0;
		bom[2] = 0;

		/* remove the ghastly BOM if present */
		fseek(file, 0, SEEK_SET);
		size_t size_read = fread(bom, 1, 3, file);
		(void)size_read;

		offset = (oracle_astrcmp_n(bom, "\xEF\xBB\xBF", 3) == 0) ? 3 : 0;

		size -= offset;
		if (size == 0)
			return 0;

		utf8str = bmalloc(size + 1);
		fseek(file, offset, SEEK_SET);

		size = fread(utf8str, 1, size, file);
		if (size == 0) {
			bfree(utf8str);
			return 0;
		}

		utf8str[size] = 0;

		*pstr = utf8str;
	}

	return len;
}
