//! # Fee Distributor Contract — `storage.rs`
//!
//! Provides typed helper functions for reading and writing persistent contract
//! storage using Soroban's `Env::storage()` API.
//!
//! All storage access is isolated to this module. Contract functions use these
//! helpers to read and write data, keeping raw storage operations centralized.

use soroban_sdk::{contracttype, Address, Env};

use crate::types::{EarningsRecord, FeeConfig, FeeEntry};

/// Storage key enum for the Fee Distributor contract.
///
/// All storage keys are defined here to ensure type safety and prevent
/// key collisions. The `#[contracttype]` attribute enables Soroban to
/// serialize these keys for storage operations.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    /// Stores an `EarningsRecord` keyed by relay node address.
    Earnings(Address),
    /// Stores a `FeeEntry` keyed by batch ID.
    FeeEntry(u64),
    /// Stores the global `FeeConfig` struct.
    FeeConfig,
    /// Stores the treasury contract address.
    TreasuryAddress,
}

/// Load the earnings record for a relay node. Returns a zeroed record if not found.
///
/// # Parameters
/// - `env`: Soroban environment.
/// - `address`: Address of the relay node.
///
/// # Returns
/// An `EarningsRecord` with the relay node's earnings data, or a zeroed record
/// if no earnings have been recorded for this address yet.
pub fn get_earnings(env: &Env, address: &Address) -> EarningsRecord {
    env.storage()
        .persistent()
        .get(&DataKey::Earnings(address.clone()))
        .unwrap_or(EarningsRecord {
            total_earned: 0,
            total_claimed: 0,
            unclaimed: 0,
        })
}

/// Persist an updated earnings record for a relay node.
///
/// # Parameters
/// - `env`: Soroban environment.
/// - `address`: Address of the relay node.
/// - `record`: The earnings record to store.
pub fn set_earnings(env: &Env, address: &Address, record: &EarningsRecord) {
    env.storage()
        .persistent()
        .set(&DataKey::Earnings(address.clone()), record);
}

/// Load a fee entry by batch ID. Returns None if not found.
///
/// # Parameters
/// - `env`: Soroban environment.
/// - `batch_id`: Unique identifier of the transaction batch.
///
/// # Returns
/// An `Option<FeeEntry>` containing the fee distribution record if found,
/// or `None` if no entry exists for this batch ID.
pub fn get_fee_entry(env: &Env, batch_id: u64) -> Option<FeeEntry> {
    env.storage().persistent().get(&DataKey::FeeEntry(batch_id))
}

/// Persist a new fee distribution entry.
///
/// # Parameters
/// - `env`: Soroban environment.
/// - `batch_id`: Unique identifier of the transaction batch.
/// - `entry`: The fee entry to store.
pub fn set_fee_entry(env: &Env, batch_id: u64, entry: &FeeEntry) {
    env.storage()
        .persistent()
        .set(&DataKey::FeeEntry(batch_id), entry);
}

/// Load the global fee configuration.
///
/// # Parameters
/// - `env`: Soroban environment.
///
/// # Returns
/// The `FeeConfig` struct containing protocol-wide fee settings.
///
/// # Panics
/// Panics if the fee configuration has not been initialized.
pub fn get_fee_config(env: &Env) -> FeeConfig {
    env.storage()
        .instance()
        .get(&DataKey::FeeConfig)
        .expect("fee config not initialized")
}

/// Persist updated fee configuration.
///
/// # Parameters
/// - `env`: Soroban environment.
/// - `config`: The fee configuration to store.
pub fn set_fee_config(env: &Env, config: &FeeConfig) {
    env.storage().instance().set(&DataKey::FeeConfig, config);
}

/// Load the treasury contract address.
///
/// # Parameters
/// - `env`: Soroban environment.
///
/// # Returns
/// The `Address` of the treasury contract.
///
/// # Panics
/// Panics if the treasury address has not been initialized.
pub fn get_treasury_address(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::TreasuryAddress)
        .expect("treasury address not initialized")
}

/// Set the treasury contract address.
///
/// # Parameters
/// - `env`: Soroban environment.
/// - `address`: The treasury contract address to store.
pub fn set_treasury_address(env: &Env, address: &Address) {
    env.storage()
        .instance()
        .set(&DataKey::TreasuryAddress, address);
}
