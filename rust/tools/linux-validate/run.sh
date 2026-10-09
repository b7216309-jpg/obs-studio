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
docker build -t obs-rust-linux-validate "$here"
docker run --rm \
  -v "$repo:/ro:ro" \
  "${env[@]}" \
  -v "$here/validate.sh:/validate.sh:ro" \
  -v obs-rust-linux-validate-build:/build \
  obs-rust-linux-validate /validate.sh
