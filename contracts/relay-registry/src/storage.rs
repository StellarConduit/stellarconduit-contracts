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
//! - `DataKey::Admin` — Address authorized to slash nodes and update config
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

#![allow(unused)]
use soroban_sdk::{contracttype, Address, Env};

use crate::types::RelayNode;

// Bump by ~30 days (assuming ~5 seconds per ledger)
const LEDGER_BUMP_AMOUNT: u32 = 518_400;
// Bump if remaining life is less than ~15 days
const LEDGER_BUMP_THRESHOLD: u32 = 259_200;

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    RelayNode(Address),
    NodeCount,
    MinStake,
    StakeLockPeriod,
    AdminCouncil,
    TokenAddress,
    LockEntry(Address),
}

pub fn get_node(env: &Env, address: &Address) -> Option<RelayNode> {
    let key = DataKey::RelayNode(address.clone());
    if let Some(node) = env.storage().persistent().get::<_, RelayNode>(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
        Some(node)
    } else {
        None
    }
}

pub fn set_node(env: &Env, address: &Address, node: &RelayNode) {
    let key = DataKey::RelayNode(address.clone());
    env.storage()
        .persistent()
        .set(&key, node);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

pub fn remove_node(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::RelayNode(address.clone()));
}

pub fn get_node_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::NodeCount)
        .unwrap_or(0)
}

pub fn set_node_count(env: &Env, count: u32) {
    env.storage().instance().set(&DataKey::NodeCount, &count);
}

pub fn increment_node_count(env: &Env) {
    let next = get_node_count(env)
        .checked_add(1)
        .expect("node count overflow");
    set_node_count(env, next);
}

pub fn get_min_stake(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::MinStake)
        .unwrap_or(0)
}

pub fn set_min_stake(env: &Env, min_stake: i128) {
    env.storage().instance().set(&DataKey::MinStake, &min_stake);
}

pub fn get_stake_lock_period(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::StakeLockPeriod)
        .unwrap_or(0)
}

pub fn set_stake_lock_period(env: &Env, period: u32) {
    env.storage()
        .instance()
        .set(&DataKey::StakeLockPeriod, &period);
}

pub fn get_admin_council(env: &Env) -> crate::types::AdminCouncil {
    env.storage()
        .instance()
        .get(&DataKey::AdminCouncil)
        .expect("admin council not initialized")
}

pub fn set_admin_council(env: &Env, council: &crate::types::AdminCouncil) {
    env.storage()
        .instance()
        .set(&DataKey::AdminCouncil, council);
}

pub fn get_token_address(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::TokenAddress)
        .expect("token address not initialized")
}

pub fn set_token_address(env: &Env, token_address: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::TokenAddress, token_address);
}

pub fn get_lock_entry(env: &Env, address: &Address) -> Option<crate::types::StakeEntry> {
    let key = DataKey::LockEntry(address.clone());
    if let Some(entry) = env
        .storage()
        .persistent()
        .get::<_, crate::types::StakeEntry>(&key)
    {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
        Some(entry)
    } else {
        None
    }
}

pub fn set_lock_entry(env: &Env, address: &Address, entry: &crate::types::StakeEntry) {
    let key = DataKey::LockEntry(address.clone());
    env.storage()
        .persistent()
        .set(&key, entry);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

pub fn remove_lock_entry(env: &Env, address: &Address) {
    env.storage()
        .persistent()
        .remove(&DataKey::LockEntry(address.clone()));
}
