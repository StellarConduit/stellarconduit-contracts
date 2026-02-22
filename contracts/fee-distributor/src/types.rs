//! # Fee Distributor Contract — `types.rs`
//!
//! Defines all data structures used by the Fee Distributor contract.
//!
//! ## Types to implement
//! - `FeeEntry` — A record of a single fee distribution event:
//!   - `batch_id: u64` — Identifier of the settled transaction batch
//!   - `relay_address: Address` — The relay node that settled the batch
//!   - `amount: i128` — Total fee distributed for this batch
//!   - `treasury_share: i128` — Portion allocated to the protocol treasury
//!   - `settled_at: u64` — Ledger timestamp of the distribution
//! - `EarningsRecord` — Cumulative earnings state per relay node:
//!   - `total_earned: i128` — Lifetime total fees earned
//!   - `total_claimed: i128` — Lifetime total fees already claimed
//!   - `unclaimed: i128` — Current claimable balance
//! - `FeeConfig` — Protocol fee configuration:
//!   - `fee_rate_bps: u32` — Fee rate in basis points (e.g., 50 = 0.5%)
//!   - `treasury_share_bps: u32` — Treasury's share in basis points
//!   - `admin: Address` — Address authorized to update fee config
//!
//! implementation tracked in GitHub issue

#![allow(unused)]

use soroban_sdk::{contracttype, Address};

// implementation tracked in GitHub issue
