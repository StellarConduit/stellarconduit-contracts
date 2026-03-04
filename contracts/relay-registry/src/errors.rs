//! # Relay Registry Contract — `errors.rs`
//!
//! Defines all error codes returned by the Relay Registry contract.
//! All errors are exposed as a `ContractError` enum that maps to Soroban
//! `contracterror` integer values consumable by clients.
//!
//! ## Error codes to implement
//! - `AlreadyRegistered (1)` — Node address is already registered in the registry
//! - `NotRegistered (2)` — Node address is not found in the registry
//! - `InsufficientStake (3)` — Stake amount is below the protocol minimum
//! - `NodeNotActive (4)` — Operation requires the node to be in Active status
//! - `NodeSlashed (5)` — Operation is blocked because the node has been slashed
//! - `StakeLocked (6)` — Unstake attempt during the stake lock period
//! - `UnauthorizedSlash (7)` — Caller is not authorized to slash this node
//! - `InvalidMetadata (8)` — Provided metadata fails validation
//! - `Overflow (9)` — Arithmetic overflow in stake calculation
//!
//! implementation tracked in GitHub issue

use soroban_sdk::contracterror;

/// All error codes returned by the Relay Registry contract.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Node address is already registered in the registry.
    AlreadyRegistered = 1,

    /// Node address is not found in the registry.
    NotRegistered = 2,

    /// Stake amount is below the protocol minimum.
    InsufficientStake = 3,

    /// Operation requires the node to be in Active status.
    NodeNotActive = 4,

    /// Operation is blocked because the node has been slashed.
    NodeSlashed = 5,

    /// Unstake attempted during the stake lock period.
    StakeLocked = 6,

    /// Caller is not authorized to slash this node.
    UnauthorizedSlash = 7,

    /// Provided metadata fails validation.
    InvalidMetadata = 8,

    /// Arithmetic overflow in stake calculation.
    Overflow = 9,
}
