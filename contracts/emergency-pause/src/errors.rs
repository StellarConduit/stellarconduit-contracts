use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    AlreadyPaused = 3,
    NotPaused = 4,
    Unauthorized = 5,
    ReasonTooLong = 6,
}
