#!/usr/bin/env bash
set -euo pipefail

cargo check --lib
cargo run --bin wgsl_check
