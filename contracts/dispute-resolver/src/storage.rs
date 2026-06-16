//! # Dispute Resolver Contract — `storage.rs`
//!
//! Provides typed helper functions for reading and writing persistent contract
//! storage using Soroban's `Env::storage()` API.
//!
//! ## Storage keys to implement
//! - `DataKey::Dispute(u64)` — Stores a `Dispute` keyed by dispute_id
//! - `DataKey::Ruling(u64)` — Stores a `Ruling` keyed by dispute_id
//! - `DataKey::DisputeCount` — Monotonically incrementing dispute ID counter
//! - `DataKey::ResolutionWindow` — Number of ledgers allowed for dispute response
//! - `DataKey::Admin` — Address authorized to configure the contract
//!
//! ## Functions to implement
//! - `get_dispute(env, dispute_id) -> Option<Dispute>` — Load a dispute from storage
//! - `set_dispute(env, dispute_id, dispute)` — Persist a dispute to storage
//! - `get_ruling(env, dispute_id) -> Option<Ruling>` — Load a ruling from storage
//! - `set_ruling(env, dispute_id, ruling)` — Persist a ruling to storage
//! - `next_dispute_id(env) -> u64` — Atomically increment and return the next dispute ID
//! - `get_resolution_window(env) -> u32` — Load the resolution window in ledgers
//!
//! implementation tracked in GitHub issue

use soroban_sdk::{contracttype, Address, BytesN, Env};

use crate::types::{Dispute, Ruling};

// Bump by ~30 days (assuming ~5 seconds per ledger)
const LEDGER_BUMP_AMOUNT: u32 = 518_400;
// Bump if remaining life is less than ~15 days
const LEDGER_BUMP_THRESHOLD: u32 = 259_200;

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Dispute(u64),
    DisputeCount,
    Ruling(u64),
    ResolutionWindow,
    AdminCouncil,
    TxDispute(BytesN<32>),
    /// Stores the raw 32-byte Ed25519 public key for an Address.
    PublicKey(Address),
    /// Stores the bond amount required to raise a dispute.
    DisputeBond,
    /// Stores the locked bond amount for a specific dispute ID.
    LockedBond(u64),
    /// Stores the cooldown period in ledgers between disputes.
    CooldownPeriod,
    /// Stores the ledger sequence of the last dispute raised by this address.
    LastDisputeLedger(Address),
    /// Stores the SAC token address for bond payments.
    TokenAddress,
    /// Stores the treasury address for forfeited bonds.
    TreasuryAddress,
}

/// Load a dispute by its ID. Returns None if not found.
pub fn get_dispute(env: &Env, dispute_id: u64) -> Option<Dispute> {
    let key = DataKey::Dispute(dispute_id);
    if let Some(dispute) = env.storage().persistent().get::<_, Dispute>(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
        Some(dispute)
    } else {
        None
    }
}

/// Persist an updated dispute record to storage.
pub fn set_dispute(env: &Env, dispute_id: u64, dispute: &Dispute) {
    let key = DataKey::Dispute(dispute_id);
    env.storage().persistent().set(&key, dispute);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Get the current dispute count. Returns 0 if none exist.
pub fn get_dispute_count(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::DisputeCount)
        .unwrap_or(0)
}

/// Increment and return the next available dispute ID.
pub fn get_next_dispute_id(env: &Env) -> u64 {
    let mut count = get_dispute_count(env);
    count = count.checked_add(1).expect("dispute count overflow");
    env.storage().instance().set(&DataKey::DisputeCount, &count);
    count
}

/// Load a ruling by the dispute ID. Returns None if no ruling exists.
pub fn get_ruling(env: &Env, dispute_id: u64) -> Option<Ruling> {
    let key = DataKey::Ruling(dispute_id);
    if let Some(ruling) = env.storage().persistent().get::<_, Ruling>(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
        Some(ruling)
    } else {
        None
    }
}

/// Persist a final ruling to storage.
pub fn set_ruling(env: &Env, dispute_id: u64, ruling: &Ruling) {
    let key = DataKey::Ruling(dispute_id);
    env.storage().persistent().set(&key, ruling);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Get the resolution window in ledgers.
pub fn get_resolution_window(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::ResolutionWindow)
        .unwrap_or(0)
}

/// Set the resolution window in ledgers.
pub fn set_resolution_window(env: &Env, window_ledgers: u32) {
    env.storage()
        .instance()
        .set(&DataKey::ResolutionWindow, &window_ledgers);
}

