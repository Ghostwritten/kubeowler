#!/usr/bin/env bash
# Multi-arch build: amd64 and arm64. Output: target/<target>/release/kubeowler
# Usage: ./build-multi-arch.sh [--release]
# Requires: rustup target add x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu

set -e

RELEASE=""
[ "$1" = "--release" ] && RELEASE="--release"

for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
  echo "Building for $target..."
  cargo build $RELEASE --target "$target"
done

echo "Done. Binaries under target/<target>/release or target/<target>/debug"
