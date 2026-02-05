#!/usr/bin/env bash
# 多架构构建并推送 kubeowler-node-inspector 镜像（manifest list）
# 使用：在项目根目录执行 ./deploy/node-inspector/build-push-multiarch.sh [tag]
# 示例：./deploy/node-inspector/build-push-multiarch.sh v0.1.0
# 默认 tag：v0.1.0（与首版发布一致）
# 依赖：Docker buildx（docker buildx create --use）

set -e

TAG="${1:-v0.1.0}"
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