/// Check if the admin council is set.
pub fn has_admin_council(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::AdminCouncil)
}

/// Get the admin council.
pub fn get_admin_council(env: &Env) -> crate::types::AdminCouncil {
    env.storage()
        .instance()
        .get(&DataKey::AdminCouncil)
        .expect("admin council not set")
}

/// Set the admin council.
pub fn set_admin_council(env: &Env, council: &crate::types::AdminCouncil) {
    env.storage()
        .instance()
        .set(&DataKey::AdminCouncil, council);
}

/// Load the dispute ID associated with a tx_id. Returns None if none exists.
pub fn get_dispute_by_tx(env: &Env, tx_id: &BytesN<32>) -> Option<u64> {
    let key = DataKey::TxDispute(tx_id.clone());
    if let Some(id) = env.storage().persistent().get::<_, u64>(&key) {
        env.storage()
            .persistent()
            .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
        Some(id)
    } else {
        None
    }
}

/// Record that a given tx_id maps to a specific dispute_id.
pub fn set_dispute_by_tx(env: &Env, tx_id: &BytesN<32>, dispute_id: u64) {
    let key = DataKey::TxDispute(tx_id.clone());
    env.storage().persistent().set(&key, &dispute_id);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Load the raw 32-byte Ed25519 public key for an address.
///
/// Panics if the key has not been registered via `set_public_key`.
pub fn get_public_key(env: &Env, address: &Address) -> BytesN<32> {
    let key = DataKey::PublicKey(address.clone());
    let pk = env
        .storage()
        .persistent()
        .get::<_, BytesN<32>>(&key)
        .expect("public key not registered for address");
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
    pk
}

/// Register the raw 32-byte Ed25519 public key for an address.
///
/// Must be called before `raise_dispute` or `respond` so that `resolve`
/// can perform signature verification.
pub fn set_public_key(env: &Env, address: &Address, public_key: &BytesN<32>) {
    let key = DataKey::PublicKey(address.clone());
    env.storage().persistent().set(&key, public_key);
    env.storage()
        .persistent()
        .extend_ttl(&key, LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

/// Get the dispute bond amount required to raise a dispute.
pub fn get_dispute_bond(env: &Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::DisputeBond)
        .unwrap_or(0)
}

/// Set the dispute bond amount required to raise a dispute.
pub fn set_dispute_bond(env: &Env, amount: i128) {
    env.storage().instance().set(&DataKey::DisputeBond, &amount);
}

/// Get the locked bond amount for a specific dispute ID.
pub fn get_locked_bond(env: &Env, dispute_id: u64) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::LockedBond(dispute_id))
        .unwrap_or(0)
}

/// Set the locked bond amount for a specific dispute ID.
pub fn set_locked_bond(env: &Env, dispute_id: u64, amount: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::LockedBond(dispute_id), &amount);
}

/// Get the cooldown period in ledgers between disputes.
pub fn get_cooldown_period(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&DataKey::CooldownPeriod)
        .unwrap_or(0)
}

/// Set the cooldown period in ledgers between disputes.
pub fn set_cooldown_period(env: &Env, period: u32) {
    env.storage()
        .instance()
        .set(&DataKey::CooldownPeriod, &period);
}

/// Get the ledger sequence of the last dispute raised by this address.
pub fn get_last_dispute_ledger(env: &Env, address: &Address) -> Option<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::LastDisputeLedger(address.clone()))
}

/// Set the ledger sequence of the last dispute raised by this address.
pub fn set_last_dispute_ledger(env: &Env, address: &Address, ledger_sequence: u32) {
    env.storage().persistent().set(
        &DataKey::LastDisputeLedger(address.clone()),
        &ledger_sequence,
    );
}

/// Get the SAC token address for bond payments.
pub fn get_token_address(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::TokenAddress)
        .expect("token address not set")
}

/// Set the SAC token address for bond payments.
pub fn set_token_address(env: &Env, token_address: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::TokenAddress, token_address);
}

/// Get the treasury address for forfeited bonds.
pub fn get_treasury_address(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::TreasuryAddress)
        .expect("treasury address not set")
}

/// Set the treasury address for forfeited bonds.
pub fn set_treasury_address(env: &Env, treasury_address: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::TreasuryAddress, treasury_address);
}
