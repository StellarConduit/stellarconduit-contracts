# StellarConduit Contract Specifications

> **Primary reference for frontend developers and integration partners.**
> This document covers the public ABI, storage layout, event schemas, and
> custom type dictionary for all four StellarConduit Soroban smart contracts.
> All contracts are written in Rust using the Soroban SDK v22.

---

## Table of Contents

1. [Relay Registry](#1-relay-registry)
2. [Fee Distributor](#2-fee-distributor)
3. [Dispute Resolver](#3-dispute-resolver)
4. [Treasury](#4-treasury)
5. [Custom Types Dictionary](#5-custom-types-dictionary)

---

## 1. Relay Registry

Manages relay node registration, token staking, and slashing. Nodes must
maintain a minimum stake to remain in `Active` status.

### 1.1 Public Functions (ABI)

#### `initialize`

| Field | Value |
|---|---|
| **Parameters** | `council: AdminCouncil`, `min_stake: i128`, `stake_lock_period: u32` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | None (first-caller-wins, one-time) |
| **Description** | One-time setup. Sets the admin council, minimum stake threshold, and the lock period (in ledgers) that must pass before an unstake can be finalized. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 10 | `AlreadyInitialized` | Contract has already been initialized |
| 11 | `InvalidAmount` | `min_stake ≤ 0` or `stake_lock_period == 0` |
| 13 | `InvalidCouncilConfig` | `threshold == 0` or `members.len() < threshold` |

---

#### `register`

| Field | Value |
|---|---|
| **Parameters** | `node_address: Address`, `metadata: NodeMetadata` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | `node_address` |
| **Description** | Registers a new relay node. The node starts with `stake = 0` and `status = Inactive`. Status is promoted to `Active` automatically when stake reaches `min_stake`. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `AlreadyRegistered` | A node with this address already exists |
| 8 | `InvalidMetadata` | `metadata.uptime_commitment > 100` |

---

#### `update_metadata`

| Field | Value |
|---|---|
| **Parameters** | `node_address: Address`, `new_metadata: NodeMetadata` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | `node_address` |
| **Description** | Replaces the metadata of an already-registered relay node. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 2 | `NotRegistered` | Node not found in the registry |
| 8 | `InvalidMetadata` | `uptime_commitment > 100` or `region.len() > 32` |

---

#### `stake`

| Field | Value |
|---|---|
| **Parameters** | `node_address: Address`, `amount: i128` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | `node_address` |
| **Description** | Transfers `amount` tokens from the node to the contract. If the node's total stake meets or exceeds `min_stake`, status is automatically promoted to `Active`. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 2 | `NotRegistered` | Node not found |
| 3 | `InsufficientStake` | `amount ≤ 0` |
| 5 | `NodeSlashed` | Node has been slashed |
| 9 | `Overflow` | Arithmetic overflow on stake accumulation |

---

#### `unstake`

| Field | Value |
|---|---|
| **Parameters** | `node_address: Address`, `amount: i128` |
| **Returns** | `Result<RelayNode, ContractError>` |
| **Auth Required** | `node_address` |
| **Description** | Initiates a stake withdrawal. Tokens are **not** transferred immediately — a `StakeEntry` is recorded with an `unlocks_at` timestamp. Call `finalize_unstake` after the lock period to receive tokens. If the remaining stake falls below `min_stake`, status is demoted to `Inactive`. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 2 | `NotRegistered` | Node not found |
| 3 | `InsufficientStake` | `amount ≤ 0` or `amount > node.stake` |
| 4 | `NodeNotActive` | Node is not in `Active` status |
| 5 | `NodeSlashed` | Node has been slashed |
| 9 | `Overflow` | Arithmetic overflow |

---

#### `finalize_unstake`

| Field | Value |
|---|---|
| **Parameters** | `node_address: Address` |
| **Returns** | `Result<i128, ContractError>` |
| **Auth Required** | `node_address` |
| **Description** | Completes a pending unstake by transferring the locked tokens back to the node after the lock period has elapsed. Returns the amount transferred. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 14 | `NoPendingUnstake` | No active unstake request exists |
| 15 | `LockPeriodActive` | Lock period has not yet elapsed |

---

#### `slash`

| Field | Value |
|---|---|
| **Parameters** | `node_address: Address`, `_reason: String` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | Admin council (M-of-N threshold) |
| **Description** | Permanently penalizes a misbehaving relay node. Sets `stake = 0` and `status = Slashed`. Slashed nodes cannot stake, unstake, or participate in the network. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 2 | `NotRegistered` | Node not found |
| 5 | `NodeSlashed` | Node is already slashed |

---

#### `get_node`

| Field | Value |
|---|---|
| **Parameters** | `address: Address` |
| **Returns** | `Result<RelayNode, ContractError>` |
| **Auth Required** | None (view) |
| **Description** | Returns the full `RelayNode` struct for the given address. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 2 | `NotRegistered` | Address not found |

---

#### `is_active`

| Field | Value |
|---|---|
| **Parameters** | `address: Address` |
| **Returns** | `bool` |
| **Auth Required** | None (view) |
| **Description** | Returns `true` if the node exists and has `status = Active`. Never errors — returns `false` for unknown or inactive addresses. |

---

### 1.2 Storage Layout

| Key | Type | Storage | Description |
|---|---|---|---|
| `RelayNode(Address)` | `RelayNode` | Persistent | Full node record keyed by node address. Bumped on every write. |
| `NodeCount` | `u32` | Instance | Total number of registered relay nodes. |
| `MinStake` | `i128` | Instance | Minimum stake required for `Active` status. |
| `StakeLockPeriod` | `u32` | Instance | Number of ledgers a node must wait before `finalize_unstake`. |
| `AdminCouncil` | `AdminCouncil` | Instance | M-of-N council authorized to slash nodes and update config. |
| `TokenAddress` | `Address` | Instance | SAC token address used for staking transfers. |
| `LockEntry(Address)` | `StakeEntry` | Persistent | Pending unstake record keyed by node address. Removed on `finalize_unstake`. |

---

### 1.3 Event Schemas

#### `relay_registry.register`
| Field | Value |
|---|---|
| **Topics** | `("relay_registry", "register")` |
| **Data** | `(node_address: Address, metadata: NodeMetadata)` |
| **Trigger** | Successful `register` call |

#### `relay_registry.update_metadata`
| Field | Value |
|---|---|
| **Topics** | `("relay_registry", "update_metadata")` |
| **Data** | `(node_address: Address,)` |
| **Trigger** | Successful `update_metadata` call |

#### `relay_registry.stake`
| Field | Value |
|---|---|
| **Topics** | `("relay_registry", "stake")` |
| **Data** | `(node_address: Address, amount: i128)` |
| **Trigger** | Successful `stake` call |

#### `relay_registry.unstake`
| Field | Value |
|---|---|
| **Topics** | `("relay_registry", "unstake")` |
| **Data** | `(node_address: Address, amount: i128, unlocks_at: u64)` |
| **Trigger** | Successful `unstake` call |

#### `relay_registry.finalize_unstake`
| Field | Value |
|---|---|
| **Topics** | `("relay_registry", "finalize_unstake")` |
| **Data** | `(node_address: Address, amount: i128)` |
| **Trigger** | Successful `finalize_unstake` call |

#### `relay_registry.slash`
| Field | Value |
|---|---|
| **Topics** | `("relay_registry", "slash")` |
| **Data** | `(node_address: Address, slashed_amount: i128)` |
| **Trigger** | Successful `slash` call |

---

## 2. Fee Distributor

Calculates and distributes protocol fees to relay nodes upon confirmed batch
settlement. Automatically routes the treasury share to the Treasury contract
via cross-contract invocation.

### 2.1 Public Functions (ABI)

#### `initialize`

| Field | Value |
|---|---|
| **Parameters** | `council: AdminCouncil`, `fee_rate_bps: u32`, `treasury_share_bps: u32`, `treasury: Address`, `token: Address` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | None (first-caller-wins, one-time) |
| **Description** | One-time setup. Configures the fee rate, treasury share, treasury contract address, and SAC token address. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `InvalidFeeRate` | `fee_rate_bps == 0` or `fee_rate_bps > 10000` |
| 9 | `InvalidCouncilConfig` | `threshold == 0` or `members.len() < threshold` |

---

#### `calculate_fee`

| Field | Value |
|---|---|
| **Parameters** | `batch_size: u32` |
| **Returns** | `Result<i128, ContractError>` |
| **Auth Required** | None (pure view) |
| **Description** | Calculates the fee for a given batch. Formula: `fee = (batch_size × fee_rate_bps) / 10000`. No storage is written. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 4 | `InvalidBatchSize` | `batch_size == 0` |
| 7 | `Overflow` | Arithmetic overflow in fee calculation |

---

#### `distribute`

| Field | Value |
|---|---|
| **Parameters** | `relay_address: Address`, `batch_id: u64`, `batch_size: u32` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | None (called by settlement layer) |
| **Description** | Records a fee distribution for a settled batch. Credits `relay_payout` to the relay node's unclaimed earnings. Transfers `treasury_share` to the Treasury contract via cross-contract call. Each `batch_id` can only be distributed once. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 3 | `BatchAlreadyDistributed` | `batch_id` already processed |
| 4 | `InvalidBatchSize` | `batch_size == 0` |
| 7 | `Overflow` | Arithmetic overflow in split calculation |
| 8 | `TreasuryTransferFailed` | Cross-contract deposit to treasury failed |

---

#### `claim`

| Field | Value |
|---|---|
| **Parameters** | `relay_address: Address` |
| **Returns** | `Result<i128, ContractError>` |
| **Auth Required** | `relay_address` |
| **Description** | Withdraws all accumulated unclaimed fees to the relay node. Resets `unclaimed` to `0` and increments `total_claimed`. Returns the amount paid out. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 5 | `NothingToClaim` | Relay node has no unclaimed earnings |
| 7 | `Overflow` | Arithmetic overflow updating `total_claimed` |

---

#### `get_earnings`

| Field | Value |
|---|---|
| **Parameters** | `relay_address: Address` |
| **Returns** | `EarningsRecord` |
| **Auth Required** | None (view) |
| **Description** | Returns lifetime earnings history for a relay node. Returns a zeroed record if the node has no history. |

---

#### `set_fee_rate`

| Field | Value |
|---|---|
| **Parameters** | `new_fee_rate_bps: u32` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | Admin council (M-of-N threshold) |
| **Description** | Updates the protocol fee rate in basis points. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 2 | `InvalidFeeRate` | `new_fee_rate_bps == 0` or `> 10000` |

---

### 2.2 Storage Layout

| Key | Type | Storage | Description |
|---|---|---|---|
| `FeeConfig` | `FeeConfig` | Instance | Protocol fee rate, treasury share bps, and admin council. |
| `TreasuryAddress` | `Address` | Instance | Address of the Treasury contract receiving the protocol share. |
| `TokenAddress` | `Address` | Instance | SAC token address used for fee transfers. |
| `FeeEntry(u64)` | `FeeEntry` | Persistent | Distribution record keyed by `batch_id`. Written once, never updated. |
| `Earnings(Address)` | `EarningsRecord` | Persistent | Cumulative earnings keyed by relay node address. Updated on every `distribute` and `claim`. |

---

### 2.3 Event Schemas

#### `fee_distributor.distribute`
| Field | Value |
|---|---|
| **Topics** | `("fee_distributor", "distribute")` |
| **Data** | `(relay_address: Address, batch_id: u64, relay_payout: i128, treasury_share: i128)` |
| **Trigger** | Successful `distribute` call |

#### `fee_distributor.claim`
| Field | Value |
|---|---|
| **Topics** | `("fee_distributor", "claim")` |
| **Data** | `(relay_address: Address, payout: i128)` |
| **Trigger** | Successful `claim` call |

#### `fee_distributor.set_fee_rate`
| Field | Value |
|---|---|
| **Topics** | `("fee_distributor", "set_fee_rate")` |
| **Data** | `(new_fee_rate_bps: u32,)` |
| **Trigger** | Successful `set_fee_rate` call |

---

## 3. Dispute Resolver

Handles final on-chain arbitration for double-spend conflicts. Parties submit
cryptographic relay chain proofs; the contract evaluates them deterministically
and issues a binding ruling.

### 3.1 Public Functions (ABI)

#### `initialize`

| Field | Value |
|---|---|
| **Parameters** | `council: AdminCouncil`, `resolution_window: u32` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | None (first-caller-wins, one-time) |
| **Description** | One-time setup. Sets the admin council and the resolution window (number of ledgers a respondent has to submit a counter-proof). |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 14 | `AlreadyInitialized` | Contract already initialized |
| 15 | `InvalidConfig` | `resolution_window == 0` |
| 18 | `InvalidCouncilConfig` | `threshold == 0` or `members.len() < threshold` |

---

#### `raise_dispute`

| Field | Value |
|---|---|
| **Parameters** | `initiator: Address`, `tx_id: BytesN<32>`, `proof: RelayChainProof` |
| **Returns** | `Result<u64, ContractError>` |
| **Auth Required** | `initiator` |
| **Description** | Submits a new dispute for a suspected double-spend. Records the initiator's relay chain proof on-chain, sets a `resolve_by` deadline, and returns the newly assigned `dispute_id`. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 8 | `DuplicateDispute` | A dispute for this `tx_id` already exists |

---

#### `respond`

| Field | Value |
|---|---|
| **Parameters** | `respondent: Address`, `dispute_id: u64`, `proof: RelayChainProof` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | `respondent` |
| **Description** | Submits a counter-proof to an open dispute. Transitions the dispute from `Open` to `Responded`. Must be called before the `resolve_by` deadline. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `DisputeNotFound` | No dispute for this ID |
| 10 | `NotOpen` | Dispute is not in `Open` status |
| 11 | `ResolutionWindowExpired` | Response deadline has passed |

---

#### `resolve`

| Field | Value |
|---|---|
| **Parameters** | `dispute_id: u64` |
| **Returns** | `Result<Ruling, ContractError>` |
| **Auth Required** | None (permissionless after conditions are met) |
| **Description** | Evaluates both proofs and issues a final `Ruling`. Can be called by anyone once the dispute is `Responded`, or after the resolution window expires (initiator wins by default if no response). Proof evaluation uses Ed25519 signature verification with sequence-number tiebreaking when both proofs are valid. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `DisputeNotFound` | No dispute for this ID |
| 2 | `DisputeAlreadyResolved` | Dispute already has a ruling |
| 12 | `ResolutionWindowActive` | Dispute is still `Open` and deadline has not passed |
| 13 | `NotResponded` | Dispute is in an unexpected state |
| 16 | `InvalidProofSignature` | Both proofs failed Ed25519 verification |

---

#### `get_dispute`

| Field | Value |
|---|---|
| **Parameters** | `dispute_id: u64` |
| **Returns** | `Result<Dispute, ContractError>` |
| **Auth Required** | None (view) |
| **Description** | Returns the full `Dispute` record including current status. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `DisputeNotFound` | No dispute for this ID |

---

#### `get_ruling`

| Field | Value |
|---|---|
| **Parameters** | `dispute_id: u64` |
| **Returns** | `Result<Ruling, ContractError>` |
| **Auth Required** | None (view) |
| **Description** | Returns the final `Ruling` for a resolved dispute. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `DisputeNotFound` | No ruling found for this ID |

---

### 3.2 Storage Layout

| Key | Type | Storage | Description |
|---|---|---|---|
| `Dispute(u64)` | `Dispute` | Persistent | Full dispute record keyed by `dispute_id`. Updated on `respond` and `resolve`. |
| `Ruling(u64)` | `Ruling` | Persistent | Final ruling keyed by `dispute_id`. Written once on `resolve`. |
| `DisputeCount` | `u64` | Instance | Monotonically incrementing dispute ID counter. |
| `ResolutionWindow` | `u32` | Instance | Number of ledgers a respondent has to submit a counter-proof. |
| `AdminCouncil` | `AdminCouncil` | Instance | M-of-N council authorized to configure the contract. |
| `TxDispute(BytesN<32>)` | `u64` | Persistent | Maps a `tx_id` to its `dispute_id`. Used to prevent duplicate disputes. |
| `PublicKey(Address)` | `BytesN<32>` | Persistent | Raw 32-byte Ed25519 public key for an address. Must be registered before `raise_dispute` or `respond`. |

---

### 3.3 Event Schemas

#### `dispute_resolver.raise`
| Field | Value |
|---|---|
| **Topics** | `("dispute_resolver", "raise")` |
| **Data** | `(initiator: Address, dispute_id: u64, tx_id: BytesN<32>)` |
| **Trigger** | Successful `raise_dispute` call |

#### `dispute_resolver.respond`
| Field | Value |
|---|---|
| **Topics** | `("dispute_resolver", "respond")` |
| **Data** | `(respondent: Address, dispute_id: u64)` |
| **Trigger** | Successful `respond` call |

#### `dispute_resolver.resolve`
| Field | Value |
|---|---|
| **Topics** | `("dispute_resolver", "resolve")` |
| **Data** | `(dispute_id: u64, winner: Address, loser: Address)` |
| **Trigger** | Successful `resolve` call (both timeout and proof-evaluated paths) |

---

## 4. Treasury

Holds protocol funds and disburses them via spending programs. All
withdrawals and allocations require admin council authorization.

### 4.1 Public Functions (ABI)

#### `initialize`

| Field | Value |
|---|---|
| **Parameters** | `council: AdminCouncil`, `token_address: Address` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | None (first-caller-wins, one-time) |
| **Description** | One-time setup. Sets the admin council and SAC token address. Initializes balance to zero. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 10 | `AlreadyInitialized` | Contract already initialized |
| 14 | `InvalidCouncilConfig` | `threshold == 0` or `members.len() < threshold` |

---

#### `deposit`

| Field | Value |
|---|---|
| **Parameters** | `from: Address`, `amount: i128` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | `from` |
| **Description** | Transfers `amount` tokens from `from` to the treasury. Updates the balance and lifetime stats. Records a `TreasuryEntry` of kind `Deposit`. Also called automatically by the Fee Distributor via cross-contract invocation. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 3 | `InvalidAmount` | `amount ≤ 0` |
| 9 | `Overflow` | Balance arithmetic overflow |

---

#### `withdraw`

| Field | Value |
|---|---|
| **Parameters** | `to: Address`, `amount: i128`, `memo: String` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | Admin council (M-of-N threshold) |
| **Description** | Transfers `amount` tokens from the treasury to `to`. Records a `TreasuryEntry` of kind `Withdrawal`. Updates lifetime stats. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `InsufficientBalance` | Treasury balance is below `amount` |
| 3 | `InvalidAmount` | `amount ≤ 0` |
| 9 | `Overflow` | Arithmetic overflow |

---

#### `create_program`

| Field | Value |
|---|---|
| **Parameters** | `name: String`, `budget: i128` |
| **Returns** | `Result<u64, ContractError>` |
| **Auth Required** | Admin council (M-of-N threshold) |
| **Description** | Creates a new spending program with a defined budget. Returns the new `program_id`. Program names must be 3–64 characters. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 3 | `InvalidAmount` | `budget ≤ 0` |
| 15 | `InvalidProgramName` | Name length < 3 or > 64 |

---

#### `update_program_budget`

| Field | Value |
|---|---|
| **Parameters** | `program_id: u64`, `new_budget: i128` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | Admin council (M-of-N threshold) |
| **Description** | Updates the budget of an existing spending program. New budget must not be less than the amount already spent. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 3 | `InvalidAmount` | `new_budget < program.spent` |
| 4 | `ProgramNotFound` | Program ID does not exist |

---

#### `deactivate_program`

| Field | Value |
|---|---|
| **Parameters** | `program_id: u64` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | Admin council (M-of-N threshold) |
| **Description** | Marks a spending program as inactive. Inactive programs cannot receive further allocations. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 4 | `ProgramNotFound` | Program ID does not exist |

---

#### `allocate`

| Field | Value |
|---|---|
| **Parameters** | `program_id: u64`, `amount: i128` |
| **Returns** | `Result<(), ContractError>` |
| **Auth Required** | Admin council (M-of-N threshold) |
| **Description** | Allocates `amount` tokens from the treasury balance to a spending program. Deducts from treasury balance and increments `program.spent`. Records a `TreasuryEntry` of kind `Allocation`. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 1 | `InsufficientBalance` | Treasury balance is below `amount` |
| 3 | `InvalidAmount` | `amount ≤ 0` |
| 4 | `ProgramNotFound` | Program ID does not exist |
| 9 | `Overflow` | Arithmetic overflow |
| 11 | `ProgramInactive` | Program is not active |
| 12 | `ProgramOverBudget` | Allocation would exceed program budget |

---

#### `get_program`

| Field | Value |
|---|---|
| **Parameters** | `program_id: u64` |
| **Returns** | `Result<SpendingProgram, ContractError>` |
| **Auth Required** | None (view) |
| **Description** | Returns the `SpendingProgram` details for the given ID. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 4 | `ProgramNotFound` | Program ID does not exist |

---

#### `get_balance`

| Field | Value |
|---|---|
| **Parameters** | None |
| **Returns** | `i128` |
| **Auth Required** | None (view) |
| **Description** | Returns the current treasury token balance. Returns `0` if uninitialized. Never errors. |

---

#### `get_history`

| Field | Value |
|---|---|
| **Parameters** | `entry_id: u64` |
| **Returns** | `Result<TreasuryEntry, ContractError>` |
| **Auth Required** | None (view) |
| **Description** | Returns a specific history entry by ID. Entry IDs are 1-indexed and increment with each deposit, withdrawal, or allocation. |

**Errors**

| Code | Name | Condition |
|---|---|---|
| 4 | `ProgramNotFound` | Entry ID does not exist |

---

#### `get_treasury_stats`

| Field | Value |
|---|---|
| **Parameters** | None |
| **Returns** | `TreasuryStats` |
| **Auth Required** | None (view) |
| **Description** | Returns aggregate lifetime statistics: current balance, total deposited, total withdrawn, and total allocated. Intended for dashboard integration. Never errors. |

---

### 4.2 Storage Layout

| Key | Type | Storage | Description |
|---|---|---|---|
| `Balance` | `i128` | Instance | Current treasury token balance. Updated on every deposit, withdrawal, and allocation. |
| `EntryCount` | `u64` | Instance | Total number of recorded treasury entries. Auto-increments. |
| `Entry(u64)` | `TreasuryEntry` | Persistent | A single treasury history record keyed by entry ID (1-indexed). Written once, never updated. |
| `ProgramCount` | `u64` | Instance | Total number of created spending programs. Auto-increments. |
| `SpendingProgram(u64)` | `SpendingProgram` | Persistent | Spending program keyed by `program_id`. Updated on `allocate`, `update_program_budget`, and `deactivate_program`. |
| `Allocation(String)` | `AllocationRecord` | Persistent | Allocation record keyed by program name string. |
| `AdminCouncil` | `AdminCouncil` | Instance | M-of-N council authorized to withdraw and allocate. |
| `TokenAddress` | `Address` | Instance | SAC token address used for all transfers. |
| `Stats` | `TreasuryStats` | Instance | Aggregate lifetime stats (deposited, withdrawn, allocated). Updated on every state-changing call. |

---

### 4.3 Event Schemas

#### `treasury.deposit`
| Field | Value |
|---|---|
| **Topics** | `("treasury", "deposit")` |
| **Data** | `(from: Address, amount: i128)` |
| **Trigger** | Successful `deposit` call (direct or via cross-contract from Fee Distributor) |

#### `treasury.withdraw`
| Field | Value |
|---|---|
| **Topics** | `("treasury", "withdraw")` |
| **Data** | `(to: Address, amount: i128, memo: String)` |
| **Trigger** | Successful `withdraw` call |

#### `treasury.create_program`
| Field | Value |
|---|---|
| **Topics** | `("treasury", "create_program")` |
| **Data** | `(program_id: u64, name: String, budget: i128)` |
| **Trigger** | Successful `create_program` call |

#### `treasury.update_budget`
| Field | Value |
|---|---|
| **Topics** | `("treasury", "update_budget")` |
| **Data** | `(program_id: u64, new_budget: i128)` |
| **Trigger** | Successful `update_program_budget` call |

#### `treasury.deactivate_program`
| Field | Value |
|---|---|
| **Topics** | `("treasury", "deactivate_program")` |
| **Data** | `(program_id: u64,)` |
| **Trigger** | Successful `deactivate_program` call |

#### `treasury.allocate`
| Field | Value |
|---|---|
| **Topics** | `("treasury", "allocate")` |
| **Data** | `(program_id: u64, amount: i128)` |
| **Trigger** | Successful `allocate` call |

---

## 5. Custom Types Dictionary

All types are annotated with `#[contracttype]` and are serializable by the
Soroban SDK. They are shared across contracts where noted.

---

### `AdminCouncil`
> Used by: all four contracts

| Field | Type | Description |
|---|---|---|
| `members` | `Vec<Address>` | List of council member addresses (max 10) |
| `threshold` | `u32` | Minimum number of member signatures required to authorize a sensitive action |

---

### `NodeMetadata`
> Used by: Relay Registry

| Field | Type | Description |
|---|---|---|
| `region` | `String` | Geographic region of the relay node (max 32 chars) |
| `capacity` | `u32` | Maximum transactions the node can handle per batch |
| `uptime_commitment` | `u32` | Promised uptime percentage (0–100) |

---

### `NodeStatus`
> Used by: Relay Registry

| Variant | Description |
|---|---|
| `Active` | Node is staked above minimum and can participate in the network |
| `Inactive` | Node is registered but stake is below minimum |
| `Slashed` | Node has been penalized; all operations blocked |

---

### `RelayNode`
> Used by: Relay Registry

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | Stellar account address of the relay node |
| `stake` | `i128` | Current staked token amount |
| `status` | `NodeStatus` | Current operational status |
| `metadata` | `NodeMetadata` | Operational characteristics |
| `registered_at` | `u64` | Ledger timestamp of registration |
| `last_active` | `u64` | Ledger timestamp of last activity |

---

### `StakeEntry`
> Used by: Relay Registry

| Field | Type | Description |
|---|---|---|
| `address` | `Address` | Node that initiated the unstake |
| `amount` | `i128` | Amount of tokens pending withdrawal |
| `unlocks_at` | `u64` | Ledger timestamp after which `finalize_unstake` can be called |

---

### `FeeEntry`
> Used by: Fee Distributor

| Field | Type | Description |
|---|---|---|
| `batch_id` | `u64` | Unique identifier of the settled transaction batch |
| `relay_address` | `Address` | Relay node that settled the batch |
| `amount` | `i128` | Total fee distributed for this batch |
| `treasury_share` | `i128` | Portion sent to the protocol treasury |
| `settled_at` | `u64` | Ledger timestamp of distribution |

---

### `EarningsRecord`
> Used by: Fee Distributor

| Field | Type | Description |
|---|---|---|
| `total_earned` | `i128` | Lifetime total fees earned |
| `total_claimed` | `i128` | Lifetime total fees claimed and paid out |
| `unclaimed` | `i128` | Current claimable balance (`total_earned - total_claimed`) |

---

### `FeeConfig`
> Used by: Fee Distributor

| Field | Type | Description |
|---|---|---|
| `fee_rate_bps` | `u32` | Fee rate in basis points (1–10000) |
| `treasury_share_bps` | `u32` | Treasury's cut of each distribution in basis points |
| `council` | `AdminCouncil` | Council authorized to call `set_fee_rate` |

---

### `DisputeStatus`
> Used by: Dispute Resolver

| Variant | Description |
|---|---|
| `Open` | Dispute raised, awaiting counter-proof |
| `Responded` | Counter-proof submitted, awaiting `resolve` call |
| `Resolved` | Final ruling has been issued |
| `Expired` | Resolution window passed without a response |

---

### `RelayChainProof`
> Used by: Dispute Resolver

| Field | Type | Description |
|---|---|---|
| `signature` | `BytesN<64>` | Ed25519 signature of the relay chain hash |
| `chain_hash` | `BytesN<32>` | Hash of the relay chain at the point of signing |
| `sequence` | `u64` | Sequence number in the relay chain at signing (lower wins tiebreak) |

---

### `Dispute`
> Used by: Dispute Resolver

| Field | Type | Description |
|---|---|---|
| `dispute_id` | `u64` | Unique monotonic identifier |
| `tx_id` | `BytesN<32>` | The Stellar transaction ID under dispute |
| `initiator` | `Address` | Party that raised the dispute |
| `respondent` | `Option<Address>` | Counter-party; set when they respond |
| `initiator_proof` | `RelayChainProof` | Initiator's cryptographic proof |
| `respondent_proof` | `OptionalRelayChainProof` | Respondent's proof; `None` until `respond` is called |
| `status` | `DisputeStatus` | Current lifecycle status |
| `raised_at` | `u64` | Ledger timestamp of dispute submission |
| `resolve_by` | `u64` | Ledger sequence number deadline for response |

---

### `Ruling`
> Used by: Dispute Resolver

| Field | Type | Description |
|---|---|---|
| `dispute_id` | `u64` | The dispute this ruling belongs to |
| `winner` | `Address` | Address that won the dispute |
| `loser` | `Address` | Address that lost and will be penalized |
| `reason` | `String` | Human-readable explanation of the ruling |
| `resolved_at` | `u64` | Ledger timestamp when the ruling was issued |

---

### `TreasuryEntry`
> Used by: Treasury

| Field | Type | Description |
|---|---|---|
| `kind` | `EntryKind` | Type of transaction |
| `amount` | `i128` | Token amount |
| `actor` | `Address` | Address that initiated the transaction |
| `recipient` | `Option<Address>` | Recipient for withdrawals; `None` for deposits and allocations |
| `memo` | `String` | Human-readable reason or memo |
| `ledger` | `u64` | Ledger sequence number when the entry occurred |

---

### `EntryKind`
> Used by: Treasury

| Variant | Description |
|---|---|
| `Deposit` | Funds deposited into the treasury |
| `Withdrawal` | Funds withdrawn from the treasury to a recipient |
| `Allocation` | Funds allocated to a spending program |

---

### `SpendingProgram`
> Used by: Treasury

| Field | Type | Description |
|---|---|---|
| `program_id` | `u64` | Unique program ID (auto-assigned) |
| `budget` | `i128` | Total budget allocated to this program |
| `spent` | `i128` | Amount already allocated from this program |
| `active` | `bool` | Whether the program is currently accepting allocations |
| `name` | `String` | Human-readable name/description (3–64 chars) |

---

### `TreasuryStats`
> Used by: Treasury

| Field | Type | Description |
|---|---|---|
| `current_balance` | `i128` | Current treasury token balance |
| `lifetime_deposited` | `i128` | Total tokens deposited over the treasury's lifetime |
| `lifetime_withdrawn` | `i128` | Total tokens withdrawn over the treasury's lifetime |
| `lifetime_allocated` | `i128` | Total tokens allocated to spending programs |

---

### `AllocationRecord`
> Used by: Treasury

| Field | Type | Description |
|---|---|---|
| `program` | `String` | Name of the spending program |
| `allocated` | `i128` | Total tokens allocated to the program |
| `spent` | `i128` | Tokens already spent from this allocation |

---

*Last updated: reflects `main` branch as of this PR. All ABI and storage layouts
are final per the issue prerequisite that all feature PRs (#40–#48) are merged.*
