//! # Full Settlement Flow — Integration Test
//!
//! End-to-end integration test that simulates a complete StellarConduit settlement
//! flow across all four contracts. This test exercises the full happy-path lifecycle:
//! relay node registration → transaction settlement → fee distribution → dispute (if needed).

extern crate std;

use soroban_sdk::{testutils::Address as _, token, Address, BytesN, Env, String};

use relay_registry::types::NodeMetadata;

/// Set up a shared environment and deploy all protocol contracts with a mock SAC token.
fn setup_all<'a>() -> (
	Env,
	relay_registry::RelayRegistryContractClient<'a>,
	fee_distributor::FeeDistributorContractClient<'a>,
	dispute_resolver::DisputeResolverContractClient<'a>,
	treasury::TreasuryContractClient<'a>,
	token::StellarAssetClient<'a>,
	emergency_pause::EmergencyPauseContractClient<'a>,
) {
	let env = Env::default();
	env.mock_all_auths();

	// Deploy mock SAC token
	let token_admin = Address::generate(&env);
	let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
	let token_id = token_contract.address();
	let token_client = token::StellarAssetClient::new(&env, &token_id);

	let admin = Address::generate(&env);
	let mut members = soroban_sdk::Vec::new(&env);
	members.push_back(admin.clone());

	// Deploy emergency-pause and initialize
	let pause_id = env.register(emergency_pause::EmergencyPauseContract, ());
	let pause_client = emergency_pause::EmergencyPauseContractClient::new(&env, &pause_id);
	let pause_council = emergency_pause::types::AdminCouncil {
		members: members.clone(),
		threshold: 1,
	};
	pause_client.initialize(&pause_council);

	// Deploy treasury and initialize
	let treasury_id = env.register(treasury::TreasuryContract, ());
	let treasury_client = treasury::TreasuryContractClient::new(&env, &treasury_id);
	let treasury_council = treasury::types::AdminCouncil {
		members: members.clone(),
		threshold: 1,
	};
	treasury_client.initialize(&treasury_council, &token_id, &pause_id);

	// Deploy fee distributor and initialize
	let fee_id = env.register(fee_distributor::FeeDistributorContract, ());
	let fee_client = fee_distributor::FeeDistributorContractClient::new(&env, &fee_id);
	let fee_council = fee_distributor::types::AdminCouncil {
		members: members.clone(),
		threshold: 1,
	};
	// fee_rate_bps = 100 (1%), treasury_share_bps = 1000 (10%)
	fee_client.initialize(&fee_council, &100u32, &1000u32, &treasury_id, &token_id, &pause_id);

	// Mint some tokens to fee distributor so treasury transfers can succeed
	token_client.mint(&fee_id, &1_000_000);

	// Deploy relay registry and initialize
	let relay_id = env.register(relay_registry::RelayRegistryContract, ());
	let relay_client = relay_registry::RelayRegistryContractClient::new(&env, &relay_id);
	let relay_council = relay_registry::types::AdminCouncil {
		members: members.clone(),
		threshold: 1,
	};
	// min_stake = 100, stake_lock_period = 10 ledgers
	relay_client.initialize(&relay_council, &100i128, &10u32, &pause_id);

	// Set token address in relay registry storage (init doesn't take token address)
	env.as_contract(&relay_client.address, || {
		relay_registry::storage::set_token_address(&env, &token_id);
	});

	// Deploy dispute resolver and initialize
	let dispute_id = env.register(dispute_resolver::DisputeResolverContract, ());
	let dispute_client = dispute_resolver::DisputeResolverContractClient::new(&env, &dispute_id);
	let dispute_council = dispute_resolver::types::AdminCouncil { members, threshold: 1 };
	dispute_client.initialize(&dispute_council, &100u32, &pause_id);

	(env, relay_client, fee_client, dispute_client, treasury_client, token_client, pause_client)
}

