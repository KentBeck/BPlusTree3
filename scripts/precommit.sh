#!/usr/bin/env bash
set -euo pipefail

echo "[pre-commit] Formatting (cargo fmt --all)"
cargo fmt --all

echo "[pre-commit] Clippy (lib only, deny warnings)"
cargo clippy -p bplustree --lib -- -D warnings

echo "[pre-commit] Running tests (workspace)"
cargo test --workspace

echo "[pre-commit] OK"

