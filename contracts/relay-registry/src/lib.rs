//! # Relay Registry Contract — `lib.rs`
//!
//! This is the main entry point for the Relay Registry Soroban smart contract.
//! It exposes the public contract interface and wires together the types, storage,
//! and error modules.
//!
//! ## Responsibilities
//! - Relay node registration on-chain (`register`)
//! - Token staking and unstaking with lock period enforcement (`stake`, `unstake`)
//! - Stake slashing for misbehaving relay nodes (`slash`)
//! - Node lookup and active-status verification (`get_node`, `is_active`)
//!
//! ## Functions to implement
//! - `register(env, node_address, metadata)` — Register a new relay node and verify minimum stake
//! - `stake(env, amount)` — Deposit stake tokens into the registry
//! - `unstake(env, amount)` — Initiate stake withdrawal, subject to lock period
//! - `slash(env, node_address, reason)` — Slash a misbehaving relay node's stake
//! - `get_node(env, address)` — Fetch relay node details and metadata
//! - `is_active(env, address)` — Check if a relay node is currently in active status
//!
//! ## See also
//! - `types.rs` — Data structures (RelayNode, NodeMetadata, NodeStatus)
//! - `storage.rs` — Persistent storage helpers
//! - `errors.rs` — Contract error codes
//!
//! implementation tracked in GitHub issue

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env};

pub mod errors;
pub mod storage;
pub mod types;

use crate::errors::ContractError;
use crate::types::{NodeStatus, StakeEntry};

#[contract]
pub struct RelayRegistryContract;

#[contractimpl]
impl RelayRegistryContract {
    /// Initiates unstaking for a registered relay node and records a lock period.
    ///
    /// # Parameters
    /// - `env`: Soroban execution environment.
    /// - `node_address`: Address of the relay node requesting unstake.
    /// - `amount`: Amount of stake to unlock.
    ///
    /// # Errors
    /// - [`ContractError::NotRegistered`] if the node is not registered.
    /// - [`ContractError::NodeSlashed`] if the node has been slashed.
    /// - [`ContractError::InsufficientStake`] if `amount <= 0` or exceeds current stake.
    /// - [`ContractError::Overflow`] if arithmetic overflows/underflows.
    pub fn unstake(env: Env, node_address: Address, amount: i128) -> Result<(), ContractError> {
        node_address.require_auth();

        let mut node =
            storage::get_node(&env, &node_address).ok_or(ContractError::NotRegistered)?;

        if node.status == NodeStatus::Slashed {
            return Err(ContractError::NodeSlashed);
        }

        if amount <= 0 || amount > node.stake {
            return Err(ContractError::InsufficientStake);
        }

        let lock_period = storage::get_stake_lock_period(&env);
        let unlock_ledger = env
            .ledger()
            .sequence()
            .checked_add(lock_period)
            .ok_or(ContractError::Overflow)?;
        let entry = StakeEntry {
            address: node_address.clone(),
            unlocks_at: unlock_ledger as u64,
        };
        storage::set_pending_unstake(&env, &node_address, &entry);

        let new_stake = node
            .stake
            .checked_sub(amount)
            .ok_or(ContractError::Overflow)?;

        let min_stake = storage::get_min_stake(&env);
        if new_stake < min_stake {
            node.status = NodeStatus::Inactive;
        }

        node.stake = new_stake;
        storage::set_node(&env, &node_address, &node);

        // TODO: finalize_unstake()
        Ok(())
    }
}
