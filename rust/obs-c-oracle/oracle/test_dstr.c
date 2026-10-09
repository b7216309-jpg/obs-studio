/* Test-only stand-in for dstr_catf from libobs util/dstr.c, which needs
 * util/utf8.c and the rest of dstr. Only the lexer oracle uses it. It is
 * dstr_vcatf with dstr_ensure_capacity reduced to brealloc: the capacity is
 * not observable through error_data_buildstring. Delete when util/dstr.c is
 * ported.
 *
 * Deliberately includes no libobs header: dstr.h marks it EXPORT. */
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void *brealloc(void *ptr, size_t size);
void bfree(void *ptr);

struct dstr {
	char *array;
	size_t len;
	size_t capacity;
};

void dstr_catf(struct dstr *dst, const char *format, ...)
{
	va_list args, args_cp;
	int len;

	va_start(args, format);
	va_copy(args_cp, args);
	len = vsnprintf(NULL, 0, format, args_cp);
	va_end(args_cp);

	if (len < 0)
		len = 4095;

	if (dst->len + (size_t)len + 1 > dst->capacity) {
		dst->capacity = dst->len + (size_t)len + 1;
		dst->array = brealloc(dst->array, dst->capacity);
	}
	len = vsnprintf(dst->array + dst->len, ((size_t)len) + 1, format, args);
	va_end(args);

	if (!*dst->array) {
		bfree(dst->array);
		dst->array = NULL;
		dst->len = 0;
		dst->capacity = 0;
		return;
	}

	dst->len += len < 0 ? strlen(dst->array + dst->len) : (size_t)len;
}
