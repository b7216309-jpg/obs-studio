/* Display product name for the window title, About dialog, --version and
 * log/crash uploads. Plain C so a cmocka test can build it without Qt. */
#pragma once

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Writes "OBS <obs_version>", or for the Rust build (rust_version not NULL)
 * "OBS-Studio-Rust <rust_version> (based on OBS <obs_version>)", into out.
 * Returns the full length like snprintf; out is always NUL-terminated when
 * size > 0. */
int obs_product_string(char *out, size_t size, const char *obs_version, const char *rust_version);

#ifdef __cplusplus
}
#endif
