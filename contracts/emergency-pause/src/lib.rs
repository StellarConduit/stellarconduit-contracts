#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol};

pub mod errors;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::ContractError;
use crate::types::{AdminCouncil, PauseRecord};

fn require_council_auth(env: &Env) -> Result<Address, ContractError> {
    let council = storage::get_admin_council(env).ok_or(ContractError::NotInitialized)?;
    let mut authorized = 0u32;
    let mut triggered_by: Option<Address> = None;

    for member in council.members.iter() {
        member.require_auth();
        if triggered_by.is_none() {
            triggered_by = Some(member.clone());
        }
        authorized += 1;
        if authorized >= council.threshold {
            break;
        }
    }

    if authorized < council.threshold {
        return Err(ContractError::Unauthorized);
    }

    Ok(triggered_by.unwrap())
}

#[contract]
pub struct EmergencyPauseContract;

#[contractimpl]
impl EmergencyPauseContract {
    pub fn initialize(env: Env, admin_council: AdminCouncil) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        if storage::is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }

        if admin_council.threshold == 0 || admin_council.members.len() < admin_council.threshold {
            return Err(ContractError::Unauthorized);
        }

        storage::set_admin_council(&env, &admin_council);
        storage::set_initialized(&env);
        Ok(())
    }

    pub fn pause(env: Env, reason: String) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        if !storage::is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        if reason.len() > 256 {
            return Err(ContractError::ReasonTooLong);
        }

        if storage::is_paused(&env) {
            return Err(ContractError::AlreadyPaused);
        }

        let triggered_by = require_council_auth(&env)?;
        let triggered_at = env.ledger().timestamp();
        let record = PauseRecord {
            triggered_at,
            reason: reason.clone(),
            triggered_by: triggered_by.clone(),
        };

        storage::set_paused(&env, true);
        storage::set_pause_record(&env, &record);

        env.events().publish(
            (Symbol::new(&env, "paused"),),
            (reason, triggered_at, triggered_by),
        );

        Ok(())
    }

    pub fn unpause(env: Env) -> Result<(), ContractError> {
        storage::extend_instance_ttl(&env);
        if !storage::is_initialized(&env) {
            return Err(ContractError::NotInitialized);
        }

        if !storage::is_paused(&env) {
            return Err(ContractError::NotPaused);
        }

        require_council_auth(&env)?;
        storage::set_paused(&env, false);
        storage::remove_pause_record(&env);

        env.events().publish(
            (Symbol::new(&env, "unpaused"),),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    pub fn is_paused(env: Env) -> bool {
        storage::extend_instance_ttl(&env);
        storage::is_paused(&env)
    }

    pub fn get_pause_record(env: Env) -> Option<PauseRecord> {
        storage::extend_instance_ttl(&env);
        storage::get_pause_record(&env)
    }
}
