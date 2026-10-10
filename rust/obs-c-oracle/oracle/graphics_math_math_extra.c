/* Test-only oracle: the original libobs/graphics/math-extra.c, unmodified, with
 * its global symbols renamed to oracle_* (graphics_math_names.h). */
#include "graphics_math_names.h"

#include "graphics/math-extra.c"

/* Host for the Rust ports: vec3_rand calls libobs rand_float, which stays C
 * in math-extra.c. Test binaries have no libobs, so the oracle provides the
 * unrenamed symbol, backed by the oracle copy. Remove it when math-extra.c
 * is ported. */
#undef rand_float
float rand_float(int positive_only)
{
	return oracle_rand_float(positive_only);
}
