/*
This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include <string.h>

#include <util/bmem.h>
#include <util/c99defs.h>

#include "pulse-reconnect.h"

bool pulse_reconnect_on_failed(struct pulse_reconnect_state *state, uint64_t *delay_usec)
{
	/* Not implemented yet: a failed context stays failed. */
	UNUSED_PARAMETER(state);
	UNUSED_PARAMETER(delay_usec);
	return false;
}

bool pulse_reconnect_on_ready(struct pulse_reconnect_state *state)
{
	UNUSED_PARAMETER(state);
	return false;
}

char *pulse_default_device(const pa_server_info *info, bool input)
{
	char *device;

	/* As pulse_server_info() did: assumes the server has a default device. */
	if (input)
		return bstrdup(info->default_source_name);

	device = bzalloc(strlen(info->default_sink_name) + 9);
	strcat(device, info->default_sink_name);
	strcat(device, ".monitor");
	return device;
}

bool pulse_restart_should_retry(uint32_t *attempts)
{
	/* Not implemented yet: streams are never restarted. */
	UNUSED_PARAMETER(attempts);
	return false;
}
