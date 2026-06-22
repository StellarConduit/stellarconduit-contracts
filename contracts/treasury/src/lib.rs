//! # Treasury Contract — `lib.rs`
//!
//! This is the main entry point for the Protocol Treasury Soroban smart contract.
//! The treasury holds protocol funds for relay node incentive programs, grants for
//! operators in underserved and remote regions, and ongoing protocol development.
//!
//! ## Responsibilities
//! - Receive fee allocations from the Fee Distributor contract
//! - Disburse grants and incentives to relay node operators
//! - Track all inflows and outflows with on-chain transparency
//! - Enforce spending limits and require multi-sig authorization for withdrawals
//! - Support future handover to a DAO governance model
//!
//! ## Functions implemented
//! - `deposit(env, from, amount)` — Deposit funds into the protocol treasury
//! - `withdraw(env, to, amount, memo)` — Withdraw funds (authorized callers only)
//! - `create_program(env, name, budget, recipient_address)` — Register a spending program
//! - `allocate(env, program_id, amount)` — Allocate budget and dispatch tokens to the recipient
//! - `get_balance(env)` — Fetch the current treasury token balance
//! - `get_history(env, entry_id)` — Fetch historical ledger audit logs
//! - `get_treasury_stats(env)` — Fetch dashboard telemetry records
//!
//! ## See also
//! - `types.rs` — Data structures (TreasuryEntry, AllocationRecord, SpendingProgram)
//! - `storage.rs` — Persistent storage helpers
//! - `errors.rs` — Contract error codes

#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};

pub mod errors;
pub mod storage;
pub mod types;

use crate::errors::ContractError;
use crate::types::{AdminCouncil, EntryKind, SpendingProgram, TreasuryEntry, TreasuryStats};

fn require_council_auth(env: &Env) {
    let council = storage::get_admin_council(env);
    let mut authorized = 0u32;
    for member in council.members.iter() {
        member.require_auth();
        authorized += 1;
        if authorized >= council.threshold {
            break;
        }
    }

    if authorized < council.threshold {
        panic!("Insufficient approvals");
    }
}

#[contract]
pub struct TreasuryContract;

#[contractimpl]
impl TreasuryContract {
    /// Returns the current treasury token balance.
    ///
    /// Public view function; never errors. Returns 0 if balance is unset.
    pub fn get_balance(env: Env) -> i128 {
        storage::extend_instance_ttl(&env);
        storage::get_balance(&env)
    }

    /// Returns a specific history entry by its ID for auditing.
    ///
    /// Uses `ContractError::ProgramNotFound` when an entry is not found.
    pub fn get_history(env: Env, entry_id: u64) -> Result<TreasuryEntry, ContractError> {
        storage::extend_instance_ttl(&env);
        storage::get_entry(&env, entry_id).ok_or(ContractError::ProgramNotFound)
    }

    /// One-time setup configuring the admin and token address.
    ///
    /// First caller wins; no auth required. Fails if already initialized.
    pub fn initialize(
        env: Env,
        council: AdminCouncil,
        token_address: Address,
    ) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        if storage::has_admin_council(&env) {
            return Err(ContractError::AlreadyInitialized);
        }

        if council.threshold == 0 || council.members.len() < council.threshold {
            return Err(ContractError::InvalidCouncilConfig);
        }

        storage::set_admin_council(&env, &council);
        storage::set_token_address(&env, &token_address);
        storage::set_balance(&env, 0);

