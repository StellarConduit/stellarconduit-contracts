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
//! ## Functions
//! - `register(env, node_address, metadata)` — Register a new relay node with metadata
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
use soroban_sdk::{contract, contractimpl, Address, Env, String};

pub mod errors;
pub mod storage;
pub mod types;

use crate::errors::ContractError;
use crate::types::{NodeMetadata, NodeStatus, RelayNode};

#[contract]
pub struct RelayRegistryContract;

#[contractimpl]
impl RelayRegistryContract {
    /// Initialize the contract with admin address, minimum stake, and stake lock period.
    ///
    /// This is a one-time setup function called immediately after the contract is deployed.
    /// It sets the admin address, minimum stake requirement, and stake lock period.
    /// It can only be called once.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `admin`: Address of the admin account authorized to slash nodes and update config.
    /// - `min_stake`: Minimum required stake amount. Must be greater than zero.
    /// - `stake_lock_period`: Number of ledgers a node must wait before unstaking. Must be greater than zero.
    ///
    /// # Errors
    /// - `ContractError::AlreadyInitialized` if the contract has already been initialized.
    /// - `ContractError::InvalidAmount` if `min_stake` is zero or negative, or if `stake_lock_period` is zero.
    pub fn initialize(
        env: Env,
        admin: Address,
        min_stake: i128,
        stake_lock_period: u32,
    ) -> Result<(), ContractError> {
        // Guard against re-initialization
        if env.storage().instance().has(&storage::DataKey::Admin) {
            return Err(ContractError::AlreadyInitialized);
        }

        // Validate inputs
        if min_stake <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if stake_lock_period == 0 {
            return Err(ContractError::InvalidAmount);
        }

        // Persist config
        storage::set_admin(&env, &admin);
        storage::set_min_stake(&env, min_stake);
        storage::set_stake_lock_period(&env, stake_lock_period);

        // Initialize node count
        storage::set_node_count(&env, 0);

        Ok(())
    }

    /// Register a new relay node with the given address and metadata.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `node_address`: Stellar account address of the relay node. Must authorize this call.
    /// - `metadata`: Metadata describing the relay node's region, capacity, and uptime commitment.
    ///
    /// # Errors
    /// - `ContractError::AlreadyRegistered` if a node with this address already exists.
    /// - `ContractError::InvalidMetadata` if `metadata.uptime_commitment` is greater than 100.
    pub fn register(
        env: Env,
        node_address: Address,
        metadata: NodeMetadata,
    ) -> Result<(), ContractError> {
        node_address.require_auth();

        if storage::get_node(&env, &node_address).is_some() {
            return Err(ContractError::AlreadyRegistered);
        }

        if metadata.uptime_commitment > 100 {
            return Err(ContractError::InvalidMetadata);
        }

        let timestamp = env.ledger().timestamp();

        let node = RelayNode {
            address: node_address.clone(),
            stake: 0,
            status: NodeStatus::Inactive,
            metadata,
            registered_at: timestamp,
            last_active: timestamp,
        };

        storage::set_node(&env, &node_address, &node);
        storage::increment_node_count(&env);
        Ok(())
    }

    /// Deposit stake tokens on-chain for a registered relay node.
    ///
    /// This function allows a registered relay node to deposit stake tokens on-chain.
    /// Once the node's total stake reaches the protocol minimum, its status is
    /// automatically promoted to Active.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `node_address`: Stellar account address of the relay node. Must authorize this call.
    /// - `amount`: Amount of tokens to stake. Must be greater than zero.
    ///
    /// # Errors
    /// - `ContractError::NotRegistered` if the node is not found in the registry.
    /// - `ContractError::NodeSlashed` if the node has been slashed.
    /// - `ContractError::InsufficientStake` if the `amount` is zero or negative.
    /// - `ContractError::Overflow` if adding the stake causes an arithmetic overflow.
    pub fn stake(env: Env, node_address: Address, amount: i128) -> Result<(), ContractError> {
        node_address.require_auth();

        let mut node =
            storage::get_node(&env, &node_address).ok_or(ContractError::NotRegistered)?;

        if matches!(node.status, NodeStatus::Slashed) {
            return Err(ContractError::NodeSlashed);
        }

        if amount <= 0 {
            return Err(ContractError::InsufficientStake);
        }

        let new_stake = node
            .stake
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;

        let min_stake = storage::get_min_stake(&env);
        if new_stake >= min_stake {
            node.status = NodeStatus::Active;
        }

        node.last_active = env.ledger().timestamp();
        node.stake = new_stake;

        // TODO: SAC transfer
        storage::set_node(&env, &node_address, &node);

        Ok(())
    }

