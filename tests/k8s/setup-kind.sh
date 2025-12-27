#!/usr/bin/env bash
set -euo pipefail

# Quick helper to spin up a kind cluster for Gateway API testing
# Usage: CLUSTER_NAME=switchboard-gateway ./setup-kind.sh

SCRIPT_DIR="$(cd -- "$(dirname -- "$0")" && pwd)"
CLUSTER_NAME=${CLUSTER_NAME:-switchboard-gateway}
KIND_CONFIG=${KIND_CONFIG:-"${SCRIPT_DIR}/kind-config.yaml"}

kind create cluster --name "${CLUSTER_NAME}" --config "${KIND_CONFIG}"

kubectl cluster-info --context "kind-${CLUSTER_NAME}"

kubectl create namespace gateway-system --dry-run=client -o yaml | kubectl apply -f -

kubectl apply -f https://github.com/kubernetes-sigs/gateway-api/releases/download/v1.4.1/standard-install.yaml
kubectl wait --for=condition=Available --timeout=180s -n gateway-system deploy/gateway-api-admission-server

kubectl apply -k "${SCRIPT_DIR}"

kubectl get gatewayclass,gateway,httproute -A
kubectl get svc,deploy -n default
