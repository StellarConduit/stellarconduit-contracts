//! Targeted M-of-N council auth tests for the Relay Registry contract.
//!
//! These tests use `env.mock_auths()` with specific subsets of council members
//! to verify that the threshold logic is real and position-independent.

extern crate std;

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String,
};

use crate::{
    errors::ContractError,
    storage,
    types::{AdminCouncil, NodeMetadata, NodeStatus, RelayNode},
    RelayRegistryContract, RelayRegistryContractClient,
};

/// Register a contract with a 3-member council [alice, bob, carol] at the given threshold.
/// Seeds a dummy registered + active node that can be slashed.
fn setup_council<'a>(
    env: &'a Env,
    alice: &Address,
    bob: &Address,
    carol: &Address,
    threshold: u32,
    node: &Address,
) -> RelayRegistryContractClient<'a> {
    let contract_id = env.register(RelayRegistryContract, ());
    let client = RelayRegistryContractClient::new(env, &contract_id);

    let mut members = soroban_sdk::Vec::new(env);
    members.push_back(alice.clone());
    members.push_back(bob.clone());
    members.push_back(carol.clone());

    let council = AdminCouncil { members, threshold };

    // Initialize without token — we'll inject the node directly via as_contract.
    env.as_contract(&contract_id, || {
        storage::set_admin_council(env, &council);
        storage::set_min_stake(env, 1000);
        storage::set_stake_lock_period(env, 100);
        storage::set_node_count(env, 1);

        let relay = RelayNode {
            address: node.clone(),
            stake: 5000,
            status: NodeStatus::Active,
            metadata: NodeMetadata {
                region: String::from_str(env, "west"),
                capacity: 100,
                uptime_commitment: 99,
            },
            registered_at: 0,
            last_active: 0,
        };
        storage::set_node(env, node, &relay);
    });

    client
}

fn slash_mock_auth<'a>(
    env: &'a Env,
    contract_id: &Address,
    signer: &'a Address,
    node: &'a Address,
) -> MockAuth<'a> {
    MockAuth {
        address: signer,
        invoke: &MockAuthInvoke {
            contract: contract_id,
            fn_name: "slash",
            args: (node.clone(), String::from_str(env, "test")).into_val(env),
            sub_invokes: &[],
        },
    }
}

// ── Acceptance-criteria tests ────────────────────────────────────────────────

/// A 1-of-3 council accepts authorization from any single member, not just Alice.
#[test]
fn test_threshold_1_of_3_carol_can_authorize() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let node = Address::generate(&env);

    let client = setup_council(&env, &alice, &bob, &carol, 1, &node);

    // Only Carol signs — Carol is position 3, not first.
    env.mock_auths(&[slash_mock_auth(&env, &client.address, &carol, &node)]);

    let result = client.try_slash(&node, &String::from_str(&env, "misbehaving"));
    assert_eq!(result, Ok(Ok(())));
}

/// A 2-of-3 council rejects a transaction signed by only one member.
#[test]
fn test_threshold_2_of_3_single_sig_rejected() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let node = Address::generate(&env);

    let client = setup_council(&env, &alice, &bob, &carol, 2, &node);

    // Only Alice signs — threshold is 2, should be rejected.
    env.mock_auths(&[slash_mock_auth(&env, &client.address, &alice, &node)]);

    let result = client.try_slash(&node, &String::from_str(&env, "misbehaving"));
    assert_eq!(result, Err(Ok(ContractError::InsufficientApprovals)));
}

/// A 2-of-3 council accepts Bob + Carol (neither is first in the Vec).
#[test]
fn test_threshold_2_of_3_bob_and_carol_succeed() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let node = Address::generate(&env);

    let client = setup_council(&env, &alice, &bob, &carol, 2, &node);

    // Bob and Carol sign — Alice is first in Vec but does NOT sign.
    env.mock_auths(&[
        slash_mock_auth(&env, &client.address, &bob, &node),
        slash_mock_auth(&env, &client.address, &carol, &node),
    ]);

    let result = client.try_slash(&node, &String::from_str(&env, "misbehaving"));
    assert_eq!(result, Ok(Ok(())));
}

/// Exactly threshold signatures (2-of-3) from any combination succeeds.
#[test]
fn test_threshold_2_of_3_alice_and_carol_succeed() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let node = Address::generate(&env);

    let client = setup_council(&env, &alice, &bob, &carol, 2, &node);

    // Alice and Carol sign (not contiguous with Bob skipped).
    env.mock_auths(&[
        slash_mock_auth(&env, &client.address, &alice, &node),
        slash_mock_auth(&env, &client.address, &carol, &node),
    ]);

    let result = client.try_slash(&node, &String::from_str(&env, "misbehaving"));
    assert_eq!(result, Ok(Ok(())));
}

/// Zero signatures always fails regardless of threshold.
#[test]
fn test_zero_signatures_rejected() {
    let env = Env::default();
    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    let node = Address::generate(&env);

    let client = setup_council(&env, &alice, &bob, &carol, 1, &node);

    // No mock auths at all — no one signed.
    let result = client.try_slash(&node, &String::from_str(&env, "misbehaving"));
    assert_eq!(result, Err(Ok(ContractError::InsufficientApprovals)));
}
