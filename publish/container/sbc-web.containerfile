# syntax=docker/dockerfile:1.7

FROM docker.io/library/rust:1-bookworm AS rust-builder
WORKDIR /usr/src/switchboard

ARG USE_RSPROXY=false

RUN apt-get update && \
    apt-get install -y --no-install-recommends protobuf-compiler clang libclang-dev && \
    rm -rf /var/lib/apt/lists/*

ENV LIBCLANG_PATH=/usr/lib/llvm-14/lib

COPY . .

RUN if [ "$USE_RSPROXY" = "true" ]; then \
        mkdir -p /usr/local/cargo; \
        echo '[source.crates-io]' > /usr/local/cargo/config.toml; \
        echo 'replace-with = "rsproxy-sparse"' >> /usr/local/cargo/config.toml; \
        echo '[source.rsproxy-sparse]' >> /usr/local/cargo/config.toml; \
        echo 'registry = "sparse+https://rsproxy.cn/index/"' >> /usr/local/cargo/config.toml; \
    fi

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/usr/src/switchboard/target \
    cargo build --release --bin sbc && \
    cp target/release/sbc /usr/local/bin/sbc

FROM docker.io/oven/bun:1.2.23 AS web-builder
WORKDIR /usr/src/switchboard/ui/controller-web

COPY ui/controller-web/package.json ui/controller-web/bun.lock ./
RUN bun install --frozen-lockfile

COPY ui/controller-web .
RUN bun run build

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

RUN useradd -r -u 1000 -s /usr/sbin/nologin switchboard

RUN mkdir -p /etc/switchboard /var/lib/switchboard /opt/switchboard/controller-web && \
    chown -R switchboard:switchboard /etc/switchboard /var/lib/switchboard /opt/switchboard

COPY --from=rust-builder --chown=switchboard:switchboard /usr/local/bin/sbc /usr/local/bin/sbc
COPY --from=web-builder --chown=switchboard:switchboard /usr/src/switchboard/ui/controller-web/build /opt/switchboard/controller-web
COPY --chown=switchboard:switchboard publish/container/config/controller.toml /etc/switchboard/controller.toml
COPY --chown=switchboard:switchboard publish/container/entrypoint/sbc.sh /usr/local/bin/sbc-entrypoint.sh

RUN chmod +x /usr/local/bin/sbc-entrypoint.sh

WORKDIR /var/lib/switchboard
EXPOSE 8056

USER switchboard

ENTRYPOINT ["/usr/local/bin/sbc-entrypoint.sh"]
