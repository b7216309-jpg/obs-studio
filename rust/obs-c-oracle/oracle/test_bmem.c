/* Test-only stand-in for libobs util/bmem.c, which needs base/platform.
 * Linked into every obs-util test binary so the oracle's darray and
 * array-serializer code can resolve bmalloc/brealloc/bfree. Delete when
 * util/bmem.c is ported.
 *
 * Deliberately includes no libobs header: bmem.h marks these functions
 * EXPORT/dllexport. */
#include <stdlib.h>
#include <string.h>

void *bmalloc(size_t size)
{
	void *ptr;
	if (!size)
		abort();
	ptr = malloc(size);
	if (!ptr)
		abort();
	return ptr;
}

void *brealloc(void *ptr, size_t size)
{
	if (!size)
		abort();
	ptr = realloc(ptr, size);
	if (!ptr)
		abort();
	return ptr;
}

void bfree(void *ptr)
{
	free(ptr);
}

/* bmem.c's bmemdup, for the bstrdup in the lexer oracle. */
void *bmemdup(const void *ptr, size_t size)
{
	void *out = bmalloc(size);
	if (size)
		memcpy(out, ptr, size);

	return out;
}