    pub fn unstake(
        env: Env,
        node_address: Address,
        amount: i128,
    ) -> Result<RelayNode, ContractError> {
        node_address.require_auth();
        if amount <= 0 {
            return Err(ContractError::InsufficientStake);
        }

        let mut node =
            storage::get_node(&env, &node_address).ok_or(ContractError::NotRegistered)?;
        if matches!(node.status, NodeStatus::Slashed) {
            return Err(ContractError::NodeSlashed);
        }
        if !matches!(node.status, NodeStatus::Active) {
            return Err(ContractError::NodeNotActive);
        }

        let unlock_after = node
            .registered_at
            .checked_add(storage::get_stake_lock_period(&env) as u64)
            .ok_or(ContractError::Overflow)?;
        if env.ledger().timestamp() < unlock_after {
            return Err(ContractError::StakeLocked);
        }
        if amount > node.stake {
            return Err(ContractError::InsufficientStake);
        }

        node.stake = node
            .stake
            .checked_sub(amount)
            .ok_or(ContractError::Overflow)?;

        if node.stake < storage::get_min_stake(&env) {
            node.status = NodeStatus::Inactive;
        }
        node.last_active = env.ledger().timestamp();
        storage::set_node(&env, &node_address, &node);
        Ok(node)
    }

    /// Permanently penalize a misbehaving relay node by forfeiting its stake.
    ///
    /// This function cuts the target node's stake to 0 and permanently sets
    /// its status to `Slashed`. Only the authorized admin can execute this.
    ///
    /// # Parameters
    /// - `env`: Soroban environment.
    /// - `node_address`: Address of the relay node to slash.
    /// - `reason`: A string explaining the reason for the slash (emitted as an event).
    ///
    /// # Errors
    /// - `ContractError::NotRegistered` if the node is not in the registry.
    /// - `ContractError::NodeSlashed` if the node is already slashed.
    /// - (Auth) Soroban will automatically panic if the caller is not the `Admin`.
    pub fn slash(env: Env, node_address: Address, reason: String) -> Result<(), ContractError> {
        // Only the admin is authorized to slash nodes.
        storage::get_admin(&env).require_auth();

        let mut node =
            storage::get_node(&env, &node_address).ok_or(ContractError::NotRegistered)?;

        // Ensure we don't slash a node that is already slashed.
        if matches!(node.status, NodeStatus::Slashed) {
            return Err(ContractError::NodeSlashed);
        }

        // Apply penalty: total loss of stake
        node.stake = 0;
        node.status = NodeStatus::Slashed;
        node.last_active = env.ledger().timestamp();

        // TODO: transfer slashed stake to treasury
        storage::set_node(&env, &node_address, &node);

        // Emit an event so the slashing reason is auditable on-chain.
        env.events()
            .publish(("slash",), (node_address.clone(), reason));

        Ok(())
    }

    pub fn get_node(env: Env, address: Address) -> Result<RelayNode, ContractError> {
        storage::get_node(&env, &address).ok_or(ContractError::NotRegistered)
    }

    pub fn is_active(env: Env, address: Address) -> bool {
        matches!(
            storage::get_node(&env, &address).map(|n| n.status),
            Some(NodeStatus::Active)
        )
    }
}
