#!/bin/bash
# Usage: rust/tools/linux-validate/run.sh
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
# In a git worktree, .git is a file whose (and whose submodules') git dirs
# resolve only on the host, so `git describe` fails inside the container.
# Resolve the OBS version here instead and hand it to validate.sh.
env=()
if [ -f "$repo/.git" ]; then
  env=(-e "OBS_VERSION_OVERRIDE=$(git -C "$repo" describe --always --tags)")
fi
# OBS_CCACHE_DIR: host directory for the compiler cache (CI restores/saves it
# with actions/cache). Unset, the cache lives in the build volume.
if [ -n "${OBS_CCACHE_DIR:-}" ]; then
  mkdir -p "$OBS_CCACHE_DIR"
  env+=(-v "$OBS_CCACHE_DIR:/ccache" -e CCACHE_DIR=/ccache)
fi
docker build -q -t obs-rust-linux-validate "$here" >/dev/null
docker run --rm \
  -v "$repo:/ro:ro" \
  "${env[@]}" \
  -v "$here/validate.sh:/validate.sh:ro" \
  -v obs-rust-linux-validate-build:/build \
  obs-rust-linux-validate /validate.sh
