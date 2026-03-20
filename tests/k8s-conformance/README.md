# K8s Gateway API Conformance (Traefik-style)

This directory follows Traefik's conformance architecture:

- Run a dedicated Go test suite with build tag `gatewayAPIConformance`
- Bootstrap a disposable k3s cluster with `testcontainers-go`
- Apply CRDs + RBAC + Switchboard deployment manifests
- Run official Gateway API conformance profiles (HTTP + TLS)
- Emit normalized report files for CI usage

## Files

- `integration_test.go`: global flags/constants
- `gateway_api_conformance_test.go`: conformance suite implementation
- `fixtures/gateway-api-conformance/00-experimental-v1.4.1.yml`: Gateway API v1.4.1 experimental CRDs
- `fixtures/gateway-api-conformance/01-rbac.yml`: cluster permissions
- `fixtures/gateway-api-conformance/02-switchboard.yml.tmpl`: switchboard bootstrap manifest
- `gateway-api-conformance-reports/`: generated reports

## Prerequisites

1. Docker available locally
2. Go available locally
3. A local switchboard image already built

## Build image (example)

Use your own build pipeline. Ensure the local image tag matches `-switchboardImage`.

## Run

```bash
cd tests/k8s-conformance
go test ./... -v -tags gatewayAPIConformance -run GatewayAPIConformanceSuite
```

Run with explicit image/version:

```bash
cd tests/k8s-conformance
go test ./... -v -tags gatewayAPIConformance -run GatewayAPIConformanceSuite \
  -switchboardImage switchboard/sbc:conformance \
  -switchboardVersion dev
```

Run a single conformance test case:

```bash
cd tests/k8s-conformance
go test ./... -v -tags gatewayAPIConformance -run GatewayAPIConformanceSuite \
  -gatewayAPIConformanceRunTest HTTPRouteHostnameIntersection
```

From repository root with `just`:

```bash
just test-k8s-conformance-local
```

## Output

Reports are written to:

- `tests/k8s-conformance/gateway-api-conformance-reports/<gateway-api-version>/`

When tests fail (or `-showLogs=true` is set), runtime logs and cluster snapshots are written to:

- `tests/k8s-conformance/logs/<timestamp>/`

Artifacts include:

- `k3s.log`
- `switchboard.log`
- `resources.yaml`
- `events.log`

The report date is normalized to `-` for stable CI diffs.
