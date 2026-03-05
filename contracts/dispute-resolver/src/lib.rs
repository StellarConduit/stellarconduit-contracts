//! # Dispute Resolver Contract — `lib.rs`
//!
//! This is the main entry point for the Dispute Resolver Soroban smart contract.
//! It handles final on-chain arbitration for double-spend conflicts that cannot be
//! resolved off-chain by the StellarConduit sync engine.
//!
//! ## Responsibilities
//! - Accept dispute submissions with cryptographic relay chain proofs
//! - Enforce dispute submission and response deadlines
//! - Evaluate competing cryptographic proofs deterministically
//! - Issue a final ruling and trigger appropriate fund recovery
//! - Penalize the relay node that submitted the invalid transaction
//!
//! ## Functions
//! - `raise_dispute(env, initiator, tx_id, proof)` — Submit a new dispute with a relay chain proof
//! - `respond(env, respondent, dispute_id, proof)` — Submit a counter-proof to an open dispute
//! - `resolve(env, dispute_id)` — Resolve a dispute after the evaluation period
//! - `get_dispute(env, dispute_id)` — Fetch dispute details and current status
//! - `respond(env, respondent, dispute_id, proof)` — Submit a counter-proof to an open dispute
//! - `resolve(env, dispute_id)` — Resolve a dispute after the evaluation period
//! - `get_ruling(env, dispute_id)` — Fetch the final ruling for a resolved dispute
//! - `initialize(env, admin, resolution_window)` — One-time setup for the contract
//!
//! ## See also
//! - `types.rs` — Data structures (Dispute, DisputeStatus, Ruling, RelayChainProof)
//! - `storage.rs` — Persistent storage helpers
//! - `errors.rs` — Contract error codes
//!
//! implementation tracked in GitHub issue

#![no_std]

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String};

pub mod errors;
pub mod storage;
pub mod types;

use crate::errors::ContractError;
use crate::types::{Dispute, DisputeStatus, OptionalRelayChainProof, RelayChainProof, Ruling};

#[contract]
pub struct DisputeResolverContract;

#[contractimpl]
impl DisputeResolverContract {
    /// Submit a new dispute for a suspected double-spend, recording the initiator's
    /// cryptographic relay chain proof on-chain and setting a response deadline.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current contract invocation.
    /// - `initiator`: Address of the party raising the dispute. Must authorize this call.
    /// - `tx_id`: The 32-byte Stellar transaction ID under dispute.
    /// - `proof`: The initiator's cryptographic relay chain proof.
    ///
    /// # Returns
    /// The newly assigned `dispute_id` (`u64`) for tracking the dispute.
    ///
    /// # Errors
    /// - `ContractError::DuplicateDispute` if a dispute for this `tx_id` already exists.
    pub fn raise_dispute(
        env: Env,
        initiator: Address,
        tx_id: BytesN<32>,
        proof: RelayChainProof,
    ) -> Result<u64, ContractError> {
        initiator.require_auth();

        // Guard against duplicate disputes for the same tx_id.
        if storage::get_dispute_by_tx(&env, &tx_id).is_some() {
            return Err(ContractError::DuplicateDispute);
        }

        // Auto-increment and get the next dispute ID (starts at 1).
        let dispute_id = storage::get_next_dispute_id(&env);

        // Compute the response deadline as a ledger sequence number.
        let resolution_window = storage::get_resolution_window(&env);
        let resolve_by = env.ledger().sequence() + resolution_window;

        let dispute = Dispute {
            dispute_id,
            tx_id: tx_id.clone(),
            initiator: initiator.clone(),
            respondent: None,
            initiator_proof: proof,
            respondent_proof: OptionalRelayChainProof::None,
            status: DisputeStatus::Open,
            raised_at: env.ledger().timestamp(),
            resolve_by: resolve_by as u64,
        };

        // Persist the dispute and record the tx → dispute_id mapping.
        storage::set_dispute(&env, dispute_id, &dispute);
        storage::set_dispute_by_tx(&env, &tx_id, dispute_id);

        // Emit event for off-chain indexers.
        env.events()
            .publish(("raise_dispute",), (initiator, dispute_id, tx_id));

        Ok(dispute_id)
    }

    /// Fetch the full dispute record by its ID.
    ///
    /// # Parameters
    /// - `env`: Soroban environment.
    /// - `dispute_id`: The ID of the dispute.
    ///
    /// # Returns
    /// The `Dispute` record if found.
    ///
    /// # Errors
    /// - `ContractError::DisputeNotFound` if the ID does not exist.
    pub fn get_dispute(env: Env, dispute_id: u64) -> Result<Dispute, ContractError> {
        storage::get_dispute(&env, dispute_id).ok_or(ContractError::DisputeNotFound)
    }

    /// Fetch the final ruling for a resolved dispute.
    ///
    /// # Parameters
    /// - `env`: Soroban environment.
    /// - `dispute_id`: The ID of the resolved dispute.
    ///
    /// # Returns
    /// The `Ruling` record if found.
    ///
    /// # Errors
    /// - `ContractError::DisputeNotFound` if no ruling exists for this ID.
    pub fn get_ruling(env: Env, dispute_id: u64) -> Result<Ruling, ContractError> {
        storage::get_ruling(&env, dispute_id).ok_or(ContractError::DisputeNotFound)
    }

    /// One-time initialization of the Dispute Resolver contract.
    ///
    /// Sets the admin capable of upgrading/configuring the contract, and
    /// configures the global resolution window for how long a respondent has
    /// to provide a counter-proof.
    ///
    /// # Parameters
    /// - `env`: Soroban environment.
    /// - `admin`: The address to set as the contract administrator.
    /// - `resolution_window`: Number of ledgers allowed for responding to disputes.
    ///
    /// # Errors
    /// - `ContractError::AlreadyInitialized` if called more than once.
    /// - `ContractError::InvalidConfig` if `resolution_window` is zero.
    pub fn initialize(
        env: Env,
        admin: Address,
        resolution_window: u32,
    ) -> Result<(), ContractError> {
        if storage::has_admin(&env) {
            return Err(ContractError::AlreadyInitialized);
        }

        if resolution_window == 0 {
            return Err(ContractError::InvalidConfig);
        }

        storage::set_admin(&env, &admin);
        storage::set_resolution_window(&env, resolution_window);

        Ok(())
    }
}
