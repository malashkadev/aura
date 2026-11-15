#!/usr/bin/env bash
# Aura 2.0 - standard startup and verification path.
# Run from the repository root: ./init.sh
set -euo pipefail
cd "$(dirname "$0")"

echo "==> Aura 2.0 verification"
echo "==> Node: $(node --version)"
echo "==> Rust: $(rustc --version)"

echo "==> Frontend dependencies (npm ci)"
npm ci

echo "==> Frontend lint and typecheck"
npm run check

echo "==> Frontend tests"
npm test

echo "==> Static audit gates"
npm run audit

echo "==> Release manifest consistency"
npm run verify:manifest

echo "==> Rust format"
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

echo "==> Rust clippy"
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings

echo "==> Rust tests"
cargo test --manifest-path src-tauri/Cargo.toml --all-targets

echo ""
echo "All Aura checks passed."
