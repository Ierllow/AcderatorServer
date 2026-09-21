#!/usr/bin/env sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPOSITORY_ROOT=$(dirname -- "$SCRIPT_DIR")
RUNNER="$REPOSITORY_ROOT/target/project-scripts"

mkdir -p "$REPOSITORY_ROOT/target"
rustc --edition 2021 "$SCRIPT_DIR/dev.rs" -o "$RUNNER"
cd "$REPOSITORY_ROOT"
exec "$RUNNER" cleanup
