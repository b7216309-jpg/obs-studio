#pragma once

#include <stdint.h>
#include <graphics/vec3.h>

enum clut_dimension {
	CLUT_1D,
	CLUT_3D,
};

/* Loads a .cube LUT as RGBA half floats in a bmalloc buffer, or returns NULL.
 * On success sets *width and *dim; DOMAIN_MIN/DOMAIN_MAX lines overwrite
 * *domain_min and *domain_max. */
void *load_cube_file(const char *const path, uint32_t *const width, struct vec3 *domain_min, struct vec3 *domain_max,
		     enum clut_dimension *dim);
