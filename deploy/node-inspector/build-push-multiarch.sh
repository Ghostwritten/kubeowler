#!/usr/bin/env bash
# Multi-arch build and push for kubeowler-node-inspector image (manifest list)
# Usage: run from repo root: ./deploy/node-inspector/build-push-multiarch.sh [tag]
# Example: ./deploy/node-inspector/build-push-multiarch.sh v0.1.2
# Default tag: v0.1.2
# Requires: Docker buildx (docker buildx create --use)

set -e

TAG="${1:-v0.1.2}"
IMAGE="${IMAGE:-docker.io/ghostwritten/kubeowler-node-inspector}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"

if [ ! -d "$REPO_ROOT/scripts" ] || [ ! -f "$REPO_ROOT/deploy/node-inspector/Dockerfile" ]; then
  echo "Error: run this script from repo root or ensure deploy/node-inspector and scripts exist." >&2
  exit 1
fi

cd "$REPO_ROOT"
docker buildx build --platform linux/amd64,linux/arm64 \
  -f deploy/node-inspector/Dockerfile \
  -t "${IMAGE}:${TAG}" \
  --push .

echo "Pushed ${IMAGE}:${TAG} (linux/amd64, linux/arm64)"
