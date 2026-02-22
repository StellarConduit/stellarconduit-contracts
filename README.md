# StellarConduit Contracts

> On-chain smart contract layer for the StellarConduit protocol — built on Stellar's Soroban smart contract platform.

This repository contains all Soroban smart contracts that power the trustless, on-chain components of StellarConduit. These contracts handle relay node registration and staking, protocol fee distribution, dispute resolution for contested transactions, and the protocol treasury.

---

## 📋 Table of Contents

- [Overview](#overview)
- [Contract Architecture](#contract-architecture)
- [Contracts](#contracts)
- [Repository Structure](#repository-structure)
- [Prerequisites](#prerequisites)
- [Getting Started](#getting-started)
- [Development](#development)
- [Testing](#testing)
- [Deployment](#deployment)
- [Security](#security)
- [Contributing](#contributing)
- [License](#license)

---

## Overview

StellarConduit's off-chain mesh network layer handles transaction propagation and offline signing, but certain protocol operations require trustless, on-chain enforcement. This is where the Soroban smart contracts come in.

The contracts in this repository are responsible for:

- **Relay Node Registry** — relay nodes must register and stake tokens on-chain before they are trusted by the protocol to submit transactions to Stellar on behalf of mesh participants
- **Fee Distribution** — when a relay node successfully settles a batch of transactions, it earns a protocol fee that is distributed automatically by the fee contract
- **Dispute Resolution** — in cases where double-spend conflicts cannot be resolved off-chain by the sync engine, the dispute contract provides a final on-chain arbitration mechanism
- **Protocol Treasury** — a governance-controlled treasury that funds relay node incentives, grants for operators in underserved regions, and protocol development

All contracts are written in Rust using the Soroban SDK and deployed on the Stellar network.

---

## Contract Architecture
```

┌─────────────────────────────────────────────────────────┐
│                   StellarConduit Protocol                │
└─────────────────────────────────────────────────────────┘
                            │
          ┌─────────────────┼─────────────────┐
          │                 │                 │
          ▼                 ▼                 ▼
┌──────────────────┐ ┌──────────────┐ ┌──────────────────┐
│  Relay Registry  │ │ Fee Contract │ │ Dispute Contract  │
│                  │ │              │ │                   │
│ - register()     │ │ - distribute()│ │ - raise()        │
│ - stake()        │ │ - claim()    │ │ - resolve()       │
│ - unstake()      │ │ - calculate()│ │ - arbitrate()     │
│ - slash()        │ │              │ │                   │
│ - get_node()     │ │              │ │                   │
└──────────────────┘ └──────────────┘ └──────────────────┘
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
                            ▼
               ┌─────────────────────┐
               │  Protocol Treasury  │
               │                     │
               │ - deposit()         │
               │ - withdraw()        │
               │ - allocate()        │
               └─────────────────────┘
```

---

## Contracts

### 1. Relay Registry Contract (`relay-registry`)
The relay registry is the foundation of StellarConduit's trust model. Any node that wants to participate as a relay — bridging the offline mesh to the Stellar network — must register on-chain and stake a minimum amount of tokens. This stake acts as a security deposit: if a relay node behaves maliciously (submitting tampered transactions, front-running, or going offline repeatedly), their stake is slashed by the protocol.

**Responsibilities:**
- Relay node registration with metadata (region, capacity, uptime commitment)
- Stake deposit and withdrawal with lock periods
- Node status tracking (active, inactive, slashed)
- Minimum stake enforcement
- Node lookup and verification by the sync engine and other contracts

**Key Functions:**
| Function | Description |
|---|---|
| `register(node_address, metadata)` | Register a new relay node |
| `stake(amount)` | Deposit stake tokens |
| `unstake(amount)` | Initiate stake withdrawal (subject to lock period) |
| `slash(node_address, reason)` | Slash a misbehaving relay node |
| `get_node(address)` | Fetch relay node details |
| `is_active(address)` | Check if a relay node is currently active |

---

### 2. Fee Distribution Contract (`fee-distributor`)
When a relay node successfully submits a batch of mesh transactions to Stellar and settlement is confirmed, the fee contract automatically calculates and distributes the relay fee to the node. Fees are funded by a small protocol fee added to each transaction at the point of signing. The fee contract is also responsible for allocating a portion of collected fees to the protocol treasury.

**Responsibilities:**
- Calculate relay fee based on batch size and transaction count
- Distribute fees to relay nodes upon confirmed settlement
- Allocate protocol treasury share
- Track fee history per relay node
- Handle fee claims for delayed distribution

**Key Functions:**
| Function | Description |
|---|---|
| `distribute(relay_address, batch_id)` | Distribute fee for a settled batch |
| `calculate_fee(batch_size)` | Calculate fee for a given batch |
| `claim(relay_address)` | Claim accumulated fees |
| `get_earnings(relay_address)` | View total earnings for a relay node |
| `set_fee_rate(rate)` | Update the protocol fee rate (governance only) |

---

### 3. Dispute Resolution Contract (`dispute-resolver`)
In rare cases where a double-spend conflict cannot be resolved deterministically by the off-chain sync engine — for example, when two relay nodes submit conflicting transactions from a partitioned mesh cluster simultaneously — the dispute contract provides the final arbitration layer. Either party can raise a dispute by submitting their cryptographic relay chain proof. The contract evaluates the proofs and determines which transaction is valid.

**Responsibilities:**
- Accept and validate dispute submissions with relay chain proofs
- Enforce dispute submission deadlines
- Evaluate competing cryptographic proofs deterministically
- Issue final ruling and trigger appropriate fund recovery
- Penalize the relay node that submitted the invalid transaction

**Key Functions:**
| Function | Description |
|---|---|
| `raise_dispute(tx_id, proof)` | Submit a dispute with relay chain proof |
| `respond(dispute_id, proof)` | Submit counter-proof to an open dispute |
| `resolve(dispute_id)` | Resolve a dispute after evaluation period |
| `get_dispute(dispute_id)` | Fetch dispute details and current status |
| `get_ruling(dispute_id)` | Fetch the final ruling for a resolved dispute |

---

### 4. Protocol Treasury Contract (`treasury`)
The protocol treasury holds funds allocated for relay node incentive programs, grants for operators running nodes in underserved and remote regions, and ongoing protocol development. In the future, treasury governance will be handed over to a DAO of protocol stakeholders.

**Responsibilities:**
- Receive fee allocations from the fee distributor
- Disburse grants to relay node operators
- Track all inflows and outflows with on-chain transparency
- Enforce spending limits and multi-sig authorization for withdrawals

**Key Functions:**
| Function | Description |
|---|---|
| `deposit(amount)` | Deposit funds into treasury |
| `withdraw(amount, recipient, reason)` | Withdraw funds (authorized only) |
| `allocate(program, amount)` | Allocate budget to a spending program |
| `get_balance()` | Fetch current treasury balance |
| `get_history()` | Fetch full transaction history |

---

## Repository Structure
```

stellarconduit-contracts/
├── contracts/
│   ├── relay-registry/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs
│   │   │   ├── storage.rs
│   │   │   └── errors.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── fee-distributor/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs
│   │   │   ├── storage.rs
│   │   │   └── errors.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   ├── dispute-resolver/
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs
│   │   │   ├── storage.rs
│   │   │   └── errors.rs
│   │   ├── Cargo.toml
│   │   └── README.md
│   └── treasury/
│       ├── src/
│       │   ├── lib.rs
│       │   ├── types.rs
│       │   ├── storage.rs
│       │   └── errors.rs
│       ├── Cargo.toml
│       └── README.md
├── tests/
│   ├── relay-registry-test.rs
│   ├── fee-distributor-test.rs
│   ├── dispute-resolver-test.rs
│   ├── treasury-test.rs
│   └── integration/
│       └── full-settlement-flow-test.rs
├── scripts/
│   ├── deploy-testnet.sh
│   ├── deploy-mainnet.sh
│   └── invoke-contract.sh
├── docs/
│   ├── contract-specs.md
│   └── deployment-guide.md
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── CONTRIBUTING.md
├── LICENSE
└── README.md
```

---

## Prerequisites

Before working with this repository, make sure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) `>=1.74.0`
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/stellar-cli) `>=22.0.0`
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) for building WASM targets
- A Stellar testnet account with test XLM — get one from [Stellar Friendbot](https://friendbot.stellar.org)

Verify your setup:
```bash
rustc --version
stellar version
```

---

## Getting Started

### 1. Clone the Repository
```bash
git clone https://github.com/StellarConduit/stellarconduit-contracts.git
cd stellarconduit-contracts
```

### 2. Install Rust WASM Target
```bash
rustup target add wasm32-unknown-unknown
```

### 3. Build All Contracts
```bash
stellar contract build
```

### 4. Run Tests
```bash
cargo test
```

### 5. Deploy to Testnet
```bash
# Make sure you have a testnet keypair configured
stellar keys generate --global testnet-deployer --network testnet

# Fund the account
stellar keys fund testnet-deployer --network testnet

# Deploy a specific contract
bash scripts/deploy-testnet.sh relay-registry
```

---

## Development

### Building a Single Contract
```bash
cd contracts/relay-registry
stellar contract build
```

### Running a Specific Test
```bash
cargo test -p relay-registry
```

### Invoking a Contract Function Locally
```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --source testnet-deployer \
  --network testnet \
  -- \
  register \
  --node_address <ADDRESS> \
  --metadata '{"region":"west-africa","capacity":100}'
```

### Code Style

We use `rustfmt` for formatting and `clippy` for linting. Before submitting a PR, always run:
```bash
cargo fmt --all
cargo clippy --all-targets --all-features
```

---

## Testing

Each contract has its own unit test suite inside its `src/` directory and an integration test in the `tests/` folder. The integration test suite simulates a full settlement flow — from relay node registration through transaction settlement and fee distribution — to ensure all contracts interact correctly.
```bash
# Run all tests
cargo test

# Run unit tests for a specific contract
cargo test -p relay-registry

# Run integration tests only
cargo test --test integration
```

We aim for a minimum of **80% test coverage** across all contracts. Coverage is checked automatically in CI.

---

## Deployment

### Testnet

Testnet deployment is safe to run freely for development and testing. Use the provided script:
```bash
bash scripts/deploy-testnet.sh <contract-name>
```

### Mainnet

Mainnet deployment requires multi-sig authorization from core maintainers. Never deploy to mainnet unilaterally. Follow the deployment guide in `docs/deployment-guide.md` and open a deployment proposal issue before proceeding.

---

## Security

Smart contract security is critical. We take the following precautions:

- All contracts go through internal review before any testnet deployment
- External security audits will be conducted before any mainnet deployment
- All deployed contract IDs and transaction hashes are published publicly in `docs/deployments.md`
- We maintain a responsible disclosure policy for vulnerabilities

**If you discover a security vulnerability, do not open a public issue.** Please send a responsible disclosure to **security@stellarconduit.org**. We will respond within 48 hours and credit you in our security acknowledgements.

---

## Contributing

We welcome contributions to the contracts repo from developers of all experience levels. If you are new to Soroban, the [Stellar Developer Docs](https://developers.stellar.org/docs/smart-contracts) are a great starting point.

**Good places to start:**
- Browse issues labeled [`good first issue`](https://github.com/StellarConduit/stellarconduit-contracts/issues?q=label%3A%22good+first+issue%22)
- Improve test coverage for existing contracts
- Help write the contract specification docs in `docs/contract-specs.md`
- Review open PRs and leave feedback

Please read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request.

---

## License

This repository is licensed under the [Apache 2.0 License](LICENSE). You are free to use, modify, and distribute this software under the terms of that license.

---

<div align="center">

Part of the [StellarConduit](https://github.com/StellarConduit) open-source organization.

**Payments that work everywhere. Even where the internet doesn't.**

</div>
