# Switchboard Container and Kubernetes Publish Assets

This directory contains reference Containerfiles and Kubernetes manifests for three deployment modes.

Web assets are built with Bun in container builds.

## Images

- `ghcr.io/<org>/switchboard/sbk:latest`
- `ghcr.io/<org>/switchboard/sbc-web:latest`
- `ghcr.io/<org>/switchboard/all-in-one:latest`

## Build Examples

```bash
docker build -f publish/container/sbk.containerfile -t ghcr.io/<org>/switchboard/sbk:latest .
docker build -f publish/container/sbc-web.containerfile -t ghcr.io/<org>/switchboard/sbc-web:latest .
docker build -f publish/container/switchboard-all-in-one.containerfile -t ghcr.io/<org>/switchboard/all-in-one:latest .
```

## Kubernetes Install

```bash
kubectl apply -k publish/k8s/sbk
kubectl apply -k publish/k8s/sbc-web
kubectl apply -k publish/k8s/all-in-one
```

Each kustomization includes a `switchboard` namespace resource. Apply one mode at a time unless you explicitly want to run multiple modes in the same cluster.

SBC startup mode is controlled by `SBC_K8S`:

- `SBC_K8S=true` enables Kubernetes-managed mode (`--k8s` is appended)
- `SBC_K8S=false` keeps standalone mode
