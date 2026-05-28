#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$REPO_ROOT"

echo ">> Formatting..."
cargo fmt --all

echo ">> Linting..."
cargo clippy --workspace --lib --tests -- -D warnings

echo ">> Testing..."
cargo test --lib --workspace
