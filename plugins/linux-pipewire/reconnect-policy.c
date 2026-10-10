/* reconnect-policy.c
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "reconnect-policy.h"

#include <util/c99defs.h>

bool obs_pipewire_core_error_is_lost(bool *disconnected, uint32_t id, int res)
{
	/* Not implemented yet: core errors are only logged. */
	UNUSED_PARAMETER(disconnected);
	UNUSED_PARAMETER(id);
	UNUSED_PARAMETER(res);
	return false;
}

bool obs_pipewire_stream_state_is_lost(bool *disconnected, bool *streamed, enum pw_stream_state state)
{
	/* Not implemented yet: stream errors are only logged. */
	UNUSED_PARAMETER(disconnected);
	UNUSED_PARAMETER(streamed);
	UNUSED_PARAMETER(state);
	return false;
}

bool obs_pipewire_reconnect_next_delay(uint32_t *attempts, uint32_t *delay_ms)
{
	/* Not implemented yet: the screencast is never reconnected. */
	UNUSED_PARAMETER(attempts);
	UNUSED_PARAMETER(delay_ms);
	return false;
}
