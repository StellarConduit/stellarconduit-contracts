//! # Relay Registry Contract — `types.rs`
//!
//! Defines all data structures used by the Relay Registry contract.
//!
//! ## Types to implement
//! - `RelayNode` — The primary struct representing a registered relay node, including:
//!   - `address: Address` — The Stellar account address of the relay node
//!   - `stake: i128` — Current staked token amount
//!   - `status: NodeStatus` — Active, Inactive, or Slashed
//!   - `metadata: NodeMetadata` — Region, capacity, uptime commitment
//!   - `registered_at: u64` — Ledger timestamp of registration
//!   - `last_active: u64` — Ledger timestamp of last activity
//! - `NodeMetadata` — Supplementary metadata:
//!   - `region: String` — Geographic region of the relay node
//!   - `capacity: u32` — Maximum transactions per batch
//!   - `uptime_commitment: u32` — Percentage uptime commitment (0–100)
//! - `NodeStatus` — Enum with variants: `Active`, `Inactive`, `Slashed`
//! - `StakeEntry` — Represents a pending unstake operation with unlock ledger
//!
//! implementation tracked in GitHub issue

use soroban_sdk::{contracttype, Address, String};

/// Represents the operational status of a relay node within the protocol.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    /// Node is active and can participate in the relay network.
    Active,
    /// Node is inactive and cannot participate in the relay network.
    Inactive,
    /// Node has been slashed due to misbehavior and cannot participate.
    Slashed,
}

/// Metadata associated with a relay node, describing its operational characteristics.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeMetadata {
    /// Geographic region where the relay node is located.
    pub region: String,
    /// Maximum number of transactions the node can handle per batch.
    pub capacity: u32,
    /// Uptime commitment percentage (0-100) that the node promises to maintain.
    pub uptime_commitment: u32,
}

/// Represents a registered relay node in the protocol, including its stake, status, and metadata.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayNode {
    /// The Stellar account address of the relay node.
    pub address: Address,
    /// Current amount of tokens staked by this node.
    pub stake: i128,
    /// Current operational status of the node.
    pub status: NodeStatus,
    /// Metadata describing the node's operational characteristics.
    pub metadata: NodeMetadata,
    /// Ledger timestamp when the node was first registered.
    pub registered_at: u64,
    /// Ledger timestamp of the node's last recorded activity.
    pub last_active: u64,
}

/// Represents a pending unstake operation that is subject to a lock period.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StakeEntry {
    /// The Stellar account address that initiated the unstake operation.
    pub address: Address,
    /// Ledger number when the unstaked tokens can be withdrawn.
    pub unlocks_at: u64,
}
