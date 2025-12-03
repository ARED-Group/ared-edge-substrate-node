# ARED Edge Blockchain - Chain Specification

This document describes the chain specifications for the ARED Edge private blockchain network.

## Overview

The ARED Edge blockchain supports three network configurations:
- **Development**: Single-node local development
- **Local Testnet**: Multi-node local testing
- **Production**: Live mainnet deployment

## Network Configurations

### Development (`ared_edge_dev`)

Single-node configuration for local development and testing.

| Parameter | Value |
|-----------|-------|
| Chain ID | `ared_edge_dev` |
| Chain Type | Development |
| Protocol ID | `ared-edge-dev` |
| Validators | 1 (Alice) |
| Sudo | Enabled (Alice) |

**Pre-funded Accounts:**
- Alice: 1,000,000 ARED (full balance)
- Bob: 100,000 ARED (testing)
- Bridge: 100,000 ARED (proof submission)

### Local Testnet (`ared_edge_local`)

Multi-node configuration for local network testing.

| Parameter | Value |
|-----------|-------|
| Chain ID | `ared_edge_local` |
| Chain Type | Local |
| Protocol ID | `ared-edge-local` |
| Validators | 2 (Alice, Bob) |
| Sudo | Enabled (Alice) |

**Pre-funded Accounts:**
- Alice: 1,000,000 ARED
- Bob: 1,000,000 ARED
- Charlie: 100,000 ARED
- Bridge: 100,000 ARED

### Production (`ared_edge_mainnet`)

Production network configuration.

| Parameter | Value |
|-----------|-------|
| Chain ID | `ared_edge_mainnet` |
| Chain Type | Live |
| Protocol ID | `ared-edge` |
| Validators | 3+ (configurable) |
| Sudo | Disabled |

**Pre-funded Accounts:**
- Root: 1,000,000 ARED (governance)
- Bridge: 100,000 ARED (proof submission)
- Validator1-3: 100,000 ARED each (staking)

## Token Properties

| Property | Value |
|----------|-------|
| Symbol | ARED |
| Decimals | 18 |
| SS58 Prefix | 42 |

## Genesis Configuration

### Balances

Initial token distribution is configured per network type. All amounts are in smallest units (10^18 = 1 ARED).

### Aura Consensus

Block production uses Aura (Authority Round) consensus:
- Development: Single authority
- Local: Two authorities
- Production: Three or more authorities

### Grandpa Finality

Block finality uses Grandpa:
- Each validator has equal voting weight (1)
- 2/3+ majority required for finalization

### Sudo (Development Only)

Sudo account has elevated privileges:
- Update runtime code
- Force set storage
- Execute privileged calls

**Security Note:** Sudo is disabled in production. Governance mechanisms should be used instead.

## Account Roles

### Bridge Account

The bridge account is used by the blockchain bridge service to submit telemetry proofs. It requires:
- Sufficient balance for transaction fees
- Authorization to call `TelemetryProofs::submit_proof`
- Authorization to call `CarbonCredits::record_energy`

### Validator Accounts

Validators require:
- Aura key for block production
- Grandpa key for finality voting
- Sufficient balance for potential staking (future)

## Runtime Parameters

### Block Time

| Parameter | Value |
|-----------|-------|
| Slot Duration | 6000 ms |
| Block Time | 6 seconds |

### Time Constants

| Constant | Blocks |
|----------|--------|
| MINUTES | 10 |
| HOURS | 600 |
| DAYS | 14,400 |

### Weight Limits

| Parameter | Value |
|-----------|-------|
| Maximum Block Weight | 2 seconds of execution |
| Normal Dispatch Ratio | 75% |
| Operational Dispatch Ratio | 25% |

## Pallet Configuration

### Telemetry Proofs

| Parameter | Value |
|-----------|-------|
| MaxDeviceIdLength | 64 bytes |
| MaxProofLength | 128 bytes |
| MaxBatchSize | 100 proofs |
| MaxProofsPerDevice | 10,000 |

### Carbon Credits

| Parameter | Value |
|-----------|-------|
| MaxDeviceIdLength | 64 bytes |
| CreditsPerTonCO2 | 1,000 credits |
| DefaultEmissionFactor | 1500 (1.5 kg CO2/kWh) |
| MinClaimableEnergy | 1,000 Wh (1 kWh) |
| MaxIssuanceRecords | 10,000 |

## Generating Chain Spec Files

### Export Raw Chain Spec

```bash
# Development
./target/release/ared-edge-node build-spec --chain dev --raw > chain-spec-dev.json

# Local
./target/release/ared-edge-node build-spec --chain local --raw > chain-spec-local.json

# Production
./target/release/ared-edge-node build-spec --chain production --raw > chain-spec-prod.json
```

### Using Custom Chain Spec

```bash
./target/release/ared-edge-node --chain ./chain-spec-prod.json
```

## Production Key Generation

For production deployment, generate keys securely:

```bash
# Generate Aura key
./target/release/ared-edge-node key generate --scheme sr25519 --output-type json

# Generate Grandpa key
./target/release/ared-edge-node key generate --scheme ed25519 --output-type json

# Insert keys into keystore
./target/release/ared-edge-node key insert \
  --chain ./chain-spec-prod.json \
  --suri "<secret seed>" \
  --key-type aura

./target/release/ared-edge-node key insert \
  --chain ./chain-spec-prod.json \
  --suri "<secret seed>" \
  --key-type gran
```

## Security Considerations

1. **Never use development seeds in production**
2. Store validator keys securely using HSM or secure key management
3. Rotate keys periodically according to security policy
4. Monitor validator uptime and performance
5. Use encrypted communication between validators
6. Implement proper access controls for node administration

## Migration and Upgrades

### Runtime Upgrades

Runtime upgrades can be performed via:
1. Sudo call (development only)
2. Governance proposal (production)

### Storage Migrations

When updating storage layouts:
1. Implement `OnRuntimeUpgrade` trait
2. Test migrations on testnet first
3. Include migration weights in upgrade
4. Verify state integrity post-migration

## Monitoring

### Key Metrics

- Block production rate
- Finality delay
- Transaction throughput
- Validator participation
- Storage growth

### Alerting

Configure alerts for:
- Validator offline
- Finality stalled
- High block production latency
- Resource exhaustion
