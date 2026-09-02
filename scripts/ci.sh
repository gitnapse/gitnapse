#!/usr/bin/env bash
set -euo pipefail

echo "=== cargo fmt --all -- --check ==="
cargo fmt --all -- --check

echo ""
echo "=== cargo clippy --all-targets --all-features -- -D warnings ==="
cargo clippy --all-targets --all-features -- -D warnings

echo ""
echo "=== cargo test --all-targets --all-features ==="
cargo test --all-targets --all-features

echo ""
echo "=== cargo audit ==="
cargo audit --ignore RUSTSEC-2023-0071

echo ""
echo "All CI checks passed."
