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

#![allow(unused)]

use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone)]
pub struct RelayNode {
    pub address: Address,
    pub stake: i128,
    pub status: NodeStatus,
    pub metadata: NodeMetadata,
    pub registered_at: u64,
    pub last_active: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct NodeMetadata {
    pub region: String,
    pub capacity: u32,
    pub uptime_commitment: u32,
}

#[contracttype]
#[derive(Clone)]
pub enum NodeStatus {
    Active,
    Inactive,
    Slashed,
}

#[contracttype]
#[derive(Clone)]
pub struct StakeEntry {
    pub node: Address,
    pub amount: i128,
    pub unlock_ledger: u32,
}
