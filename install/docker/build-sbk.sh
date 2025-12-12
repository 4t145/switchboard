#!/bin/bash
# 构建 Switchboard Kernel Docker 镜像

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 配置
IMAGE_NAME="${IMAGE_NAME:-switchboard/sbk}"
VERSION="${VERSION:-$(git describe --tags --always --dirty 2>/dev/null || echo 'dev')}"
REGISTRY="${REGISTRY:-}"

echo -e "${GREEN}Building Switchboard Kernel (sbk)${NC}"
echo "Version: $VERSION"
echo "Image: $IMAGE_NAME:$VERSION"

# 1. 编译 Release 版本
echo -e "\n${YELLOW}Step 1: Building release binary...${NC}"
cd ../..
cargo build --release --bin sbk

# 检查二进制文件
if [ ! -f "target/release/sbk" ]; then
    echo -e "${RED}Error: Binary not found at target/release/sbk${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Binary built successfully${NC}"
ls -lh target/release/sbk

# 2. 构建 Docker 镜像
echo -e "\n${YELLOW}Step 2: Building Docker image...${NC}"
cd install/docker
docker build \
    -f Dockerfile.sbk \
    -t "$IMAGE_NAME:$VERSION" \
    -t "$IMAGE_NAME:latest" \
    ../..

echo -e "${GREEN}✓ Docker image built successfully${NC}"

# 3. 显示镜像信息
echo -e "\n${YELLOW}Image info:${NC}"
docker images "$IMAGE_NAME" --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"

# 4. 询问是否推送
if [ -n "$REGISTRY" ]; then
    echo -e "\n${YELLOW}Push to registry? (y/N)${NC}"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}Pushing to $REGISTRY...${NC}"
        docker tag "$IMAGE_NAME:$VERSION" "$REGISTRY/$IMAGE_NAME:$VERSION"
        docker tag "$IMAGE_NAME:latest" "$REGISTRY/$IMAGE_NAME:latest"
        docker push "$REGISTRY/$IMAGE_NAME:$VERSION"
        docker push "$REGISTRY/$IMAGE_NAME:latest"
        echo -e "${GREEN}✓ Pushed successfully${NC}"
    fi
fi

echo -e "\n${GREEN}Done! You can now run:${NC}"
echo "  docker run --rm $IMAGE_NAME:$VERSION"
echo ""
echo -e "${GREEN}Or with docker-compose:${NC}"
echo "  docker compose up -d"
