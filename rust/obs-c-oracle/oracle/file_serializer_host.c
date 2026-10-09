/* Symbols file-serializer.c needs from platform.c / bmem.c, without pulling
 * obs.h in. On Unix these are the same calls platform.c and platform-nix.c
 * make. On Windows they follow platform.c (wide fopen) and platform-windows.c
 * (DeleteFileW / MoveFileExW). */
#define _FILE_OFFSET_BITS 64

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wchar.h>

void *bmalloc(size_t size);

void *bmemdup(const void *ptr, size_t size)
{
	void *out = bmalloc(size);
	if (size)
		memcpy(out, ptr, size);
	return out;
}

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>

static wchar_t *utf8_to_wide(const char *path)
{
	int n;
	wchar_t *wide;

	if (!path)
		return NULL;
	n = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, NULL, 0);
	if (n <= 0)
		return NULL;
	wide = (wchar_t *)malloc((size_t)n * sizeof(wchar_t));
	if (!wide)
		return NULL;
	if (MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, path, -1, wide, n) != n) {
		free(wide);
		return NULL;
	}
	return wide;
}

FILE *os_fopen(const char *path, const char *mode)
{
	wchar_t *wpath;
	wchar_t *wmode;
	FILE *file;

	if (!path)
		return NULL;
	wpath = utf8_to_wide(path);
	wmode = utf8_to_wide(mode);
	file = (wpath && wmode) ? _wfopen(wpath, wmode) : NULL;
	free(wpath);
	free(wmode);
	return file;
}

int os_fseeki64(FILE *file, int64_t offset, int origin)
{
	return _fseeki64(file, offset, origin);
}

int64_t os_ftelli64(FILE *file)
{
	return _ftelli64(file);
}

int os_unlink(const char *path)
{
	wchar_t *wide = utf8_to_wide(path);
	int code;

	if (!wide)
		return -1;
	code = DeleteFileW(wide) ? 0 : -1;
	free(wide);
	return code;
}

int os_rename(const char *old_path, const char *new_path)
{
	wchar_t *old_wide = utf8_to_wide(old_path);
	wchar_t *new_wide = utf8_to_wide(new_path);
	int code = -1;

	if (old_wide && new_wide)
		code = MoveFileExW(old_wide, new_wide, MOVEFILE_REPLACE_EXISTING) ? 0 : -1;
	free(old_wide);
	free(new_wide);
	return code;
}
#else
#include <unistd.h>

FILE *os_fopen(const char *path, const char *mode)
{
	return path ? fopen(path, mode) : NULL;
}

int os_fseeki64(FILE *file, int64_t offset, int origin)
{
	return fseeko(file, offset, origin);
}

int64_t os_ftelli64(FILE *file)
{
	return ftello(file);
}

int os_unlink(const char *path)
{
	return unlink(path);
}

int os_rename(const char *old_path, const char *new_path)
{
	return rename(old_path, new_path);
}
#endif

/* dstr.c (compiled into this oracle for dstr_copy / dstr_ncat) references
 * these. file-serializer.c does not call them. */
size_t os_mbs_to_utf8_ptr(const char *str, size_t len, char **pstr)
{
	(void)str;
	(void)len;
	if (pstr)
		*pstr = NULL;
	return 0;
}

size_t os_utf8_to_wcs_ptr(const char *str, size_t len, wchar_t **pstr)
{
	(void)str;
	(void)len;
	if (pstr)
		*pstr = NULL;
	return 0;
}

size_t wchar_to_utf8(const wchar_t *in, size_t insize, char *out, size_t outsize, int flags)
{
	(void)in;
	(void)insize;
	(void)out;
	(void)outsize;
	(void)flags;
	return 0;
}
