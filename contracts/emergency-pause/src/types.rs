use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminCouncil {
    pub members: Vec<Address>,
    pub threshold: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PauseRecord {
    pub triggered_at: u64,
    pub reason: String,
    pub triggered_by: Address,
}
