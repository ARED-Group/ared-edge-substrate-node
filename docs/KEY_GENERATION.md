# Production Key Generation Guide

This document describes how to generate secure validator keys and a production
chain spec for the ARED Edge Substrate node.

---

## Prerequisites

- `subkey` CLI (install via `cargo install subkey` or use the Docker image)
- Secure offline workstation for key generation
- Access to the node binary for chain spec export

---

## Step 1: Generate Validator Keys

For each validator node, generate an Aura (sr25519) and Grandpa (ed25519) keypair.

```bash
# Validator 1 — Aura (block production)
subkey generate --scheme sr25519 --output-type json > validator1_aura.json

# Validator 1 — Grandpa (finality)
subkey generate --scheme ed25519 --output-type json > validator1_grandpa.json
```

Repeat for each validator (minimum 3 recommended for production).

Store the secret seeds in a hardware security module or encrypted vault.
Only the public keys are needed for the chain spec.

---

## Step 2: Generate Account Keys

```bash
# Root/Sudo account (if applicable)
subkey generate --scheme sr25519 --output-type json > root.json

# Bridge account (submits telemetry proofs)
subkey generate --scheme sr25519 --output-type json > bridge.json
```

---

## Step 3: Export a Template Chain Spec

```bash
./ared-edge-node build-spec --chain=production --raw=false > chain-spec-template.json
```

---

## Step 4: Edit the Chain Spec

Open `chain-spec-template.json` and replace the placeholder public keys with the
real keys from Step 1 and Step 2:

- `genesis.runtime.aura.authorities` — list of Aura (sr25519) public keys
- `genesis.runtime.grandpa.authorities` — list of [Grandpa (ed25519) public key, weight] pairs
- `genesis.runtime.balances.balances` — list of [account, balance] pairs
- `genesis.runtime.sudo.key` — root account public key (remove for production if sudo pallet is disabled)

---

## Step 5: Convert to Raw Format

```bash
./ared-edge-node build-spec --chain=chain-spec-template.json --raw > chain-spec-production.json
```

---

## Step 6: Deploy

Provide the raw chain spec to the node at startup:

```bash
# Option A: via CLI argument
./ared-edge-node --chain=/etc/substrate/chain-spec-production.json

# Option B: via environment variable (used by production_config())
CHAIN_SPEC_PATH=/etc/substrate/chain-spec-production.json ./ared-edge-node --chain=production
```

In Kubernetes, mount the chain spec as a ConfigMap or Secret:

```yaml
env:
  - name: CHAIN_SPEC_PATH
    value: /config/chain-spec-production.json
volumeMounts:
  - name: chain-spec
    mountPath: /config
    readOnly: true
volumes:
  - name: chain-spec
    secret:
      secretName: substrate-chain-spec
```

---

## Security Checklist

- [ ] Keys generated on an air-gapped machine
- [ ] Secret seeds stored in HSM or encrypted vault
- [ ] Only public keys present in the chain spec JSON
- [ ] Chain spec JSON reviewed by at least two team members
- [ ] Backup of the raw chain spec stored securely
- [ ] Node key (libp2p identity) generated per-node and stored in K8s Secret
