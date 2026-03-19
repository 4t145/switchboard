#!/usr/bin/env bash

set -euo pipefail

SBC_CONFIG_PATH="${SBC_CONFIG_PATH:-/etc/switchboard/controller.toml}"
SBC_K8S="${SBC_K8S:-false}"

ARGS=(start --config "${SBC_CONFIG_PATH}")

if [[ "${SBC_K8S}" == "true" ]]; then
    ARGS+=(--k8s)
fi

exec /usr/local/bin/sbc "${ARGS[@]}" "$@"
