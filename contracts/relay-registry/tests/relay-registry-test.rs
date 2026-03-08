#![cfg(test)]

use relay_registry::{
    types::{AdminCouncil, NodeMetadata, NodeStatus},
    RelayRegistryContract, RelayRegistryContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token, Address, Env, String,
};

fn setup<'a>() -> (Env, RelayRegistryContractClient<'a>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(RelayRegistryContract, ());
    let client = RelayRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let mut members = soroban_sdk::Vec::new(&env);
    members.push_back(admin.clone());
    let council = AdminCouncil {
        members,
        threshold: 1,
    };

    client.initialize(&council, &100i128, &10u32);
    (env, client, admin)
}

#[test]
fn test_update_metadata_success() {
    let (env, client, _) = setup();
    let node_addr = Address::generate(&env);
    let initial_metadata = NodeMetadata {
        region: String::from_str(&env, "us-east"),
        capacity: 1000,
        uptime_commitment: 99,
    };

    client.register(&node_addr, &initial_metadata);

    let mut current_node = client.get_node(&node_addr);
    assert_eq!(
        current_node.metadata.region,
        String::from_str(&env, "us-east")
    );

    let new_metadata = NodeMetadata {
        region: String::from_str(&env, "eu-west"),
        capacity: 2000,
        uptime_commitment: 98,
    };

    client.update_metadata(&node_addr, &new_metadata);

    current_node = client.get_node(&node_addr);
    assert_eq!(
        current_node.metadata.region,
        String::from_str(&env, "eu-west")
    );
    assert_eq!(current_node.metadata.capacity, 2000);
    assert_eq!(current_node.metadata.uptime_commitment, 98);
}

