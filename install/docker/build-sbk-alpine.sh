#!/bin/bash
# 构建 sbk Alpine Docker 镜像

set -e

# 切换到项目根目录
cd "$(dirname "$0")/../.."

echo "==> 编译 sbk 二进制..."
cargo build --release --bin sbk

echo "==> 检查二进制文件..."
if [ ! -f "target/release/sbk" ]; then
    echo "错误: target/release/sbk 不存在"
    exit 1
fi

# 获取版本信息
VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[] | select(.name == "switchboard-kernel") | .version' 2>/dev/null || echo "latest")
GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")

echo "==> 构建 Docker 镜像 (Alpine)..."
docker build \
    -f install/docker/Dockerfile.sbk-alpine \
    -t switchboard/sbk:${VERSION}-alpine \
    -t switchboard/sbk:latest-alpine \
    -t switchboard/sbk:${VERSION}-${GIT_HASH}-alpine \
    .

echo "==> 镜像构建完成!"
echo "    switchboard/sbk:${VERSION}-alpine"
echo "    switchboard/sbk:latest-alpine"
echo "    switchboard/sbk:${VERSION}-${GIT_HASH}-alpine"

# 显示镜像大小
docker images switchboard/sbk:latest-alpine

# 可选: 推送到 registry
if [ "$1" = "--push" ]; then
    echo "==> 推送到 registry..."
    docker push switchboard/sbk:${VERSION}-alpine
    docker push switchboard/sbk:latest-alpine
    docker push switchboard/sbk:${VERSION}-${GIT_HASH}-alpine
    echo "==> 推送完成!"
fi
