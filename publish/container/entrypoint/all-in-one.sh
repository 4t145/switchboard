#!/usr/bin/env bash

set -euo pipefail

SBK_CONFIG_PATH="${SBK_CONFIG_PATH:-/etc/switchboard/kernel.toml}"
SBC_CONFIG_PATH="${SBC_CONFIG_PATH:-/etc/switchboard/controller.toml}"

cleanup() {
    if [[ -n "${SBC_PID:-}" ]] && kill -0 "${SBC_PID}" 2>/dev/null; then
        kill -TERM "${SBC_PID}" 2>/dev/null || true
    fi
    if [[ -n "${SBK_PID:-}" ]] && kill -0 "${SBK_PID}" 2>/dev/null; then
        kill -TERM "${SBK_PID}" 2>/dev/null || true
    fi
}

trap cleanup TERM INT

/usr/local/bin/sbk "${SBK_CONFIG_PATH}" &
SBK_PID=$!

/usr/local/bin/sbc start --config "${SBC_CONFIG_PATH}" &
SBC_PID=$!

wait -n "${SBK_PID}" "${SBC_PID}"
EXIT_CODE=$?

cleanup
wait || true

exit "${EXIT_CODE}"
