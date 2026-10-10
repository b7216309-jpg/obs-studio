/* Test-only stand-in for the obsconfig.h that CMake generates from
 * libobs/obsconfig.h.in. obs-config.h includes it unconditionally, so oracles
 * that include obs.h need one. The values only feed macros the oracles do not
 * use. */
#pragma once

#define OBS_DATA_PATH "data"
#define OBS_PLUGIN_PATH "obs-plugins"
#define OBS_PLUGIN_DESTINATION "obs-plugins"
#define OBS_RELEASE_CANDIDATE 0
#define OBS_BETA 0
