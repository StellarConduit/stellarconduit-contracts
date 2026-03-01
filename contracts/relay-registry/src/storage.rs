//! # Relay Registry Contract — `storage.rs`
//!
//! Provides typed helper functions for reading and writing persistent contract
//! storage using Soroban's `Env::storage()` API.
//!
//! ## Storage keys to implement
//! - `DataKey::RelayNode(Address)` — Stores a `RelayNode` struct keyed by address
//! - `DataKey::NodeCount` — Tracks total number of registered relay nodes
//! - `DataKey::MinStake` — Minimum required stake amount (set at initialization)
//! - `DataKey::StakeLockPeriod` — Number of ledgers a node must wait before unstaking
//!
//! ## Functions to implement
//! - `get_node(env, address) -> Option<RelayNode>` — Load a relay node from storage
//! - `set_node(env, address, node)` — Persist a relay node to storage
//! - `remove_node(env, address)` — Remove a relay node from storage
//! - `get_node_count(env) -> u32` — Get the total number of registered nodes
//! - `increment_node_count(env)` — Increment the node count by 1
//! - `get_min_stake(env) -> i128` — Load the minimum stake requirement
//! - `get_stake_lock_period(env) -> u32` — Load the stake lock period in ledgers
//!
//! implementation tracked in GitHub issue

use soroban_sdk::{contracttype, Address, Env};

use crate::types::{RelayNode, StakeEntry};

/// Storage keys used by the relay registry contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Registered relay node data keyed by address.
    RelayNode(Address),
    /// Total number of registered relay nodes.
    NodeCount,
    /// Protocol minimum stake required for active participation.
    MinStake,
    /// Number of ledgers to wait before pending unstake can be finalized.
    StakeLockPeriod,
    /// Pending unstake entry keyed by node address.
    PendingUnstake(Address),
}

/// Returns the relay node for `address`, if one exists.
pub fn get_node(env: &Env, address: &Address) -> Option<RelayNode> {
    let key = DataKey::RelayNode(address.clone());
    env.storage().persistent().get(&key)
}

/// Persists relay `node` for `address`.
pub fn set_node(env: &Env, address: &Address, node: &RelayNode) {
    let key = DataKey::RelayNode(address.clone());
    env.storage().persistent().set(&key, node);
}

/// Removes any relay node stored for `address`.
pub fn remove_node(env: &Env, address: &Address) {
    let key = DataKey::RelayNode(address.clone());
    env.storage().persistent().remove(&key);
}

/// Returns the total number of registered nodes.
pub fn get_node_count(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::NodeCount)
        .unwrap_or(0)
}

/// Increments and persists the total node count by one.
pub fn increment_node_count(env: &Env) {
    let next = get_node_count(env).saturating_add(1);
    env.storage().persistent().set(&DataKey::NodeCount, &next);
}

/// Returns the configured minimum stake. Defaults to `0` if unset.
pub fn get_min_stake(env: &Env) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::MinStake)
        .unwrap_or(0)
}

/// Persists the minimum stake value.
pub fn set_min_stake(env: &Env, min_stake: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::MinStake, &min_stake);
}

/// Returns the configured stake lock period (in ledgers). Defaults to `0` if unset.
pub fn get_stake_lock_period(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::StakeLockPeriod)
        .unwrap_or(0)
}

/// Persists the stake lock period in ledgers.
pub fn set_stake_lock_period(env: &Env, lock_period: u32) {
    env.storage()
        .persistent()
        .set(&DataKey::StakeLockPeriod, &lock_period);
}

/// Returns the pending unstake entry for `address`, if present.
pub fn get_pending_unstake(env: &Env, address: &Address) -> Option<StakeEntry> {
    let key = DataKey::PendingUnstake(address.clone());
    env.storage().persistent().get(&key)
}

/// Stores (or overwrites) the pending unstake `entry` for `address`.
pub fn set_pending_unstake(env: &Env, address: &Address, entry: &StakeEntry) {
    let key = DataKey::PendingUnstake(address.clone());
    env.storage().persistent().set(&key, entry);
}