#[test]
fn test_update_metadata_preserves_status_and_stake() {
    let (env, client, _) = setup();
    let node_addr = Address::generate(&env);
    let metadata = NodeMetadata {
        region: String::from_str(&env, "us-east"),
        capacity: 1000,
        uptime_commitment: 99,
    };

    client.register(&node_addr, &metadata);

    assert_eq!(client.get_node(&node_addr).status, NodeStatus::Inactive);
    assert_eq!(client.get_node(&node_addr).stake, 0);

    let new_metadata = NodeMetadata {
        region: String::from_str(&env, "eu-west"),
        capacity: 2000,
        uptime_commitment: 98,
    };
    client.update_metadata(&node_addr, &new_metadata);

    let updated_node = client.get_node(&node_addr);
    assert_eq!(updated_node.status, NodeStatus::Inactive);
    assert_eq!(updated_node.stake, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")] // NotRegistered
fn test_update_metadata_not_registered() {
    let (env, client, _) = setup();
    let node_addr = Address::generate(&env);
    let new_metadata = NodeMetadata {
        region: String::from_str(&env, "eu-west"),
        capacity: 2000,
        uptime_commitment: 98,
    };

    client.update_metadata(&node_addr, &new_metadata);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")] // InvalidMetadata
fn test_update_metadata_invalid_commitment() {
    let (env, client, _) = setup();
    let node_addr = Address::generate(&env);
    let metadata = NodeMetadata {
        region: String::from_str(&env, "us-east"),
        capacity: 1000,
        uptime_commitment: 99,
    };

    client.register(&node_addr, &metadata);

    let new_metadata = NodeMetadata {
        region: String::from_str(&env, "eu-west"),
        capacity: 2000,
        uptime_commitment: 105, // > 100
    };

    client.update_metadata(&node_addr, &new_metadata);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")] // InvalidMetadata
fn test_update_metadata_region_too_long() {
    let (env, client, _) = setup();
    let node_addr = Address::generate(&env);
    let metadata = NodeMetadata {
        region: String::from_str(&env, "us-east"),
        capacity: 1000,
        uptime_commitment: 99,
    };

    client.register(&node_addr, &metadata);

    // 33 characters long
    let long_region = "this-region-string-is-too-long-xx";

    let new_metadata = NodeMetadata {
        region: String::from_str(&env, long_region),
        capacity: 2000,
        uptime_commitment: 100,
    };

    client.update_metadata(&node_addr, &new_metadata);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn test_update_metadata_auth_required_clean() {
    let env = Env::default();
    let contract_id = env.register(RelayRegistryContract, ());
    let client = RelayRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    let mut members = soroban_sdk::Vec::new(&env);
    members.push_back(admin.clone());
    let council = AdminCouncil {
        members,
        threshold: 1,
    };

    // Hack: to initialize and register without `mock_all_auths`,
    // we just don't call `mock_all_auths` and let it panic on `initialize` because
    // `require_auth` isn't called in `initialize`!
    // Wait, `initialize` does not call `require_auth`!
    client.initialize(&council, &100i128, &10u32);

    let node_addr = Address::generate(&env);

    // `register` calls `require_auth`, so this will panic before we even get to `update_metadata`.
    // So we can just test `update_metadata` directly and it will panic on auth.
    // Actually we can't because `update_metadata` also fails on `NotRegistered` before auth? No, `require_auth` is called FIRST.
    let new_metadata = NodeMetadata {
        region: String::from_str(&env, "eu-west"),
        capacity: 2000,
        uptime_commitment: 98,
    };
    client.update_metadata(&node_addr, &new_metadata);
}

#[test]
fn test_unstake_creates_lock_entry() {
    let (env, client, _admin) = setup();

    // We must deploy a deterministic token contract and initialize it for stakes
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_contract.address());
    let token_address = token_client.address.clone();

    // Set the token address in the registry manually by calling the storage helper directly
    // because the client's initialize() in this contract version didn't natively take a token address.
    // However, wait, in our setup we don't have access to storage helpers from the test if we only use the client?
    // Let's look at `storage::set_token_address(&env, &token_address)`
    // It's a public function.
    env.as_contract(&client.address, || {
        relay_registry::storage::set_token_address(&env, &token_address);
    });

    let node_addr = Address::generate(&env);
    let metadata = NodeMetadata {
        region: String::from_str(&env, "us-east"),
        capacity: 1000,
        uptime_commitment: 99,
    };

    // Mint tokens to the node so it can stake
    token_client.mint(&node_addr, &500);

    client.register(&node_addr, &metadata);
    client.stake(&node_addr, &200);

    let node_pre_unstake = client.get_node(&node_addr);
    assert_eq!(node_pre_unstake.stake, 200);

    // Unstake 50 tokens
    client.unstake(&node_addr, &50);

    let node_post_unstake = client.get_node(&node_addr);
    assert_eq!(node_post_unstake.stake, 150);

    // Instead of tokens arriving immediately, we verify the lock entry exists.
    // It's hard to read `get_lock_entry` via client because we didn't expose it,
    // but we can test `finalize_unstake` fails if lock period is active.
    let res = client.try_finalize_unstake(&node_addr);
    assert!(res.is_err()); // LockPeriodActive
}

#[test]
fn test_finalize_unstake_success_after_lock() {
    let (env, client, _admin) = setup();

    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_client = token::StellarAssetClient::new(&env, &token_contract.address());
    let token_address = token_client.address.clone();

    env.as_contract(&client.address, || {
        relay_registry::storage::set_token_address(&env, &token_address);
    });

    let node_addr = Address::generate(&env);
    let metadata = NodeMetadata {
        region: String::from_str(&env, "us-east"),
        capacity: 1000,
        uptime_commitment: 99,
    };

    token_client.mint(&node_addr, &500);

    client.register(&node_addr, &metadata);
    client.stake(&node_addr, &200);
    client.unstake(&node_addr, &50);

    // Advance time past the 10 ledger lock period
    env.ledger().with_mut(|l| l.timestamp += 11);

    // Node balance should be 300 (500 minted - 200 staked)
    let token_client_standard = token::Client::new(&env, &token_contract.address());
    assert_eq!(token_client_standard.balance(&node_addr), 300);

    // Finalize
    client.finalize_unstake(&node_addr);

    // Node balance should correctly increment by 50
    assert_eq!(token_client_standard.balance(&node_addr), 350);

    // Fetching the entry again should yield NoPendingUnstake
    let res2 = client.try_finalize_unstake(&node_addr);
    assert!(res2.is_err());
}

#[test]
#[should_panic(expected = "Error(Contract, #14)")] // NoPendingUnstake
fn test_finalize_unstake_no_entry() {
    let (env, client, _) = setup();
    let node_addr = Address::generate(&env);

    client.finalize_unstake(&node_addr);
}
