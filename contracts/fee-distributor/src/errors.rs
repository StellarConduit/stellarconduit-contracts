//! # Fee Distributor Contract — `errors.rs`
//!
//! Defines all error codes returned by the Fee Distributor contract.
//! All errors are exposed as a `ContractError` enum that maps to Soroban
//! `contracterror` integer values consumable by clients.
//!
//! ## Error codes to implement
//! - `BatchAlreadyDistributed (1)` — Fee for this batch_id has already been distributed
//! - `BatchNotFound (2)` — The specified batch_id does not exist
//! - `NothingToClaim (3)` — The relay node has no unclaimed earnings
//! - `InvalidFeeRate (4)` — Fee rate is outside of the allowed range
//! - `Unauthorized (5)` — Caller is not authorized to perform this action (e.g., set_fee_rate)
//! - `InvalidBatchSize (6)` — Batch size is zero or exceeds the maximum
//! - `TreasuryTransferFailed (7)` — Token transfer to treasury address failed
//! - `Overflow (8)` — Arithmetic overflow in fee calculation
//!
//! implementation tracked in GitHub issue

#![allow(unused)]

use soroban_sdk::contracterror;

// implementation tracked in GitHub issue
