# ARED Edge Blockchain - Runtime Configuration

This document describes the runtime configuration parameters for the ARED Edge blockchain.

## Overview

The ARED Edge runtime is built on Substrate and includes:
- Core frame pallets (system, timestamp, balances)
- Consensus pallets (Aura, Grandpa)
- Custom pallets (TelemetryProofs, CarbonCredits)

## Block Time Configuration

- **MILLISECS_PER_BLOCK:** 6000 (Target block time in milliseconds)
- **SLOT_DURATION:** 6000 (Aura slot duration)
- **MINUTES:** 10 blocks (Blocks per minute)
- **HOURS:** 600 blocks (Blocks per hour)
- **DAYS:** 14,400 blocks (Blocks per day)

Block time of 6 seconds provides a balance between:
- Fast enough for responsive telemetry proof submission
- Slow enough for network propagation and finality
- Sustainable for long-term operation

## Weight Configuration

### Maximum Block Weight

```rust
const MAXIMUM_BLOCK_WEIGHT: Weight = Weight::from_parts(
    WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2),
    u64::MAX,
);
```

This allows approximately 2 seconds of computation per block.

### Weight to Fee Conversion

Transaction fees are calculated using identity fee model:
- Weight maps directly to fee units
- Simple and predictable fee calculation

```rust
type WeightToFee = frame_support::weights::IdentityFee<Balance>;
type LengthToFee = frame_support::weights::IdentityFee<Balance>;
```

### Operational Fee Multiplier

```rust
type OperationalFeeMultiplier = ConstU8<5>;
```

Operational transactions pay 5x the normal fee, ensuring they can be prioritized.

## Pallet Configurations

### System Pallet

- **BlockHashCount:** 256 (Number of recent block hashes to keep)
- **AccountData:** pallet_balances::AccountData (Account balance storage)

### Timestamp Pallet

- **MinimumPeriod:** 3000 ms (Half of slot duration)

### Balances Pallet

- **MaxLocks:** 50 (Maximum balance locks per account)
- **ExistentialDeposit:** 500 (Minimum balance for account existence)

### Transaction Payment Pallet

- **OperationalFeeMultiplier:** 5 (Priority fee multiplier)
- **FeeMultiplierUpdate:** ConstFeeMultiplier(1) (Static fee multiplier)

### Consensus Pallets

#### Aura

- **MaxAuthorities:** 32 (Maximum block producers)
- **AllowMultipleBlocksPerSlot:** false (One block per slot)

#### Grandpa

- **MaxAuthorities:** 32 (Maximum finalizers)
- **MaxNominators:** 0 (No nomination, private chain)

### Telemetry Proofs Pallet

- **MaxDeviceIdLength:** 64 (UUID 36 + buffer for future formats)
- **MaxProofLength:** 128 (SHA-256 hex 64 + metadata buffer)
- **MaxBatchSize:** 100 (Balance between efficiency and block weight)
- **MaxProofsPerDevice:** 10,000 (Approximately 1 year of daily proofs with margin)

### Carbon Credits Pallet

- **MaxDeviceIdLength:** 64 (Consistent with TelemetryProofs)
- **CreditsPerTonCO2:** 1,000 (1 credit = 1 kg CO2 avoided)
- **DefaultEmissionFactor:** 1500 (1.5 kg CO2/kWh, traditional cooking baseline)
- **MinClaimableEnergy:** 1,000 Wh (Minimum 1 kWh to prevent dust claims)
- **MaxIssuanceRecords:** 10,000 (Consistent retention with proofs)

## Carbon Credit Calculation

### Emission Factor

The default emission factor of 1.5 kg CO2/kWh is based on:
- Traditional biomass cooking emissions
- Average fuel efficiency of displaced cooking methods
- Conservative estimate for carbon credit verification

This can be adjusted via governance (`set_emission_factor`).

### Credit Formula

```
CO2 avoided (kg) = Energy (kWh) × Emission Factor (kg/kWh)
Credits = CO2 avoided (kg) / 1000 × Credits per ton
```

Example:
- Energy: 100 kWh
- Emission Factor: 1.5 kg/kWh
- CO2 avoided: 150 kg
- Credits: 150 × (1000/1000) = 150 credits

## Runtime Version

```rust
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: "ared-edge",
    impl_name: "ared-edge-node",
    authoring_version: 1,
    spec_version: 100,
    impl_version: 1,
    transaction_version: 1,
    state_version: 1,
};
```

### Version Semantics

- **spec_version**: Increment for runtime logic changes
- **impl_version**: Increment for non-breaking implementation changes
- **transaction_version**: Increment for transaction format changes
- **state_version**: Increment for state encoding changes

## Runtime Upgrades

### Upgrade Process

1. Build new runtime WASM
2. Test on development network
3. Submit upgrade via sudo (dev) or governance (prod)
4. Runtime hot-swaps at designated block

### Migration Considerations

When upgrading:
1. Ensure storage migrations are in place
2. Test migration on testnet
3. Include migration weights
4. Verify post-migration state

### Storage Migration Template

```rust
pub struct Migration;

impl OnRuntimeUpgrade for Migration {
    fn on_runtime_upgrade() -> Weight {
        // Migration logic
        Weight::from_parts(0, 0)
    }
    
    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
        // Pre-upgrade checks
        Ok(Vec::new())
    }
    
    #[cfg(feature = "try-runtime")]
    fn post_upgrade(_state: Vec<u8>) -> Result<(), &'static str> {
        // Post-upgrade verification
        Ok(())
    }
}
```

## Benchmarking

### Running Benchmarks

```bash
cargo build --release --features runtime-benchmarks

./target/release/ared-edge-node benchmark pallet \
    --chain dev \
    --pallet pallet_telemetry_proofs \
    --extrinsic "*" \
    --steps 50 \
    --repeat 20 \
    --output ./pallets/telemetry-proofs/src/weights.rs
```

### Weight Structure

Weights include:
- **Ref Time**: Computational cost
- **Proof Size**: State access cost

Each extrinsic weight accounts for:
- Database reads
- Database writes
- Computation
- Event emission

## Security Configuration

### Transaction Validation

All transactions include:
- NonZeroSender check
- Spec version check
- Transaction version check
- Genesis hash check
- Era check (mortality)
- Nonce check
- Weight check
- Fee payment

### Access Control

- TelemetryProofs: Any signed account can submit proofs
- CarbonCredits: Any signed account can record energy
- Governance functions require Root origin

## Performance Tuning

### Block Production

For optimal performance:
- Use NVMe storage for chain data
- Allocate sufficient CPU for block production
- Ensure network connectivity between validators

### RPC Configuration

Recommended limits:
- Max connections: 100
- Request timeout: 60s
- Batch request limit: 100

### Database Configuration

RocksDB tuning:
- Use SSD/NVMe storage
- Allocate appropriate cache size
- Enable compression for storage efficiency

## Monitoring Integration

### Prometheus Metrics

The node exposes metrics on port 9615:
- Block height
- Transaction pool size
- Peer count
- Block production timing

### Telemetry

Optional telemetry can be sent to:
- Internal monitoring systems
- Substrate telemetry servers (development only)

## Disaster Recovery

### Backup Strategy

1. Regular state snapshots
2. Warp sync endpoints for fast recovery
3. Validator key backups (secure storage)

### Recovery Procedures

1. Restore from snapshot
2. Sync to current block
3. Verify state integrity
4. Resume block production
