# StellarConduit — Contract Specifications

> Technical specification document for all Soroban smart contracts in the StellarConduit protocol.
> This document is a living specification. Sections are filled in as contracts are designed and implemented.

---

## Table of Contents

- [Overview](#overview)
- [Relay Registry Contract](#relay-registry-contract)
- [Fee Distributor Contract](#fee-distributor-contract)
- [Dispute Resolver Contract](#dispute-resolver-contract)
- [Treasury Contract](#treasury-contract)
- [Cross-Contract Interactions](#cross-contract-interactions)
- [Storage Layout Reference](#storage-layout-reference)
- [Error Code Reference](#error-code-reference)

---

## Overview

<!-- TODO: Brief overview of the contract system, the trust model, and how contracts relate to each other. -->

---

## Relay Registry Contract

**Package:** `relay-registry`
**Source:** [`contracts/relay-registry/`](../contracts/relay-registry/)

### Purpose

<!-- TODO: Describe the role of the relay registry in the StellarConduit trust model. -->

### State Machine

<!-- TODO: Diagram or description of the node state machine: Unregistered → Registered → Active → Inactive / Slashed. -->

### Function Specifications

#### `register(node_address: Address, metadata: NodeMetadata) -> Result<(), ContractError>`

<!-- TODO: Preconditions, postconditions, storage effects, events emitted, error cases. -->

#### `stake(amount: i128) -> Result<(), ContractError>`

<!-- TODO -->

#### `unstake(amount: i128) -> Result<(), ContractError>`

<!-- TODO -->

#### `slash(node_address: Address, reason: String) -> Result<(), ContractError>`

<!-- TODO -->

#### `get_node(address: Address) -> Result<RelayNode, ContractError>`

<!-- TODO -->

#### `is_active(address: Address) -> bool`

<!-- TODO -->

### Events

<!-- TODO: List all events emitted by this contract (topic, data). -->

### Security Considerations

<!-- TODO: Access control, reentrancy, overflow handling, minimum stake rationale. -->

---

## Fee Distributor Contract

**Package:** `fee-distributor`
**Source:** [`contracts/fee-distributor/`](../contracts/fee-distributor/)

### Purpose

<!-- TODO: Describe the fee distribution mechanism and treasury allocation. -->

### Fee Formula

<!-- TODO: Document the full fee formula with basis points, examples, rounding behavior. -->

### Function Specifications

#### `distribute(relay_address: Address, batch_id: u64) -> Result<(), ContractError>`

<!-- TODO -->

#### `calculate_fee(batch_size: u32) -> Result<i128, ContractError>`

<!-- TODO -->

#### `claim(relay_address: Address) -> Result<i128, ContractError>`

<!-- TODO -->

#### `get_earnings(relay_address: Address) -> EarningsRecord`

<!-- TODO -->

#### `set_fee_rate(rate: u32) -> Result<(), ContractError>`

<!-- TODO -->

### Events

<!-- TODO -->

### Security Considerations

<!-- TODO: Double-distribution prevention, treasury transfer atomicity, fee rate governance. -->

---

## Dispute Resolver Contract

**Package:** `dispute-resolver`
**Source:** [`contracts/dispute-resolver/`](../contracts/dispute-resolver/)

### Purpose

<!-- TODO: Describe the dispute lifecycle and the relay-chain proof model. -->

### Dispute Lifecycle

<!-- TODO: Diagram or description: Open → Responded → Resolved / Expired. -->

### Proof Evaluation Algorithm

<!-- TODO: Full description of how competing RelayChainProof values are evaluated deterministically. -->

### Function Specifications

#### `raise_dispute(tx_id: BytesN<32>, proof: RelayChainProof) -> Result<u64, ContractError>`

<!-- TODO -->

#### `respond(dispute_id: u64, proof: RelayChainProof) -> Result<(), ContractError>`

<!-- TODO -->

#### `resolve(dispute_id: u64) -> Result<Ruling, ContractError>`

<!-- TODO -->

#### `get_dispute(dispute_id: u64) -> Result<Dispute, ContractError>`

<!-- TODO -->

#### `get_ruling(dispute_id: u64) -> Result<Ruling, ContractError>`

<!-- TODO -->

### Events

<!-- TODO -->

### Security Considerations

<!-- TODO: Sybil-resistance, proof forgery, deadline enforcement, penalty triggering. -->

---

## Treasury Contract

**Package:** `treasury`
**Source:** [`contracts/treasury/`](../contracts/treasury/)

### Purpose

<!-- TODO: Describe the treasury's role in the protocol incentive model. -->

### Authorization Model

<!-- TODO: Describe the admin model and future multi-sig / DAO handover plan. -->

### Function Specifications

#### `deposit(amount: i128) -> Result<(), ContractError>`

<!-- TODO -->

#### `withdraw(amount: i128, recipient: Address, reason: String) -> Result<(), ContractError>`

<!-- TODO -->

#### `allocate(program: String, amount: i128) -> Result<(), ContractError>`

<!-- TODO -->

#### `get_balance() -> i128`

<!-- TODO -->

#### `get_history() -> Vec<TreasuryEntry>`

<!-- TODO -->

### Events

<!-- TODO -->

### Security Considerations

<!-- TODO: Authorization, withdrawal limits, reentrancy, spending program bounds. -->

---

## Cross-Contract Interactions

<!-- TODO: Diagram and description of how the four contracts call each other:
  - Fee Distributor → Relay Registry (is_active check)
  - Fee Distributor → Treasury (treasury share transfer)
  - Dispute Resolver → Relay Registry (slash call after ruling)
-->

---

## Storage Layout Reference

<!-- TODO: Consolidated table of all DataKey variants, storage tier (Instance vs Persistent vs Temporary), and type. -->

---

## Error Code Reference

<!-- TODO: Consolidated table of all error codes across all contracts. -->
