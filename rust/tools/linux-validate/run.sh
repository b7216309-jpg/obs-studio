#!/bin/bash
# Usage: rust/tools/linux-validate/run.sh
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
docker build -t obs-rust-linux-validate "$here"
docker run --rm \
  -v "$repo:/ro:ro" \
  -v "$here/validate.sh:/validate.sh:ro" \
  -v obs-rust-linux-validate-build:/build \
  obs-rust-linux-validate /validate.sh
