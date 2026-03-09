//! # Dispute Resolver Contract — `errors.rs`
//!
//! Defines all error codes returned by the Dispute Resolver contract.
//! All errors are exposed as a `ContractError` enum that maps to Soroban
//! `contracterror` integer values consumable by clients.

use soroban_sdk::contracterror;

/// Contract error codes returned by the Dispute Resolver contract.
///
/// Each variant represents a typed failure state for arbitration flows
/// and maps to a stable `u32` value for clients and callers.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Thrown when the referenced dispute ID does not exist in storage.
    DisputeNotFound = 1,
    /// Thrown when a dispute is not currently in the Open state.
    NotOpen = 2,
    /// Thrown when resolution is attempted before a respondent has submitted a response.
    NotResponded = 3,
    /// Thrown when the response deadline has already passed.
    ResolutionWindowExpired = 4,
    /// Thrown when resolution is attempted while the respondent window is still active.
    ResolutionWindowActive = 5,
    /// Thrown when the cryptographic signature on a provided proof is invalid.
    InvalidProofSignature = 6,
    /// Thrown when a provided relay chain hash is malformed.
    InvalidChainHash = 7,
    /// Thrown when a dispute for the same transaction ID has already been raised.
    DuplicateDispute = 8,
    /// Thrown when the caller is not authorized to perform the action.
    Unauthorized = 9,
    /// Thrown when the dispute has already been finalized.
    AlreadyResolved = 10,
    /// Thrown when contract initialization has not been performed yet.
    NotInitialized = 11,
    /// Thrown when initialization is attempted after the contract is already initialized.
    AlreadyInitialized = 12,
}
