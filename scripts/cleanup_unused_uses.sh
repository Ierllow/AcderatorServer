#!/usr/bin/env bash
set -euo pipefail

cargo fix --allow-dirty --allow-staged --all-targets
cargo fix --allow-dirty --allow-staged --all-targets --features debug-ui
cargo fmt
