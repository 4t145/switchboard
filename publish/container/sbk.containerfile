# Build
FROM docker.io/library/rust:1-bookworm AS builder
WORKDIR /usr/src/switchboard

# especially for China users, use rsproxy registry mirror
ARG USE_RSPROXY=false

# Copy source code, build skip target folder
COPY . .

RUN if [ "$USE_RSPROXY" = "true" ]; then \
        mkdir -p /usr/local/cargo; \
        echo '[source.crates-io]' > /usr/local/cargo/config.toml; \
        echo 'replace-with = "rsproxy-sparse"' >> /usr/local/cargo/config.toml; \
        echo '[source.rsproxy-sparse]' >> /usr/local/cargo/config.toml; \
        echo 'registry = "sparse+https://rsproxy.cn/index/"' >> /usr/local/cargo/config.toml; \
        echo "Configured rsproxy mirror."; \
    else \
        echo "Using default crates.io registry."; \
    fi

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/switchboard/target \
    cargo build --release --bin sbk \
    && cp target/release/sbk /usr/local/bin/sbk

# Switchboard Kernel Runtime Image
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create application user
RUN useradd -r -u 1000 -s /bin/false switchboard

# Create necessary directories
RUN mkdir -p /etc/switchboard /var/lib/switchboard /var/run/switchboard && \
    chown -R switchboard:switchboard /var/lib/switchboard /var/run/switchboard

# Copy locally built binary
COPY --from=builder --chown=switchboard:switchboard /usr/local/bin/sbk /usr/local/bin/sbk
COPY --chown=switchboard:switchboard publish/container/kernel.toml /etc/switchboard/kernel.toml
# Copy default kernel config file
# COPY --chown=switchboard:switchboard examples/config/ /etc/switchboard/

# 设置工作目录
WORKDIR /var/lib/switchboard

# 切换到非 root 用户
USER switchboard

# 暴露端口（根据实际需要调整）
# EXPOSE 8080
# EXPOSE 8443

# 健康检查（可选）
# HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
#   CMD ["/usr/local/bin/sbk", "health"]
# 启动命令
ENTRYPOINT ["/usr/local/bin/sbk"]
CMD ["/etc/switchboard/kernel.toml"]