        Ok(())
    }

    /// Deposit funds into the protocol treasury.
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current invocation.
    /// - `from`: Address funding the deposit. Must authorize this call.
    /// - `amount`: Amount to deposit. Must be greater than zero.
    ///
    /// # Errors
    /// - `ContractError::InvalidAmount` if `amount` is zero or negative.
    /// - `ContractError::Overflow` if the balance arithmetic overflows.
    pub fn deposit(env: Env, from: Address, amount: i128) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        from.require_auth();

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let balance = storage::get_balance(&env);
        let new_balance = balance.checked_add(amount).ok_or(ContractError::Overflow)?;
        storage::set_balance(&env, new_balance);

        // Update lifetime stats
        let mut stats = storage::get_stats(&env);
        stats.lifetime_deposited = stats
            .lifetime_deposited
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        storage::set_stats(&env, &stats);

        let entry = TreasuryEntry {
            kind: EntryKind::Deposit,
            amount,
            actor: from.clone(),
            recipient: None,
            memo: String::from_str(&env, "deposit"),
            ledger: env.ledger().sequence() as u64,
        };
        storage::set_entry(&env, entry);

        let token_address = storage::get_token_address(&env);
        let token = token::Client::new(&env, &token_address);
        token.transfer(&from, &env.current_contract_address(), &amount);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "treasury"),
                soroban_sdk::Symbol::new(&env, "deposit"),
            ),
            (from.clone(), amount),
        );

        Ok(())
    }

    /// Withdraw funds from the protocol treasury (admin only).
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current invocation.
    /// - `to`: Recipient of the withdrawal.
    /// - `amount`: Amount to withdraw. Must be greater than zero.
    /// - `memo`: Human-readable memo for the withdrawal entry.
    ///
    /// # Errors
    /// - `ContractError::InvalidAmount` if `amount` is zero or negative.
    /// - `ContractError::InsufficientBalance` if treasury balance is too low.
    /// - `ContractError::Overflow` if arithmetic underflows/overflows.
    pub fn withdraw(
        env: Env,
        to: Address,
        amount: i128,
        memo: String,
    ) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        require_council_auth(&env);

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let balance = storage::get_balance(&env);
        if balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        let new_balance = balance.checked_sub(amount).ok_or(ContractError::Overflow)?;
        storage::set_balance(&env, new_balance);

        // Update lifetime stats
        let mut stats = storage::get_stats(&env);
        stats.lifetime_withdrawn = stats
            .lifetime_withdrawn
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        storage::set_stats(&env, &stats);

        let entry = TreasuryEntry {
            kind: EntryKind::Withdrawal,
            amount,
            actor: env.current_contract_address(),
            recipient: Some(to.clone()),
            memo: memo.clone(),
            ledger: env.ledger().sequence() as u64,
        };
        storage::set_entry(&env, entry);

        let token = token::Client::new(&env, &storage::get_token_address(&env));
        token.transfer(&env.current_contract_address(), &to, &amount);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "treasury"),
                soroban_sdk::Symbol::new(&env, "withdraw"),
            ),
            (to.clone(), amount, memo),
        );

        Ok(())
    }

    /// Create a new spending program with a specified budget.
    ///
    /// # Parameters
    /// - `env`: Soroban environment.
    /// - `name`: Name of the program (3-64 chars).
    /// - `budget`: Initial budget for the program. Must be > 0.
    /// - `recipient_address`: Address that receives tokens when allocate() is called.
    pub fn create_program(
        env: Env,
        name: String,
        budget: i128,
        recipient_address: Address,
    ) -> Result<u64, ContractError> {
        storage::extend_instance_ttl(&env);
        require_council_auth(&env);

        if budget <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        if name.len() < 3 || name.len() > 64 {
            return Err(ContractError::InvalidProgramName);
        }

        let program_id = storage::increment_program_count(&env);

        let program = SpendingProgram {
            program_id,
            budget,
            spent: 0,
            active: true,
            name: name.clone(),
            recipient_address: recipient_address.clone(),
        };

        storage::set_spending_program(&env, program_id, program);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "treasury"),
                soroban_sdk::Symbol::new(&env, "create_program"),
            ),
            (program_id, name, budget),
        );

        Ok(program_id)
    }

    /// Update the budget of an existing spending program.
    pub fn update_program_budget(
        env: Env,
        program_id: u64,
        new_budget: i128,
    ) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        require_council_auth(&env);

        let mut program = storage::get_spending_program(&env, program_id)
            .ok_or(ContractError::ProgramNotFound)?;

        if new_budget < program.spent {
            return Err(ContractError::InvalidAmount);
        }

        program.budget = new_budget;
        storage::set_spending_program(&env, program_id, program);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "treasury"),
                soroban_sdk::Symbol::new(&env, "update_budget"),
            ),
            (program_id, new_budget),
        );

        Ok(())
    }

    /// Deactivate a spending program.
    pub fn deactivate_program(env: Env, program_id: u64) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        require_council_auth(&env);

        let mut program = storage::get_spending_program(&env, program_id)
            .ok_or(ContractError::ProgramNotFound)?;

        program.active = false;
        storage::set_spending_program(&env, program_id, program);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "treasury"),
                soroban_sdk::Symbol::new(&env, "deactivate_program"),
            ),
            (program_id,),
        );

        Ok(())
    }

    /// Get details of a spending program.
    pub fn get_program(env: Env, program_id: u64) -> Result<SpendingProgram, ContractError> {
        storage::extend_instance_ttl(&env);
        storage::get_spending_program(&env, program_id).ok_or(ContractError::ProgramNotFound)
    }

    /// Allocate treasury funds to a spending program (admin only).
    ///
    /// # Parameters
    /// - `env`: Soroban environment for the current invocation.
    /// - `program_id`: ID of the spending program to allocate to.
    /// - `amount`: Amount to allocate. Must be greater than zero.
    ///
    /// # Errors
    /// - `ContractError::InvalidAmount` if `amount` is zero or negative.
    /// - `ContractError::ProgramNotFound` if the program does not exist.
    /// - `ContractError::ProgramInactive` if the program is not active.
    /// - `ContractError::ProgramOverBudget` if the allocation exceeds budget.
    /// - `ContractError::InsufficientBalance` if treasury balance is too low.
    /// - `ContractError::Overflow` if arithmetic overflows.
    pub fn allocate(env: Env, program_id: u64, amount: i128) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        require_council_auth(&env);

        if amount <= 0 {
            return Err(ContractError::InvalidAmount);
        }

        let mut program = storage::get_spending_program(&env, program_id)
            .ok_or(ContractError::ProgramNotFound)?;

        if !program.active {
            return Err(ContractError::ProgramInactive);
        }

        let new_spent = program
            .spent
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        if new_spent > program.budget {
            return Err(ContractError::ProgramOverBudget);
        }

        let balance = storage::get_balance(&env);
        if balance < amount {
            return Err(ContractError::InsufficientBalance);
        }

        program.spent = new_spent;

        // FIXED: Clone the program struct here to keep it alive for code processing downstream
        storage::set_spending_program(&env, program_id, program.clone());

        let new_balance = balance.checked_sub(amount).ok_or(ContractError::Overflow)?;
        storage::set_balance(&env, new_balance);

        // Update lifetime stats
        let mut stats = storage::get_stats(&env);
        stats.lifetime_allocated = stats
            .lifetime_allocated
            .checked_add(amount)
            .ok_or(ContractError::Overflow)?;
        storage::set_stats(&env, &stats);

        // Disburse physical Stellar Asset Contract tokens to the program's recipient
        let token = token::Client::new(&env, &storage::get_token_address(&env));
        token.transfer(
            &env.current_contract_address(),
            &program.recipient_address,
            &amount,
        );

        let entry = TreasuryEntry {
            kind: EntryKind::Allocation,
            amount,
            actor: env.current_contract_address(),
            recipient: Some(program.recipient_address.clone()),
            memo: String::from_str(&env, "allocation"),
            ledger: env.ledger().sequence() as u64,
        };
        storage::set_entry(&env, entry);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "treasury"),
                soroban_sdk::Symbol::new(&env, "allocate"),
            ),
            (program_id, amount, program.recipient_address.clone()),
        );

        Ok(())
    }

    /// Returns aggregate statistics for the treasury.
    pub fn get_treasury_stats(env: Env) -> TreasuryStats {
        storage::extend_instance_ttl(&env);
        let mut stats = storage::get_stats(&env);
        stats.current_balance = storage::get_balance(&env);
        stats
    }
}

#[cfg(test)]
mod test;
