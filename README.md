# ARED Edge — Substrate Node

Short description
The Substrate Node is the private blockchain runtime that provides the ledger, transaction execution, on-chain logic, and event emission for the ARED Edge platform. It runs as a long-lived node in the cluster and serves as the authoritative source of on-chain state.

What this system does
- Maintain the distributed ledger and execute transactions (extrinsics).
- Host the runtime (WASM + Rust pallets) that implements business logic: token economics, proofs, carbon calculation, device assertions, and governance.
- Provide RPC and gRPC endpoints for clients to submit extrinsics, query chain state, and subscribe to events.
- Emit normalized events for downstream consumers (indexers, integration services).
- Persist chain state to durable storage and expose snapshot/restore capabilities.

Primary responsibilities
- Consensus & Networking: peer discovery, block production/validation, and transaction propagation.
- Runtime Execution: run the compiled runtime (WASM) and execute pallet logic deterministically.
- Transaction Handling: pool accepted extrinsics, transaction fee handling, and block inclusion.
- Event Emission & Indexing Hooks: generate events from blocks and extrinsics for the indexer and external consumers.
- State Persistence & Recovery: store chain DB on a durable PVC and provide snapshot/restore.
- Admin & Maintenance: support runtime upgrades, telemetry, and health endpoints.

Key components
- runtime/: Substrate runtime crates and pallets (WASM target).
- node/: Node binary, networking, and RPC server.
- indexer/: Off-chain indexer that consumes block events and writes normalized data to the off-chain DB.
- migration/: Scripts and utilities for on-chain migrations and runtime upgrades.
- k8s/: Kubernetes manifests, PVC templates, and resource configurations.
- dev/: Local development tooling, docker-compose, and helper scripts.

APIs and interfaces
- JSON-RPC / WebSocket: standard Substrate RPC for querying state and submitting extrinsics.
- gRPC (internal): telemetry, control, and admin interfaces for internal automation.
- Events stream (e.g., Kafka / Postgres notifications / HTTP webhooks): published for blocks, extrinsics, and custom runtime events.
- Admin endpoints: health, readiness, metrics, and upgrade triggers (secured).

Data model & persistence
- On-chain storage: RocksDB/ParityDB persisted to a PVC (Longhorn recommended).
- Off-chain store: Postgres (or equivalent) for normalized events, indexes, and derived data used by UIs and integration services.
- Snapshot storage: scheduled snapshots retained in object storage for recovery.

Deployment & runtime characteristics
- Runs as a long-lived container in K3s with CPU and memory resource limits.
- Requires a durable PVC for chain data and configured backup/snapshot policies.
- Liveness and readiness probes should be configured to avoid routing traffic during startup/compaction.
- TLS certificates and keys are provided via Kubernetes secrets for RPC/admin endpoints.

Operational behavior
- Starts consensus and connects to peers for block propagation.
- Continuously executes transactions and writes blocks to the local DB.
- Publishes events to the configured event sink for consumption by indexer and anchors.
- Supports runtime upgrades via on-chain proposals or admin-triggered upgrades.

Configuration
- NODE_ENV / RUST_LOG: logging & environment mode.
- STORAGE_PATH: chain DB location inside the container.
- INDEXER_DB_URL: connection string for the off-chain DB.
- METRICS_PORT: Prometheus exporter port.
- ADMIN_API_KEY / TLS_SECRETS: credentials & TLS materials via K8s secrets.

Observability
- Prometheus metrics (block time, tx throughput, memory, CPU).
- Structured logs shipped to centralized logging (ELK/Loki).
- Optional tracing for RPC and runtime hotspots.

Testing & CI
- Unit tests for pallets and runtime logic.
- Integration tests that spin up a single-node chain and exercise common extrinsics.
- CI pipeline builds the runtime, node binary, runs tests, and publishes container images.

Who owns it
- Primary: Blockchain Team (OWNERS file lists maintainers)
- Secondary: Platform/DevOps for infra and deployment

Local development (quickstart)
- Build runtime and node: cargo build --release (or use provided container images).
- Run a single-node dev chain (see dev/ folder & docker-compose scripts).
- Use polkadot.js apps or RPC clients to submit extrinsics and inspect state.

Files & layout (high level)
- runtime/      — runtime crates and pallets
- node/         — node binary, CLI and startup scripts
- indexer/      — block event consumer and normalizer
- k8s/          — deployment manifests, PVC, and secrets templates
- dev/          — local dev tooling, docker-compose
- docs/         — API schemas, event definitions, and runtime docs

Contact & support
- See OWNERS for maintainers and escalation paths.
- Use issues to report bugs or request features; label appropriately (bug, enhancement, infra).

Integration with Edge Services

This node integrates with the edge-iot-mqtt-services repository for telemetry ingestion and proof generation.

Related Documentation
- [Failure Recovery Matrix](../edge-iot-mqtt-services/docs/FAILURE_RECOVERY_MATRIX.md) - Recovery procedures for Substrate failures
- [Prospect Integration](../edge-iot-mqtt-services/docs/PROSPECT_INTEGRATION.md) - Cloud sync of proof references
- [Storage Retention](../edge-iot-mqtt-services/docs/STORAGE_RETENTION.md) - Data retention policies

Storage Requirements
**Storage Requirements:**

- Chain DB: Longhorn PVC, 50GB minimum, RocksDB/ParityDB grows with chain history
- Snapshots: MinIO/S3, 100GB, periodic backups for disaster recovery
- Logs: Ephemeral, 1GB, structured JSON shipped to centralized logging

Failure Recovery
**Failure Recovery:**

- Node crash: Detected by liveness probe failure, recovery via K8s auto-restart and resume from persisted state
- DB corruption: Detected by health check and block import errors, recovery via restore from snapshot and resync from peers
- Network partition: Detected by peer count drop and finalization stall, recovery via wait for network recovery and manual peer injection
- Resource exhaustion: Detected by OOM kill and CPU throttle, recovery via increase limits and optimize runtime

License
- Add LICENSE in repo root (choose appropriate open-source license).
