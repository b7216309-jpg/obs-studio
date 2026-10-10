#include "product-name.h"

int obs_product_string(char *out, size_t size, const char *obs_version, const char *rust_version)
{
	(void)obs_version;
	(void)rust_version;
	if (size)
		out[0] = 0;
	return 0;
}
