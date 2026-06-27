use soroban_sdk::{
    testutils::{Address as _, Events as _},
    Address, Env, String,
};

use crate::{storage, EmergencyPauseContract, EmergencyPauseContractClient};

fn council(env: &Env, threshold: u32) -> crate::types::AdminCouncil {
    let mut members = soroban_sdk::Vec::new(env);
    for _ in 0..threshold {
        members.push_back(Address::generate(env));
    }
    crate::types::AdminCouncil { members, threshold }
}

fn setup<'a>() -> (Env, EmergencyPauseContractClient<'a>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EmergencyPauseContract, ());
    let client = EmergencyPauseContractClient::new(&env, &contract_id);
    let council = council(&env, 1);
    client.initialize(&council);
    (env, client, contract_id)
}

#[test]
fn test_initialize_success() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EmergencyPauseContract, ());
    let client = EmergencyPauseContractClient::new(&env, &contract_id);
    let council = council(&env, 1);

    client.initialize(&council);

    assert!(!client.is_paused());
}

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(EmergencyPauseContract, ());
    let client = EmergencyPauseContractClient::new(&env, &contract_id);
    let council = council(&env, 1);

    client.initialize(&council);
    let result = client.try_initialize(&council);

    assert_eq!(
        result,
        Err(Ok(crate::errors::ContractError::AlreadyInitialized))
    );
}

#[test]
fn test_pause_sets_paused_flag() {
    let (_env, client, _contract_id) = setup();
    let reason = String::from_str(&_env, "test");

    client.pause(&reason);

    assert!(client.is_paused());
}

#[test]
fn test_pause_emits_event() {
    let (env, client, contract_id) = setup();
    let reason = String::from_str(&env, "test");

    client.pause(&reason);

    let events = env.events().all();
    assert_eq!(events.len(), 1);
    let (emitting_contract, _topics, _data) = events.get(0).unwrap();
    assert_eq!(emitting_contract, contract_id);

    let record = env.as_contract(&contract_id, || storage::get_pause_record(&env));
    assert!(record.is_some());
    let record = record.unwrap();
    assert_eq!(record.reason, reason);
    assert_eq!(record.triggered_at, env.ledger().timestamp());
}

#[test]
fn test_unpause_clears_flag() {
    let (env, client, contract_id) = setup();
    let reason = String::from_str(&env, "test");

    client.pause(&reason);
    client.unpause();

    assert!(!client.is_paused());
    assert!(env
        .as_contract(&contract_id, || storage::get_pause_record(&env))
        .is_none());
}

#[test]
fn test_pause_when_already_paused_returns_error() {
    let (env, client, _contract_id) = setup();
    let reason = String::from_str(&env, "test");

    client.pause(&reason);
    let result = client.try_pause(&reason);

    assert_eq!(result, Err(Ok(crate::errors::ContractError::AlreadyPaused)));
}

#[test]
fn test_unpause_when_not_paused_returns_error() {
    let (_env, client, _contract_id) = setup();

    let result = client.try_unpause();

    assert_eq!(result, Err(Ok(crate::errors::ContractError::NotPaused)));
}

#[test]
fn test_get_pause_record_returns_none_when_unpaused() {
    let (env, client, _contract_id) = setup();

    assert!(client.get_pause_record().is_none());
    assert!(env
        .as_contract(&client.address, || storage::get_pause_record(&env))
        .is_none());
}

#[test]
fn test_get_pause_record_returns_record_when_paused() {
    let (env, client, _contract_id) = setup();
    let reason = String::from_str(&env, "security incident");

    client.pause(&reason);

    let record = client.get_pause_record().unwrap();
    assert_eq!(record.reason, reason);
    assert_eq!(record.triggered_at, env.ledger().timestamp());
    let stored_council = env.as_contract(&client.address, || storage::get_admin_council(&env));
    assert_eq!(
        record.triggered_by,
        stored_council.unwrap().members.get(0).unwrap()
    );
}

