//! # Dispute Resolver Contract — `errors.rs`
//!
//! Defines all error codes returned by the Dispute Resolver contract.
//! All errors are exposed as a `ContractError` enum that maps to Soroban
//! `contracterror` integer values consumable by clients.
//!
//! ## Error codes to implement
//! - `DisputeNotFound (1)` — The specified dispute_id does not exist in storage
//! - `DisputeAlreadyResolved (2)` — The dispute has already been resolved or ruled on
//! - `DisputeExpired (3)` — The dispute passed its resolution deadline without a response
//! - `DisputeNotResolvable (4)` — The dispute is still open and awaiting a counter-proof
//! - `ProofAlreadySubmitted (5)` — The calling party has already submitted a proof for this dispute
//! - `InvalidProof (6)` — The submitted relay chain proof fails cryptographic verification
//! - `Unauthorized (7)` — Caller is not a party to this dispute
//! - `TxAlreadyDisputed (8)` — A dispute for this transaction ID already exists
//! - `Overflow (9)` — Arithmetic overflow in dispute ID generation
//!
//! implementation tracked in GitHub issue

#![allow(unused)]

use soroban_sdk::contracterror;

// implementation tracked in GitHub issue
