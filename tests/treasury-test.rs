//! # Treasury — Integration Test Suite
//!
//! Unit and integration tests for the Protocol Treasury contract.
//!
//! ## Test cases to implement
//!
//! ### Deposits
//! - `test_deposit_increases_balance` — Balance increases by the deposited amount
//! - `test_deposit_logs_entry` — A `TreasuryEntry` of kind `Deposit` is recorded
//! - `test_deposit_zero_amount` — Returns `InvalidAmount` for zero deposits
//!
//! ### Withdrawals
//! - `test_withdraw_by_admin` — Admin can withdraw funds to a recipient
//! - `test_withdraw_unauthorized` — Returns `Unauthorized` for non-admin callers
//! - `test_withdraw_insufficient_balance` — Returns `InsufficientBalance` if over balance
//! - `test_withdraw_logs_entry` — A `TreasuryEntry` of kind `Withdrawal` is recorded
//! - `test_withdraw_decreases_balance` — Balance decreases by the withdrawn amount
//!
//! ### Allocations
//! - `test_allocate_creates_record` — Creates an `AllocationRecord` for the program
//! - `test_allocate_updates_existing` — Updates existing program allocation
//! - `test_allocate_over_balance` — Returns `InsufficientBalance` if allocation exceeds balance
//!
//! ### Balance & History
//! - `test_get_balance_initial` — Returns zero balance on a newly deployed contract
//! - `test_get_balance_after_operations` — Returns correct balance after deposits and withdrawals
//! - `test_get_history_empty` — Returns empty history for a new contract
//! - `test_get_history_after_activity` — Returns all recorded entries in chronological order
//!
//! implementation tracked in GitHub issue

// implementation tracked in GitHub issue
