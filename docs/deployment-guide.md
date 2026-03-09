# StellarConduit — Deployment Guide

> Step-by-step guide for deploying StellarConduit Soroban contracts to testnet and mainnet.

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Local Development Setup](#local-development-setup)
- [Building Contracts](#building-contracts)
- [Testnet Deployment](#testnet-deployment)
- [Mainnet Deployment](#mainnet-deployment)
- [Post-Deployment Verification](#post-deployment-verification)
- [Upgrade Process](#upgrade-process)
- [Rollback Procedure](#rollback-procedure)
- [Deployed Contract IDs](#deployed-contract-ids)

---

## Prerequisites

Before deploying, ensure you have the following tools installed and configured:

### Required Tools

| Tool | Minimum Version | Install |
|---|---|---|
| Rust | 1.74.0 | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Stellar CLI | 22.0.0 | See [Stellar Developer Docs](https://developers.stellar.org/docs/tools/developer-tools/stellar-cli) |
| wasm32 target | — | `rustup target add wasm32-unknown-unknown` |

### Verify Installation

```bash
rustc --version
stellar --version
rustup target list --installed | grep wasm32
```

### Pre-deployment Checklist (All Networks)

Before deploying to **testnet** or **mainnet**, confirm the following:

- **Rust toolchain installed**: `rustc --version` matches or exceeds the required version.
- **WASM target installed**: `rustup target list --installed | grep wasm32-unknown-unknown`.
- **Stellar CLI configured**:
  - `stellar --version` prints a supported version.
  - Network configuration includes `testnet` (and `mainnet` when relevant); see Stellar CLI docs for `stellar network add`.
- **Funded deployer identity created** (example for testnet):

```bash
# Create a named identity for deployments
stellar keys generate --global testnet-deployer --network testnet

# Fund it via Friendbot
stellar keys fund testnet-deployer --network testnet

# Sanity-check the account
stellar account show testnet-deployer --network testnet
```

- **Contracts compiled to optimized WASM** (see `Building Contracts` below).

---

## Local Development Setup

<!-- TODO: Full local setup instructions including devcontainer, Docker Compose for local Stellar network, etc. -->

---

## Building Contracts

```bash
# Build all contracts
stellar contract build

# Build a single contract
stellar contract build --package relay-registry

# Verify WASM outputs
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

### Expected WASM Outputs

After a successful build, you should see:
- `target/wasm32-unknown-unknown/release/relay_registry.wasm`
- `target/wasm32-unknown-unknown/release/fee_distributor.wasm`
- `target/wasm32-unknown-unknown/release/dispute_resolver.wasm`
- `target/wasm32-unknown-unknown/release/treasury.wasm`

---

## Testnet Deployment

Testnet deployments are safe to run freely for development and testing.

### 1. Generate and Fund a Testnet Keypair

If you have not already created a deployer identity (see **Prerequisites**), do so now:

```bash
# Generate a new keypair
stellar keys generate --global testnet-deployer --network testnet

# Fund the account via Friendbot
stellar keys fund testnet-deployer --network testnet

# Verify the account is funded
stellar account show testnet-deployer --network testnet
```

### 2. (Optional) Deploy Using the Helper Script

If you prefer to use the project-provided helper script:

```bash
# Deploy the relay registry
bash scripts/deploy-testnet.sh relay-registry

# Deploy all contracts
bash scripts/deploy-testnet.sh all
```

The remainder of this section documents the **equivalent manual steps** using `stellar` directly.

### 3. Manual Deployment (Topological Order)

Contracts must be deployed in the following order due to cross-contract dependencies:

1. **Treasury** — no dependencies.
2. **Relay Registry** — no dependencies.
3. **Fee Distributor** — needs the **Treasury** contract address (and the SAC token contract).
4. **Dispute Resolver** — only depends on its own configuration.

#### 3.1 Define common environment variables

```bash
export SC_NETWORK=testnet
export SC_DEPLOYER=testnet-deployer

# Paths to compiled WASM artifacts
export TREASURY_WASM=target/wasm32-unknown-unknown/release/treasury.wasm
export RELAY_REGISTRY_WASM=target/wasm32-unknown-unknown/release/relay_registry.wasm
export FEE_DISTRIBUTOR_WASM=target/wasm32-unknown-unknown/release/fee_distributor.wasm
export DISPUTE_RESOLVER_WASM=target/wasm32-unknown-unknown/release/dispute_resolver.wasm
```

#### 3.2 Deploy Treasury (step 1 of 4)

```bash
stellar contract deploy \
  --wasm "$TREASURY_WASM" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK"
```

The command prints the **contract ID**. Copy it and set:

```bash
export TREASURY_CONTRACT_ID=<TREASURY_CONTRACT_ID>
```

#### 3.3 Deploy Relay Registry (step 2 of 4)

```bash
stellar contract deploy \
  --wasm "$RELAY_REGISTRY_WASM" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK"
```

Then:

```bash
export RELAY_REGISTRY_CONTRACT_ID=<RELAY_REGISTRY_CONTRACT_ID>
```

#### 3.4 Deploy Fee Distributor (step 3 of 4)

```bash
stellar contract deploy \
  --wasm "$FEE_DISTRIBUTOR_WASM" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK"
```

Then:

```bash
export FEE_DISTRIBUTOR_CONTRACT_ID=<FEE_DISTRIBUTOR_CONTRACT_ID>
```

#### 3.5 Deploy Dispute Resolver (step 4 of 4)

```bash
stellar contract deploy \
  --wasm "$DISPUTE_RESOLVER_WASM" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK"
```

Then:

```bash
export DISPUTE_RESOLVER_CONTRACT_ID=<DISPUTE_RESOLVER_CONTRACT_ID>
```

At this point all four contracts are **deployed but not initialized**.

### 4. Initialization Sequence (Testnet)

The initialization order is as important as the deployment order:

1. **Treasury** — initialize admin council and token used for fees.
2. **Relay Registry** — initialize council, minimum stake, and lock period.
3. **Fee Distributor** — initialize council, fee configuration, and link to **Treasury** and the SAC token.
4. **Dispute Resolver** — initialize council and resolution window.

All examples below assume:

- `SC_NETWORK=testnet`
- `SC_DEPLOYER=testnet-deployer`
- A pre-deployed SAC token contract: `SAC_TOKEN_CONTRACT_ID=<SAC_TOKEN_CONTRACT_ID>`

Set the token contract ID:

```bash
export SAC_TOKEN_CONTRACT_ID=<SAC_TOKEN_CONTRACT_ID>
```

#### 4.1 Initialize Treasury

`TreasuryContract::initialize(env, council: AdminCouncil, token_address: Address)`

Example council (2-of-3) with placeholder addresses:

```bash
export COUNCIL_MEMBER_1=<COUNCIL_MEMBER_1_ADDRESS>
export COUNCIL_MEMBER_2=<COUNCIL_MEMBER_2_ADDRESS>
export COUNCIL_MEMBER_3=<COUNCIL_MEMBER_3_ADDRESS>
```

```bash
stellar contract invoke \
  --id "$TREASURY_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- initialize \
  --council '{"members":["'"$COUNCIL_MEMBER_1"'","'"$COUNCIL_MEMBER_2"'","'"$COUNCIL_MEMBER_3"'"],"threshold":2}' \
  --token_address "$SAC_TOKEN_CONTRACT_ID"
```

#### 4.2 Initialize Relay Registry

`RelayRegistryContract::initialize(env, council: AdminCouncil, min_stake: i128, stake_lock_period: u32)`

Example configuration:

- **Minimum stake**: `1000` tokens.
- **Stake lock period**: `1000` ledgers (adjust per protocol policy).

```bash
stellar contract invoke \
  --id "$RELAY_REGISTRY_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- initialize \
  --council '{"members":["'"$COUNCIL_MEMBER_1"'","'"$COUNCIL_MEMBER_2"'","'"$COUNCIL_MEMBER_3"'"],"threshold":2}' \
  --min_stake 1000 \
  --stake_lock_period 1000
```

#### 4.3 Initialize Fee Distributor

`FeeDistributorContract::initialize(env, council: AdminCouncil, fee_rate_bps: u32, treasury_share_bps: u32, treasury: Address, token: Address)`

Example configuration:

- **Fee rate**: `50` bps = `0.5%` of each batch.
- **Treasury share**: `2000` bps = `20%` of each fee goes to the treasury.
- **Treasury address**: `TREASURY_CONTRACT_ID` (from deploy step).
- **Token address**: `SAC_TOKEN_CONTRACT_ID`.

```bash
stellar contract invoke \
  --id "$FEE_DISTRIBUTOR_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- initialize \
  --council '{"members":["'"$COUNCIL_MEMBER_1"'","'"$COUNCIL_MEMBER_2"'","'"$COUNCIL_MEMBER_3"'"],"threshold":2}' \
  --fee_rate_bps 50 \
  --treasury_share_bps 2000 \
  --treasury "$TREASURY_CONTRACT_ID" \
  --token "$SAC_TOKEN_CONTRACT_ID"
```

This step **links the Fee Distributor to the Treasury** via `TREASURY_CONTRACT_ID` and to the SAC token via `SAC_TOKEN_CONTRACT_ID`.

#### 4.4 Initialize Dispute Resolver

`DisputeResolverContract::initialize(env, council: AdminCouncil, resolution_window: u32)`

Example configuration:

- **Resolution window**: `5000` ledgers.

```bash
stellar contract invoke \
  --id "$DISPUTE_RESOLVER_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- initialize \
  --council '{"members":["'"$COUNCIL_MEMBER_1"'","'"$COUNCIL_MEMBER_2"'","'"$COUNCIL_MEMBER_3"'"],"threshold":2}' \
  --resolution_window 5000
```

All four contracts are now **initialized and wired together** (Fee Distributor → Treasury; Dispute Resolver and Relay Registry share council configuration).

---

## Mainnet Deployment

> ⚠️ **Warning:** Mainnet deployment is irreversible and requires multi-sig authorization from core maintainers. Do not deploy to mainnet unilaterally.

### Pre-Deployment Checklist

Before any mainnet deployment, the following must be completed:

- [ ] Contract has been deployed and tested on testnet for at least 2 weeks
- [ ] All contract unit tests pass with ≥80% coverage
- [ ] External security audit has been completed and report published
- [ ] WASM checksum of the audited build is recorded in `docs/audit/`
- [ ] Deployment proposal issue has been opened and approved by maintainers
- [ ] Multi-sig authorization from at least 3 core maintainers has been obtained
- [ ] Deployment announcement has been posted in the community Discord

### Deployment Steps

Mainnet deployment uses **the same sequence and commands** as testnet, with the following changes:

- **Network flag**: Use `--network mainnet` instead of `--network testnet`.
- **Identities**:
  - Use production identities (e.g., `mainnet-deployer`) configured with the required multi-sig.
  - Ensure `SC_DEPLOYER` is set to the authorized mainnet identity.
- **Token contract**:
  - Use the production SAC token contract ID (`SAC_TOKEN_CONTRACT_ID`) agreed upon in governance.
- **Configuration values**:
  - Double-check `fee_rate_bps`, `treasury_share_bps`, `min_stake`, and `resolution_window` against the approved deployment proposal.

At a high level:

1. Build WASM artifacts (`stellar contract build`).
2. Deploy contracts in order: **Treasury → Relay Registry → Fee Distributor → Dispute Resolver**.
3. Initialize contracts in the same order, using production council members and configuration values.
4. Run the **Post-Deployment Verification** commands below on **mainnet** and record outputs in the deployment issue.

### Emergency Contact

If a critical issue is discovered post-deployment, contact the security team immediately at **security@stellarconduit.org**.

---

## Post-Deployment Verification

After deployment and initialization, use the following **read-only** invocations to confirm the system is live.

All commands below assume testnet; replace `testnet` with `mainnet` as needed.

### Verify Relay Registry

Check that an admin (or known relay node) is registered and inspect its node record:

```bash
stellar contract invoke \
  --id "$RELAY_REGISTRY_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- get_node \
  --address <RELAY_NODE_ADDRESS>
```

Check whether a node is active:

```bash
stellar contract invoke \
  --id "$RELAY_REGISTRY_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- is_active \
  --address <RELAY_NODE_ADDRESS>
```

### Verify Treasury

Confirm that the treasury is initialized and has the expected token balance:

```bash
stellar contract invoke \
  --id "$TREASURY_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- get_balance
```

Optionally, inspect treasury stats:

```bash
stellar contract invoke \
  --id "$TREASURY_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- get_treasury_stats
```

### Verify Fee Distributor

Inspect earnings for a relay node (should be zero immediately after deployment):

```bash
stellar contract invoke \
  --id "$FEE_DISTRIBUTOR_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- get_earnings \
  --relay_address <RELAY_NODE_ADDRESS>
```

Optionally, test fee calculation with an example batch size:

```bash
stellar contract invoke \
  --id "$FEE_DISTRIBUTOR_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- calculate_fee \
  --batch_size 100
```

### Verify Dispute Resolver

Confirm that dispute-related view methods are callable (will error if uninitialized):

```bash
stellar contract invoke \
  --id "$DISPUTE_RESOLVER_CONTRACT_ID" \
  --source "$SC_DEPLOYER" \
  --network "$SC_NETWORK" \
  -- get_dispute \
  --dispute_id 1
```

If no disputes exist yet, you should see a well-formed contract error (e.g., `DisputeNotFound`), which confirms the contract is responding on-chain.

---

## Upgrade Process

<!-- TODO: Document the contract upgrade process using Soroban's WASM update mechanism. -->

---

## Rollback Procedure

<!-- TODO: Document the rollback procedure if a deployed contract has a critical bug. -->

---

## Deployed Contract IDs

<!-- This table is updated automatically by the deploy scripts. -->

| Contract | Contract ID | Network | Deployed At | Deployer |
|---|---|---|---|---|
| relay-registry | — | testnet | — | — |
| fee-distributor | — | testnet | — | — |
| dispute-resolver | — | testnet | — | — |
| treasury | — | testnet | — | — |
| relay-registry | — | mainnet | — | — |
| fee-distributor | — | mainnet | — | — |
| dispute-resolver | — | mainnet | — | — |
| treasury | — | mainnet | — | — |
