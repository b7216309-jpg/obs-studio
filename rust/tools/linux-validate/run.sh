#!/bin/bash
# Usage: rust/tools/linux-validate/run.sh [--repeat N] [--tests REGEX]
#   --repeat N     after the normal ctest run, rerun the tests (or those
#                  matching --tests) up to N times in both builds, stopping
#                  at the first failure (ctest --repeat until-fail:N)
#   --tests REGEX  limit the repeat run to tests matching REGEX (ctest -R)
# e.g. run.sh --repeat 100 --tests 'test_threading|test_task'
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
# In a git worktree, .git is a file whose (and whose submodules') git dirs
# resolve only on the host, so `git describe` fails inside the container.
# Resolve the OBS version here instead and hand it to validate.sh.
env=()
# The same goes for submodules: the container cannot initialize them from a
# worktree (obs-rust/obs-studio#32), so initialize any empty one here.
for sub in plugins/obs-browser plugins/obs-websocket; do
  if [ -z "$(ls -A "$repo/$sub" 2>/dev/null)" ]; then
    git -C "$repo" submodule update --init --depth 1 "$sub"
  fi
done
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
  obs-rust-linux-validate /validate.sh "$@"