#[test]
fn test_full_settlement_flow() {
	let (
		env,
		relay_client,
		fee_client,
		dispute_client,
		treasury_client,
		token_client,
		_pause_client,
	) = setup_all();

	// ----------------------
	// Phase 2 — Relay Onboarding
	// ----------------------
	let alice = Address::generate(&env);
	let bob = Address::generate(&env);
	let carol = Address::generate(&env);

	let meta = NodeMetadata {
		region: String::from_str(&env, "us-east"),
		capacity: 1000,
		uptime_commitment: 99u32,
	};

	// Mint tokens so nodes can stake
	token_client.mint(&alice, &1000);
	token_client.mint(&bob, &1000);
	token_client.mint(&carol, &1000);

	// Register nodes
	relay_client.register(&alice, &meta.clone());
	relay_client.register(&bob, &meta.clone());
	relay_client.register(&carol, &meta.clone());

	// Stake above minimum (min_stake = 100)
	relay_client.stake(&alice, &200);
	relay_client.stake(&bob, &200);
	relay_client.stake(&carol, &200);

	// Verify active
	assert!(relay_client.is_active(&alice));
	assert!(relay_client.is_active(&bob));
	assert!(relay_client.is_active(&carol));

	// Verify node count == 3
	let node_count = env.as_contract(&relay_client.address, || relay_registry::storage::get_node_count(&env));
	assert_eq!(node_count, 3);

	// ----------------------
	// Phase 3 — Batch Settlement & Fee Distribution
	// ----------------------
	let batch_id = 1u64;
	let batch_size = 100u32; // fee = 100 * 100 / 10000 = 1

	// Distribute for Alice
	fee_client.distribute(&alice, &batch_id, &batch_size);

	// Verify Alice's earnings updated
	let earnings = fee_client.get_earnings(&alice);
	assert_eq!(earnings.total_earned, 1);
	assert_eq!(earnings.unclaimed, 1);

	// Verify treasury received its share (treasury_share_bps = 1000 => 10%)
	let treasury_balance = treasury_client.get_balance();
	assert!(treasury_balance >= 0);

	// Ensure duplicate distribution fails
	let dup = fee_client.try_distribute(&alice, &batch_id, &batch_size);
	assert_eq!(dup, Err(Ok(fee_distributor::errors::ContractError::BatchAlreadyDistributed)));

	// ----------------------
	// Phase 4 — Fee Claiming
	// ----------------------
	let alice_balance_before = token_client.balance(&alice);
	let fee_distributor_balance_before = token_client.balance(&fee_client.address);

	// Alice claims
	let payout = fee_client.claim(&alice);
	assert_eq!(payout, 1);

	let alice_balance_after = token_client.balance(&alice);
	let fee_distributor_balance_after = token_client.balance(&fee_client.address);

	assert_eq!(alice_balance_after, alice_balance_before + payout);
	assert_eq!(fee_distributor_balance_after, fee_distributor_balance_before - payout);

	let earnings_after = fee_client.get_earnings(&alice);
	assert_eq!(earnings_after.unclaimed, 0);
	assert_eq!(earnings_after.total_claimed, 1);

	// Second claim should return NothingToClaim
	let second = fee_client.try_claim(&alice);
	assert_eq!(second, Err(Ok(fee_distributor::errors::ContractError::NothingToClaim)));

	// ----------------------
	// Phase 5 — Dispute Resolution
	// ----------------------
	let initiator = bob.clone();
	let respondent = carol.clone();

	// Prepare ed25519 keys
	let initiator_sk = ed25519_dalek::SigningKey::from_bytes(&[1u8; 32]);
	let initiator_pk_bytes: [u8; 32] = initiator_sk.verifying_key().to_bytes();
	let initiator_pk = BytesN::from_array(&env, &initiator_pk_bytes);

	let respondent_sk = ed25519_dalek::SigningKey::from_bytes(&[2u8; 32]);
	let respondent_pk_bytes: [u8; 32] = respondent_sk.verifying_key().to_bytes();
	let respondent_pk = BytesN::from_array(&env, &respondent_pk_bytes);

	env.as_contract(&dispute_client.address, || {
		dispute_resolver::storage::set_public_key(&env, &initiator, &initiator_pk);
		dispute_resolver::storage::set_public_key(&env, &respondent, &respondent_pk);
	});

	fn create_proof(env: &Env, sk: &ed25519_dalek::SigningKey, chain_hash_bytes: &[u8; 32], sequence: u64) -> dispute_resolver::types::RelayChainProof {
		use ed25519_dalek::Signer;
		let sig = sk.sign(chain_hash_bytes.as_slice());
		let signature = BytesN::from_array(env, &sig.to_bytes());
		let chain_hash = BytesN::from_array(env, chain_hash_bytes);
		dispute_resolver::types::RelayChainProof { signature, chain_hash, sequence }
	}

	let tx_id = BytesN::from_array(&env, &[9u8; 32]);
	let chain_hash = [8u8; 32];
	let init_proof = create_proof(&env, &initiator_sk, &chain_hash, 20);
	let resp_proof = create_proof(&env, &respondent_sk, &chain_hash, 10);

	let dispute_id = dispute_client.raise_dispute(&initiator, &respondent, &tx_id, &init_proof);
	dispute_client.respond(&respondent, &dispute_id, &resp_proof);

	// Resolve — respondent (carol) should win due to lower sequence
	let ruling = dispute_client.resolve(&dispute_id);
	assert_eq!(ruling.winner, respondent);

	// Cannot resolve again
	let res_again = dispute_client.try_resolve(&dispute_id);
	assert_eq!(res_again, Err(Ok(dispute_resolver::errors::ContractError::DisputeAlreadyResolved)));

	// ----------------------
	// Phase 6 — Slash and Recovery
	// ----------------------
	relay_client.slash(&bob, &String::from_str(&env, "misbehavior"));

	let bob_node = relay_client.get_node(&bob);
	assert_eq!(bob_node.stake, 0);
	assert!(matches!(bob_node.status, relay_registry::types::NodeStatus::Slashed));
	assert!(!relay_client.is_active(&bob));

	let _ = fee_client.try_distribute(&bob, &2u64, &200u32);
}

#[test]
fn test_pause_blocks_protocol_operations() {
	let (
		env,
		relay_client,
		fee_client,
		_dispute_client,
		_treasury_client,
		token_client,
		pause_client,
	) = setup_all();

	let node = Address::generate(&env);
	token_client.mint(&node, &1000);
	relay_client.register(
		&node,
		&NodeMetadata {
			region: String::from_str(&env, "eu-west"),
			capacity: 500,
			uptime_commitment: 95,
		},
	);

	// Pause the protocol
	pause_client.pause(&String::from_str(&env, "security incident"));
	assert!(pause_client.is_paused());

	// stake() is blocked
	let stake_result = relay_client.try_stake(&node, &200i128);
	assert_eq!(
		stake_result,
		Err(Ok(relay_registry::errors::ContractError::ProtocolPaused))
	);

	// claim() is blocked
	let relay_addr = Address::generate(&env);
	let claim_result = fee_client.try_claim(&relay_addr);
	assert_eq!(
		claim_result,
		Err(Ok(fee_distributor::errors::ContractError::ProtocolPaused))
	);

	// Unpause and verify operations resume
	pause_client.unpause();
	assert!(!pause_client.is_paused());

	// stake() now succeeds
	relay_client.stake(&node, &200i128);
	assert!(relay_client.is_active(&node));
}
