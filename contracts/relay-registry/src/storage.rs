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

#![allow(unused)]

use soroban_sdk::{contracttype, Address, Env};

// implementation tracked in GitHub issue
