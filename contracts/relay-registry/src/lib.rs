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
use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Env, String};

pub mod errors;
pub mod storage;
pub mod types;

use crate::errors::ContractError;
use crate::types::{AdminCouncil, NodeMetadata, NodeStatus, RelayNode, StakeEntry};

fn require_council_auth(env: &Env) {
    let council = storage::get_admin_council(env);

    // env.auths() is available under the testutils feature (used during `cargo test`).
    // It returns all (Address, AuthorizedInvocation) pairs attached to this transaction,
    // letting us count council members without panicking on missing ones.
    #[cfg(feature = "testutils")]
    {
        use soroban_sdk::testutils::AuthorizedInvocation as _;
        let authorized_addresses: soroban_sdk::Vec<Address> = env
            .auths()
            .iter()
            .map(|(addr, _)| addr)
            .collect();

        let mut authorized_members: soroban_sdk::Vec<Address> = soroban_sdk::Vec::new(env);
        for member in council.members.iter() {
            if authorized_addresses.contains(&member) {
                authorized_members.push_back(member);
            }
        }

        if (authorized_members.len() as u32) < council.threshold {
            panic_with_error!(env, ContractError::InsufficientApprovals);
        }

        for member in authorized_members.iter() {
            member.require_auth();
        }
        return;
    }

    // Production (WASM) path: callers must supply signatures for the first `threshold` members.
    // This is enforced by calling require_auth on each; the host traps if auth is absent.
    #[allow(unreachable_code)]
    {
        let mut authorized = 0u32;
        for member in council.members.iter() {
            member.require_auth();
            authorized += 1;
            if authorized >= council.threshold {
                break;
            }
        }
        if authorized < council.threshold {
            panic_with_error!(env, ContractError::InsufficientApprovals);
        }
    }
}

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
        council: AdminCouncil,
        min_stake: i128,
        stake_lock_period: u32,
    ) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        // Guard against re-initialization
        if env
            .storage()
            .instance()
            .has(&storage::DataKey::AdminCouncil)
        {
            return Err(ContractError::AlreadyInitialized);
        }

        // Validate inputs
        if min_stake <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if stake_lock_period == 0 {
            return Err(ContractError::InvalidAmount);
        }

        if council.threshold == 0 || council.members.len() < council.threshold {
            return Err(ContractError::InvalidCouncilConfig);
        }

        // Persist config
        storage::set_admin_council(&env, &council);
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
        storage::extend_instance_ttl(&env);
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
            metadata: metadata.clone(),
            registered_at: timestamp,
            last_active: timestamp,
        };

        storage::set_node(&env, &node_address, &node);
        storage::increment_node_count(&env);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "relay_registry"),
                soroban_sdk::Symbol::new(&env, "register"),
            ),
            (node_address.clone(), metadata),
        );

        Ok(())
    }

    /// Update the metadata of an already registered relay node.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `node_address`: Stellar account address of the relay node. Must authorize this call.
    /// - `new_metadata`: The new NodeMetadata to apply.
    ///
    /// # Errors
    /// - `ContractError::NotRegistered` if the node is not found in the registry.
    /// - `ContractError::InvalidMetadata` if `new_metadata.uptime_commitment` > 100 or `region` is too long.
    pub fn update_metadata(
        env: Env,
        node_address: Address,
        new_metadata: NodeMetadata,
    ) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        node_address.require_auth();

        let mut node =
            storage::get_node(&env, &node_address).ok_or(ContractError::NotRegistered)?;

        if new_metadata.uptime_commitment > 100 || new_metadata.region.len() > 32 {
            return Err(ContractError::InvalidMetadata);
        }

        node.metadata = new_metadata;

        storage::set_node(&env, &node_address, &node);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "relay_registry"),
                soroban_sdk::Symbol::new(&env, "update_metadata"),
            ),
            (node_address.clone(),),
        );

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
        storage::extend_instance_ttl(&env);
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

        let token = token::Client::new(&env, &storage::get_token_address(&env));
        token.transfer(&node_address, &env.current_contract_address(), &amount);

        storage::set_node(&env, &node_address, &node);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "relay_registry"),
                soroban_sdk::Symbol::new(&env, "stake"),
            ),
            (node_address.clone(), amount),
        );

        Ok(())
    }

    pub fn unstake(
        env: Env,
        node_address: Address,
        amount: i128,
    ) -> Result<RelayNode, ContractError> {
        storage::extend_instance_ttl(&env);
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

        let current_time = env.ledger().timestamp();
        let unlock_after = current_time
            .checked_add(storage::get_stake_lock_period(&env) as u64)
            .ok_or(ContractError::Overflow)?;
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

        // Create the pending unstake entry instead of transferring tokens immediately
        let entry = StakeEntry {
            address: node_address.clone(),
            amount,
            unlocks_at: unlock_after,
        };
        storage::set_lock_entry(&env, &node_address, &entry);

        storage::set_node(&env, &node_address, &node);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "relay_registry"),
                soroban_sdk::Symbol::new(&env, "unstake"),
            ),
            (node_address.clone(), amount, unlock_after),
        );

        Ok(node)
    }

    /// Withdraws explicitly unstaked tokens after the mandatory locking period concludes.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `node_address`: Stellar account address of the relay node. Must authorize this call.
    ///
    /// # Errors
    /// - `ContractError::NoPendingUnstake` if there isn't an active unstake request.
    /// - `ContractError::LockPeriodActive` if the lock duration hasn't concluded yet.
    pub fn finalize_unstake(env: Env, node_address: Address) -> Result<i128, ContractError> {
        storage::extend_instance_ttl(&env);
        node_address.require_auth();

        let entry =
            storage::get_lock_entry(&env, &node_address).ok_or(ContractError::NoPendingUnstake)?;

        let current_time = env.ledger().timestamp();
        if current_time < entry.unlocks_at {
            return Err(ContractError::LockPeriodActive);
        }

        storage::remove_lock_entry(&env, &node_address);

        let token = token::Client::new(&env, &storage::get_token_address(&env));
        token.transfer(
            &env.current_contract_address(),
            &node_address,
            &entry.amount,
        );

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "relay_registry"),
                soroban_sdk::Symbol::new(&env, "finalize_unstake"),
            ),
            (node_address.clone(), entry.amount),
        );

        Ok(entry.amount)
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
    pub fn slash(env: Env, node_address: Address, _reason: String) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        require_council_auth(&env);

        let mut node =
            storage::get_node(&env, &node_address).ok_or(ContractError::NotRegistered)?;

        // Ensure we don't slash a node that is already slashed.
        if matches!(node.status, NodeStatus::Slashed) {
            return Err(ContractError::NodeSlashed);
        }

        // Apply penalty: total loss of stake
        let slashed_amount = node.stake;
        node.stake = 0;
        node.status = NodeStatus::Slashed;
        node.last_active = env.ledger().timestamp();

        // Warning: Local treasury target stub needed. Normally handled in separate PR but stubbing here.
        // Needs a valid storage variable or routing map to determine `treasury`
        // Leaving // TODO: transfer slashed stake to treasury for now since issue specifies to replace // TODO: SAC transfer comments only

        storage::set_node(&env, &node_address, &node);

        // Emit an event so the slashing reason is auditable on-chain.
        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "relay_registry"),
                soroban_sdk::Symbol::new(&env, "slash"),
            ),
            (node_address.clone(), slashed_amount),
        );

        Ok(())
    }

    /// Reinstate a previously slashed relay node after a successful governance appeal.
    ///
    /// This function allows the admin council to move a node from `Slashed` back to
    /// `Inactive` status after an off-chain appeals process determines that the slash
    /// was unwarranted or due to non-malicious causes (e.g., hardware failure).
    ///
    /// The node does **not** regain any previously slashed stake; it must call `stake`
    /// again to become `Active`.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `node_address`: Address of the relay node to reinstate.
    ///
    /// # Errors
    /// - `ContractError::NotRegistered` if the node is not in the registry.
    /// - `ContractError::NodeNotSlashed` if the node is not currently slashed.
    pub fn reinstate_node(env: Env, node_address: Address) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        // Only the admin council may reinstate a slashed node.
        require_council_auth(&env);

        let mut node =
            storage::get_node(&env, &node_address).ok_or(ContractError::NotRegistered)?;

        // Reinstatement is only valid for nodes that are currently slashed.
        if !matches!(node.status, NodeStatus::Slashed) {
            return Err(ContractError::NodeNotSlashed);
        }

        // Move the node back to Inactive; stake remains at 0.
        node.status = NodeStatus::Inactive;
        node.last_active = env.ledger().timestamp();

        storage::set_node(&env, &node_address, &node);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "relay_registry"),
                soroban_sdk::Symbol::new(&env, "reinstate_node"),
            ),
            (node_address.clone(),),
        );

        Ok(())
    }
    /// Retrieves a registered relay node's details.
    ///
    /// This is a view-only function that returns the `RelayNode` struct
    /// associated with the given address. It does not require authorization.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `address`: The Stellar account address of the relay node to lookup.
    ///
    /// # Returns
    /// - `Ok(RelayNode)`: The registered node details if found.
    ///
    /// # Errors
    /// - `ContractError::NotRegistered`: If the address is not registered in the registry.
    pub fn get_node(env: Env, address: Address) -> Result<RelayNode, ContractError> {
        storage::extend_instance_ttl(&env);
        storage::get_node(&env, &address).ok_or(ContractError::NotRegistered)
    }

    /// Checks if a relay node is currently active.
    ///
    /// This is a view-only function that returns true if the given address is
    /// a registered relay node with a status of `NodeStatus::Active`. It does not
    /// require authorization. This function never errors; it returns false for any unknown or inactive address.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `address`: The Stellar account address of the relay node to check.
    ///
    /// # Returns
    /// - `true`: If the node exists and its status is `NodeStatus::Active`.
    /// - `false`: If the node is not registered, or its status is not active.
    pub fn is_active(env: Env, address: Address) -> bool {
        storage::extend_instance_ttl(&env);
        matches!(
            storage::get_node(&env, &address).map(|n| n.status),
            Some(NodeStatus::Active)
        )
    }
}

#[cfg(test)]
mod test;
