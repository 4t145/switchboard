# Switchboard Gateway Review: Gaps in `crates/controller` and `crates/kernel`

## Scope

This review focuses on two critical crates:

- `crates/controller`
- `crates/kernel`

Goal: evaluate whether Switchboard is ready to be a production-grade universal gateway, and identify key gaps.

## Executive Assessment

The architecture already has a good foundation:

- `controller`: control-plane API, config resolve, storage, kernel orchestration
- `kernel`: data-plane runtime (TCP listeners, routes, TLS, service execution) and gRPC control endpoint

However, it is currently closer to a functional gateway core than an operations-ready universal gateway platform.

The biggest risks are not forwarding capability itself, but:

1. control-plane security
2. config consistency semantics
3. lifecycle reliability
4. HA and operability

## Critical Findings (Prioritized)

### P0 - Must Fix Immediately

#### 1) Control-plane security baseline is missing

- Controller HTTP management API is exposed and high privilege (`/api/kernel_manager/*`, `/api/resolve/*`, `/api/storage/*`).
- No clear authentication/authorization boundary.
- Default bind on wide interface increases attack surface.

References:

- `crates/controller/src/interface/http.rs`
- `crates/controller/src/interface/http/resolve.rs`
- `crates/controller/src/interface/http/storage.rs`

#### 2) Config application consistency is weak (partial success ambiguity)

- Controller updates local `current_config` before all kernels are confirmed successful.
- If only part of kernels apply successfully, control-plane state may diverge from actual data-plane state.

References:

- `crates/controller/src/kernel/discovery.rs`
- `crates/controller/src/interface/http/kernel_manager.rs`

#### 3) Kernel listener lifecycle bug

- Listener startup handle is not correctly persisted for shutdown path.
- Graceful shutdown may fail to stop controller listener as intended.

References:

- `crates/kernel/src/lib.rs`
- `crates/kernel/src/controller/listener.rs`

### P1 - High Priority Reliability Gaps

#### 4) Kernel address parsing and connectivity robustness

- `KernelAddr::from_str` handling for `http/https/grpc` can lose scheme semantics, increasing endpoint normalization risk.
- Connection management lacks robust reconnection/backoff orchestration at manager level.

References:

- `crates/controller/src/kernel.rs`
- `crates/controller/src/kernel/connection.rs`
- `crates/controller/src/kernel/grpc_client.rs`

#### 5) Discovery-to-connection loop is incomplete

- Discovery is mostly UDS scan and manual refresh.
- Config fields like periodic scan interval are not fully reflected in runtime behavior.
- Missing automated connect/reconnect and health-driven lifecycle.

References:

- `crates/controller/src/config.rs`
- `crates/controller/src/kernel/discovery.rs`

#### 6) Single-node storage limits control-plane HA

- Current storage provider is local SurrealDB + RocksDB.
- Multi-controller deployment, failover, and shared-state patterns are constrained.

References:

- `crates/controller/src/storage.rs`
- `crates/controller/src/storage/surrealdb_local.rs`

### P2 - Operability and Platform Completeness

#### 7) Observability and auditability are not enough

- Insufficient metrics and audit events for config rollout, kernel state, and failure attribution.
- HTTP error mapping is mostly coarse-grained.

References:

- `crates/controller/src/interface/http.rs`
- `crates/kernel/src/controller/grpc_service.rs`

#### 8) Universal gateway plugin ecosystem needs stronger contracts

- Resolver/provider extensibility exists, but platform-level guardrails are thin:
  - resource quotas
  - timeout/circuit-breaker policy envelope
  - plugin capability/version governance

References:

- `crates/controller/src/resolve.rs`
- `crates/kernel/src/registry.rs`

## Recommended Remediation Roadmap

### Phase 1 - Security and Correctness Baseline (P0)

1. Add management-plane authn/authz (token or mTLS; ideally both).
2. Restrict default bind strategy (loopback/internal network by default).
3. Enforce path/link permission boundaries for resolve/storage APIs.
4. Fix listener handle lifecycle bug.
5. Change config apply semantics:
   - either all-success then commit `current_config`
   - or explicit transactional state machine with partial-failure status.

### Phase 2 - Reliability and HA (P1)

1. Implement robust kernel session management:
   - reconnect with backoff
   - health checks
   - stream re-subscription
2. Complete periodic discovery and auto-connect loop.
3. Add stale UDS socket handling on restart.
4. Introduce distributed/shared control-plane storage option.

### Phase 3 - Operability and Platformization (P2)

1. Add metrics/tracing for rollout SLOs:
   - apply latency
   - success/partial/failure ratios
   - listener bind failures
   - kernel connectivity states
2. Add audit logs:
   - who changed what
   - when
   - target kernels
   - config version/digest
3. Define plugin contracts and governance:
   - capability model
   - timeout/resource policy
   - version compatibility matrix

## Conclusion

Switchboard already has a solid control/data-plane split and a hot-reload-capable runtime structure.

To become a production-grade universal gateway, the next step is to prioritize:

1. security hardening
2. config consistency semantics
3. lifecycle reliability
4. HA and observability

Once these are in place, ecosystem extensibility (resolver/provider) can scale safely.

## Appendix: Notable Technical Notes

- Kernel gRPC update path includes config digest validation (`bincode` + version check), which is a strong foundation.
- Controller storage uses content digest verification for object integrity.
- Router/listener separation in kernel event loop is a good architectural choice for hot updates.
