use soroban_sdk::{symbol_short, Env, Symbol};

use crate::types::PauseRecord;

const LEDGER_BUMP_AMOUNT: u32 = 518_400;
const LEDGER_BUMP_THRESHOLD: u32 = 259_200;

const PAUSED_KEY: Symbol = symbol_short!("PAUSED");
const PAUSE_RECORD_KEY: Symbol = symbol_short!("RECORD");
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const INIT_KEY: Symbol = symbol_short!("INIT");

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(LEDGER_BUMP_THRESHOLD, LEDGER_BUMP_AMOUNT);
}

pub fn is_paused(env: &Env) -> bool {
    env.storage().instance().get(&PAUSED_KEY).unwrap_or(false)
}

pub fn set_paused(env: &Env, paused: bool) {
    env.storage().instance().set(&PAUSED_KEY, &paused);
}

pub fn set_pause_record(env: &Env, record: &PauseRecord) {
    env.storage().instance().set(&PAUSE_RECORD_KEY, record);
}

pub fn get_pause_record(env: &Env) -> Option<PauseRecord> {
    env.storage().instance().get(&PAUSE_RECORD_KEY)
}

pub fn remove_pause_record(env: &Env) {
    env.storage().instance().remove(&PAUSE_RECORD_KEY);
}

pub fn set_admin_council(env: &Env, council: &crate::types::AdminCouncil) {
    env.storage().instance().set(&ADMIN_KEY, council);
}

pub fn get_admin_council(env: &Env) -> Option<crate::types::AdminCouncil> {
    env.storage().instance().get(&ADMIN_KEY)
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&INIT_KEY)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&INIT_KEY, &true);
}
