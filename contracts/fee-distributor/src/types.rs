//! # Fee Distributor Contract — `types.rs`
//!
//! Defines all data structures used by the Fee Distributor contract.

use soroban_sdk::{contracttype, Address, Vec};

/// A multi-signature admin council requiring threshold approvals.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminCouncil {
    /// List of council member addresses (max 10)
    pub members: Vec<Address>,
    /// Minimum number of members required to authorize a sensitive action
    pub threshold: u32,
}

/// A record of a single fee distribution event.
///
/// This struct permanently records each fee distribution that occurs when
/// a transaction batch is successfully settled on Stellar. It tracks which
/// relay node settled the batch, the total fee amount, the treasury's share,
/// and when the distribution occurred.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeeEntry {
    /// Unique identifier of the settled transaction batch.
    pub batch_id: u64,
    /// The relay node that settled the batch.
    pub relay_address: Address,
    /// Total fee distributed for this batch.
    pub amount: i128,
    /// Portion of the fee sent to the protocol treasury.
    pub treasury_share: i128,
    /// Ledger timestamp when the distribution occurred.
    pub settled_at: u64,
}

/// Cumulative earnings state tracked per relay node.
///
/// This struct maintains the lifetime earnings history for each relay node,
/// tracking total fees earned, total fees claimed, and the current unclaimed
/// balance available for withdrawal.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EarningsRecord {
    /// Lifetime total fees earned by this relay node.
    pub total_earned: i128,
    /// Lifetime total fees already claimed and paid out.
    pub total_claimed: i128,
    /// Current claimable balance (total_earned - total_claimed).
    pub unclaimed: i128,
}

/// Global protocol fee configuration.
///
/// This struct stores the protocol-wide fee settings, including the fee rate
/// applied to transaction batches and the treasury's share of each distribution.
/// The admin address is authorized to update these settings via `set_fee_rate()`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeeConfig {
    /// Fee rate in basis points (e.g., 50 = 0.5%).
    pub fee_rate_bps: u32,
    /// Treasury's share of each distribution in basis points.
    pub treasury_share_bps: u32,
    /// Council authorized to update fee config via `set_fee_rate()`.
    pub council: AdminCouncil,
}
