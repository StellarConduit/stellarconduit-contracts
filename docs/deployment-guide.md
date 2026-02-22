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

```bash
# Generate a new keypair
stellar keys generate --global testnet-deployer --network testnet

# Fund the account via Friendbot
stellar keys fund testnet-deployer --network testnet

# Verify the account is funded
stellar account show testnet-deployer --network testnet
```

### 2. Deploy Using the Script

```bash
# Deploy the relay registry
bash scripts/deploy-testnet.sh relay-registry

# Deploy all contracts
bash scripts/deploy-testnet.sh all
```

### 3. Manual Deployment

<!-- TODO: Step-by-step manual deployment instructions using stellar contract upload + deploy. -->

### 4. Deployment Order

Contracts must be deployed in the following order due to cross-contract dependencies:

1. **Treasury** — no dependencies
2. **Relay Registry** — no dependencies
3. **Fee Distributor** — depends on Relay Registry and Treasury addresses at initialization
4. **Dispute Resolver** — depends on Relay Registry address at initialization

<!-- TODO: Document exact initialization parameters for each contract. -->

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

<!-- TODO: Full mainnet deployment steps including multi-sig signing flow. -->

### Emergency Contact

If a critical issue is discovered post-deployment, contact the security team immediately at **security@stellarconduit.org**.

---

## Post-Deployment Verification

<!-- TODO: Steps to verify a contract is deployed and functioning correctly after deployment. -->

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