#[test]
fn test_pause_requires_threshold_auth() {
    use soroban_sdk::{testutils::MockAuth, testutils::MockAuthInvoke, IntoVal};

    let env = Env::default();
    let contract_id = env.register(EmergencyPauseContract, ());
    let client = EmergencyPauseContractClient::new(&env, &contract_id);
    let member_a = Address::generate(&env);
    let member_b = Address::generate(&env);
    let mut members = soroban_sdk::Vec::new(&env);
    members.push_back(member_a.clone());
    members.push_back(member_b.clone());
    let council = crate::types::AdminCouncil {
        members,
        threshold: 2,
    };
    client.initialize(&council);
    let reason = String::from_str(&env, "threshold");

    let result = client
        .mock_auths(&[MockAuth {
            address: &member_a,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "pause",
                args: (reason.clone(),).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .try_pause(&reason);

    assert!(
        result.is_err(),
        "pause should fail when council threshold is not met"
    );

    client
        .mock_auths(&[
            MockAuth {
                address: &member_a,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "pause",
                    args: (reason.clone(),).into_val(&env),
                    sub_invokes: &[],
                },
            },
            MockAuth {
                address: &member_b,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "pause",
                    args: (reason.clone(),).into_val(&env),
                    sub_invokes: &[],
                },
            },
        ])
        .pause(&reason);

    assert!(client.is_paused());
}

#[test]
fn test_stake_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let pause_id = env.register(EmergencyPauseContract, ());
    let pause_client = EmergencyPauseContractClient::new(&env, &pause_id);
    pause_client.initialize(&council(&env, 1));
    pause_client.pause(&String::from_str(&env, "test"));

    let relay_id = env.register(relay_registry::RelayRegistryContract, ());
    let relay_client = relay_registry::RelayRegistryContractClient::new(&env, &relay_id);
    let mut members = soroban_sdk::Vec::new(&env);
    members.push_back(Address::generate(&env));
    let council = relay_registry::types::AdminCouncil {
        members,
        threshold: 1,
    };
    let token_address = Address::generate(&env);
    let treasury = Address::generate(&env);
    relay_client.initialize(
        &council,
        &token_address,
        &treasury,
        &100i128,
        &10u32,
        &pause_id,
    );
    let node = Address::generate(&env);
    relay_registry::RelayRegistryContractClient::new(&env, &relay_id).register(
        &node,
        &relay_registry::types::NodeMetadata {
            region: String::from_str(&env, "us-east"),
            capacity: 1000,
            uptime_commitment: 99,
        },
    );

    let result = relay_client.try_stake(&node, &150i128);

    assert_eq!(
        result,
        Err(Ok(relay_registry::errors::ContractError::ProtocolPaused))
    );
}

#[test]
fn test_claim_blocked_when_paused() {
    let env = Env::default();
    env.mock_all_auths();
    let pause_id = env.register(EmergencyPauseContract, ());
    let pause_client = EmergencyPauseContractClient::new(&env, &pause_id);
    pause_client.initialize(&council(&env, 1));
    pause_client.pause(&String::from_str(&env, "test"));

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_id = token_contract.address();
    let _token_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_id);

    let fee_id = env.register(fee_distributor::FeeDistributorContract, ());
    let fee_client = fee_distributor::FeeDistributorContractClient::new(&env, &fee_id);
    let mut members = soroban_sdk::Vec::new(&env);
    members.push_back(Address::generate(&env));
    let council = fee_distributor::types::AdminCouncil {
        members,
        threshold: 1,
    };
    let treasury = Address::generate(&env);
    fee_client.initialize(&council, &100u32, &1000u32, &treasury, &token_id, &pause_id);

    let relay = Address::generate(&env);
    let result = fee_client.try_claim(&relay);

    assert_eq!(
        result,
        Err(Ok(fee_distributor::errors::ContractError::ProtocolPaused))
    );
    assert_eq!(
        soroban_sdk::token::Client::new(&env, &token_id).balance(&relay),
        0
    );
}
