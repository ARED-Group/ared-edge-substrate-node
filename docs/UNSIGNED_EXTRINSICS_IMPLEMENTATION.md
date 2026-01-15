# Unsigned Extrinsics Implementation

Last Updated: January 3, 2026

This document describes the implementation of unsigned extrinsics for telemetry proof submission in the ARED Edge Substrate node.

## Background

### The Problem

The Substrate SDK version `polkadot-stable2503` introduced `TransactionExtension` to replace the older `SignedExtension` trait. This change affects how extrinsics are encoded and validated.

**Issue Encountered:**
- All client libraries (subxt 0.44.x, substrate-interface 1.7.x) expect the V4 extrinsic format with `SignedExtension`
- The new SDK uses V5 extrinsic format with `TransactionExtension`
- Attempts to submit signed extrinsics result in: `Invalid Transaction (1010): Transaction has a bad signature`

**Root Cause:**
The signature validation fails because client libraries encode extrinsics using the old format, but the runtime expects the new format. This is a fundamental incompatibility that cannot be resolved by adjusting client code alone.

### Solutions Evaluated

**Option A: Downgrade SDK to polkadot-stable2409**
- Would restore `SignedExtension` compatibility
- Rejected: Version conflict with `sc-network` crate causes build failures

**Option B: Wait for client library updates**
- subxt and substrate-interface need updates for `TransactionExtension`
- Rejected: Timeline uncertain, blocks development progress

**Option C: Unsigned Extrinsics with ValidateUnsigned (Implemented)**
- Add unsigned extrinsic variants for proof submission
- Implement `ValidateUnsigned` trait for custom validation
- Bypasses signature requirement entirely

## Implementation Details

### Pallet Changes

The `telemetry-proofs` pallet was updated to support unsigned extrinsics:

**New Dispatchable Functions:**

```rust
#[pallet::call_index(3)]
pub fn submit_proof_unsigned(
    origin: OriginFor<T>,
    device_id: BoundedVec<u8, ConstU32<64>>,
    proof_hash: BoundedVec<u8, ConstU32<64>>,
    record_count: u32,
    window_start: u64,
    window_end: u64,
) -> DispatchResult

#[pallet::call_index(4)]
pub fn submit_batch_proofs_unsigned(
    origin: OriginFor<T>,
    proofs: BoundedVec<ProofData<T>, ConstU32<100>>,
) -> DispatchResult
```

**ValidateUnsigned Implementation:**

```rust
impl<T: Config> ValidateUnsigned for Pallet<T> {
    type Call = Call<T>;

    fn validate_unsigned(_source: TransactionSource, call: &Self::Call) -> TransactionValidity {
        match call {
            Call::submit_proof_unsigned { device_id, proof_hash, window_start, window_end, .. } => {
                // Validation checks:
                // - device_id length (1-64 bytes)
                // - proof_hash length (exactly 32 bytes)
                // - window_end > window_start
                // Returns ValidTransaction with unique tag
            }
            Call::submit_batch_proofs_unsigned { proofs } => {
                // Validation checks:
                // - batch not empty
                // - batch size <= 100
                // Returns ValidTransaction with unique tag
            }
            _ => InvalidTransaction::Call.into(),
        }
    }
}
```

### Runtime Configuration

The runtime automatically picks up the `ValidateUnsigned` implementation through the `construct_runtime!` macro:

```rust
construct_runtime!(
    pub struct Runtime {
        // ...
        TelemetryProofs: pallet_telemetry_proofs,
    }
);
```

No additional configuration is required in `runtime/src/lib.rs`.

### Client Changes

The blockchain-bridge Python client was updated to use unsigned extrinsics:

```python
# Before (signed extrinsic - broken)
call = substrate.compose_call(
    call_module="TelemetryProofs",
    call_function="submit_proof",
    call_params={...}
)
extrinsic = substrate.create_signed_extrinsic(call=call, keypair=keypair)

# After (unsigned extrinsic - working)
call = substrate.compose_call(
    call_module="TelemetryProofs",
    call_function="submit_proof_unsigned",
    call_params={...}
)
extrinsic = substrate.create_unsigned_extrinsic(call=call)
```

## Security Considerations

### Authentication Model

Unsigned extrinsics bypass cryptographic signature verification. For the ARED platform, this is acceptable because:

1. **Upstream Authentication**: Telemetry data is authenticated at the MQTT layer via mTLS or password credentials before reaching the blockchain bridge
2. **Data Validation**: The ingest service validates, deduplicates, and stores telemetry in PostgreSQL before any blockchain submission
3. **Internal Network**: The blockchain bridge runs within the K3s cluster and is not exposed externally
4. **Proof Integrity**: Proof hashes are computed from validated telemetry data

### ValidateUnsigned Checks

The `ValidateUnsigned` implementation performs these validation checks:

- Device ID must be 1-64 bytes (prevents empty or oversized identifiers)
- Proof hash must be exactly 32 bytes (SHA-256 hash)
- Time window must be valid (end > start)
- Batch size must be 1-100 proofs (prevents spam)

### Transaction Uniqueness

Each unsigned transaction includes a unique tag based on:
- Device ID
- Window start/end timestamps
- Current block number

This prevents duplicate transactions from being included in the same block.

## Deployment Notes

### Building the Runtime

```bash
cd ared-edge-substrate-node
cargo build --release
```

### Chain Data Migration

When deploying the new runtime with unsigned extrinsic support, you must either:

1. **Fresh Chain Start**: Delete existing chain data to use the new runtime from genesis
2. **Runtime Upgrade**: Use on-chain governance to upgrade the runtime (preserves chain history)

For development environments, option 1 is simpler:

```bash
# On the edge server
sudo rm -rf /var/lib/ared-edge/substrate/data/*
kubectl delete pod -n ared-edge substrate-node-0
```

### Verifying the Implementation

Test unsigned extrinsic submission:

```python
from substrateinterface import SubstrateInterface
import time

substrate = SubstrateInterface(url="ws://substrate-node:9944", ss58_format=42)

call = substrate.compose_call(
    call_module="TelemetryProofs",
    call_function="submit_proof_unsigned",
    call_params={
        "device_id": b"test-device-001",
        "proof_hash": bytes.fromhex("a" * 64),
        "record_count": 10,
        "window_start": int(time.time()) - 3600,
        "window_end": int(time.time()),
    }
)

extrinsic = substrate.create_unsigned_extrinsic(call=call)
receipt = substrate.submit_extrinsic(extrinsic, wait_for_inclusion=True)
print(f"Block: {receipt.block_hash}")
```

## Related Files

- `pallets/telemetry-proofs/src/lib.rs` - Pallet implementation with ValidateUnsigned
- `runtime/src/lib.rs` - Runtime configuration
- `services/blockchain-bridge/src/app/services/substrate_client.py` - Python client (in edge-iot-mqtt-services repo)

## Commits

- `968c38f` - fix ValidateUnsigned batch provides tag to use u32 instead of usize
- `0834319` - add unsigned extrinsic support for telemetry proofs with ValidateUnsigned
- `12f29ed` - update blockchain bridge to use unsigned extrinsics for proof submission
