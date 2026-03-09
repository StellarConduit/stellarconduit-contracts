//! # Relay Registry — Comprehensive Unit Test Suite
//!
//! Unit tests for the Relay Registry contract covering all public functions
//! with happy paths and error cases. Minimum 80% code coverage required.
//!
//! ## Test Coverage
//! - initialize() - Contract initialization
//! - register() - Node registration with metadata validation
//! - stake() - Token staking and activation logic
//! - unstake() - Token unstaking with lock periods
//! - slash() - Admin slashing of misbehaving nodes
//! - get_node() and is_active() - Node lookup functions

#[cfg(test)]
mod tests {
    use relay_registry::{
        AdminCouncil, ContractError, NodeMetadata, NodeStatus, RelayRegistryContract,
        RelayRegistryContractClient,
    };
    use soroban_sdk::{
        testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
        token, Address, Env, String, Symbol, Vec,
    };

    /// Test setup helper that creates a fresh environment and deployed contract
    fn setup() -> (Env, RelayRegistryContractClient, Address) {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, RelayRegistryContract);
        let client = RelayRegistryContractClient::new(&env, &contract_id);
        
        // Create admin council with single member for testing
        let admin = Address::generate(&env);
        let mut members = Vec::new(&env);
        members.push_back(admin.clone());
        let council = AdminCouncil {
            members,
            threshold: 1,
        };
        
        // Initialize with test parameters
        client.initialize(&council, &100i128, &10u32);
        
        (env, client, admin)
    }

    /// Setup helper that also creates and registers a test node
    fn setup_with_node() -> (Env, RelayRegistryContractClient, Address, Address) {
        let (env, client, admin) = setup();
        let node_address = Address::generate(&env);
        
        // Register a test node
        let metadata = NodeMetadata {
            region: String::from_str(&env, "us-east"),
            capacity: 1000,
            uptime_commitment: 95,
        };
        client.register(&node_address, &metadata);
        
        (env, client, admin, node_address)
    }

    // ==================== initialize() Tests ====================

    #[test]
    fn test_initialize_success() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, RelayRegistryContract);
        let client = RelayRegistryContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let mut members = Vec::new(&env);
        members.push_back(admin.clone());
        let council = AdminCouncil {
            members,
            threshold: 1,
        };
        
        // Test successful initialization
        let result = client.try_initialize(&council, &100i128, &10u32);
        assert_eq!(result, Ok(()));
        
        // Verify initialization was successful by trying to register a node
        let node_address = Address::generate(&env);
        let metadata = NodeMetadata {
            region: String::from_str(&env, "test"),
            capacity: 1000,
            uptime_commitment: 95,
        };
        let result = client.try_register(&node_address, &metadata);
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn test_initialize_already_initialized() {
        let (env, client, _) = setup();
        
        let admin = Address::generate(&env);
        let mut members = Vec::new(&env);
        members.push_back(admin);
        let council = AdminCouncil {
            members,
            threshold: 1,
        };
        
        // Test that second initialization fails
        let result = client.try_initialize(&council, &200i128, &20u32);
        assert_eq!(result, Err(ContractError::AlreadyInitialized));
    }

    #[test]
    fn test_initialize_invalid_amount_zero_stake() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, RelayRegistryContract);
        let client = RelayRegistryContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let mut members = Vec::new(&env);
        members.push_back(admin);
        let council = AdminCouncil {
            members,
            threshold: 1,
        };
        
        // Test that zero min_stake fails
        let result = client.try_initialize(&council, &0i128, &10u32);
        assert_eq!(result, Err(ContractError::InvalidAmount));
    }

    #[test]
    fn test_initialize_invalid_amount_zero_lock_period() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, RelayRegistryContract);
        let client = RelayRegistryContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let mut members = Vec::new(&env);
        members.push_back(admin);
        let council = AdminCouncil {
            members,
            threshold: 1,
        };
        
        // Test that zero lock period fails
        let result = client.try_initialize(&council, &100i128, &0u32);
        assert_eq!(result, Err(ContractError::InvalidAmount));
    }

    #[test]
    fn test_initialize_invalid_council_config() {
        let env = Env::default();
        env.mock_all_auths();
        
        let contract_id = env.register_contract(None, RelayRegistryContract);
        let client = RelayRegistryContractClient::new(&env, &contract_id);
        
        let admin = Address::generate(&env);
        let mut members = Vec::new(&env);
        members.push_back(admin);
        let council = AdminCouncil {
            members,
            threshold: 2, // Threshold > members count
        };
        
        // Test that invalid council config fails
        let result = client.try_initialize(&council, &100i128, &10u32);
        assert_eq!(result, Err(ContractError::InvalidCouncilConfig));
    }

    // ==================== register() Tests ====================

    #[test]
    fn test_register_success() {
        let (env, client, _) = setup();
        let node_address = Address::generate(&env);
        
        let metadata = NodeMetadata {
            region: String::from_str(&env, "us-west"),
            capacity: 500,
            uptime_commitment: 99,
        };
        
        // Test successful registration
        let result = client.try_register(&node_address, &metadata);
        assert_eq!(result, Ok(()));
        
        // Verify node was stored correctly
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.address, node_address);
        assert_eq!(node.status, NodeStatus::Inactive);
        assert_eq!(node.stake, 0);
        assert_eq!(node.metadata.region, String::from_str(&env, "us-west"));
        assert_eq!(node.metadata.capacity, 500);
        assert_eq!(node.metadata.uptime_commitment, 99);
    }

    #[test]
    fn test_register_already_registered() {
        let (env, client, _, node_address) = setup_with_node();
        
        let metadata = NodeMetadata {
            region: String::from_str(&env, "eu-central"),
            capacity: 2000,
            uptime_commitment: 98,
        };
        
        // Test that duplicate registration fails
        let result = client.try_register(&node_address, &metadata);
        assert_eq!(result, Err(ContractError::AlreadyRegistered));
    }

    #[test]
    fn test_register_invalid_metadata_uptime_too_high() {
        let (env, client, _) = setup();
        let node_address = Address::generate(&env);
        
        let metadata = NodeMetadata {
            region: String::from_str(&env, "asia-pacific"),
            capacity: 1500,
            uptime_commitment: 101, // Invalid: > 100
        };
        
        // Test that invalid metadata fails
        let result = client.try_register(&node_address, &metadata);
        assert_eq!(result, Err(ContractError::InvalidMetadata));
    }

    // ==================== stake() Tests ====================

    #[test]
    fn test_stake_success_below_minimum() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Stake below minimum (100)
        let result = client.try_stake(&node_address, &50i128);
        assert_eq!(result, Ok(()));
        
        // Verify stake was added but status remains Inactive
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.stake, 50);
        assert_eq!(node.status, NodeStatus::Inactive);
    }

    #[test]
    fn test_stake_success_reaches_minimum() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Stake exactly minimum amount
        let result = client.try_stake(&node_address, &100i128);
        assert_eq!(result, Ok(()));
        
        // Verify node became Active
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.stake, 100);
        assert_eq!(node.status, NodeStatus::Active);
    }

    #[test]
    fn test_stake_not_registered() {
        let (env, client, _) = setup();
        let unregistered_address = Address::generate(&env);
        
        // Test that staking for unregistered node fails
        let result = client.try_stake(&unregistered_address, &50i128);
        assert_eq!(result, Err(ContractError::NotRegistered));
    }

    #[test]
    fn test_stake_node_slashed() {
        let (env, client, admin, node_address) = setup_with_node();
        
        // First slash the node
        client.slash(&node_address, &String::from_str(&env, "test slash"));
        
        // Then try to stake
        let result = client.try_stake(&node_address, &50i128);
        assert_eq!(result, Err(ContractError::NodeSlashed));
    }

    #[test]
    fn test_stake_zero_amount() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Test that zero amount fails
        let result = client.try_stake(&node_address, &0i128);
        assert_eq!(result, Err(ContractError::InsufficientStake));
    }

    #[test]
    fn test_stake_negative_amount() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Test that negative amount fails
        let result = client.try_stake(&node_address, &-10i128);
        assert_eq!(result, Err(ContractError::InsufficientStake));
    }

    // ==================== unstake() Tests ====================

    #[test]
    fn test_unstake_success() {
        let (env, client, _, node_address) = setup_with_node();
        
        // First stake enough to become active
        client.stake(&node_address, &150i128);
        
        // Then unstake some amount
        let result = client.try_unstake(&node_address, &50i128);
        assert_eq!(result, Ok(()));
        
        // Verify stake was reduced and status remains Active (still above minimum)
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.stake, 100); // 150 - 50
        assert_eq!(node.status, NodeStatus::Active); // Still above minimum
    }

    #[test]
    fn test_unstake_not_registered() {
        let (env, client, _) = setup();
        let unregistered_address = Address::generate(&env);
        
        // Test that unstaking for unregistered node fails
        let result = client.try_unstake(&unregistered_address, &50i128);
        assert_eq!(result, Err(ContractError::NotRegistered));
    }

    #[test]
    fn test_unstake_node_slashed() {
        let (env, client, admin, node_address) = setup_with_node();
        
        // First slash the node
        client.slash(&node_address, &String::from_str(&env, "test slash"));
        
        // Then try to unstake
        let result = client.try_unstake(&node_address, &50i128);
        assert_eq!(result, Err(ContractError::NodeSlashed));
    }

    #[test]
    fn test_unstake_amount_exceeds_stake() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Stake some amount first
        client.stake(&node_address, &100i128);
        
        // Try to unstake more than available
        let result = client.try_unstake(&node_address, &150i128);
        assert_eq!(result, Err(ContractError::InsufficientStake));
    }

    #[test]
    fn test_unstake_zero_amount() {
        let (env, client, _, node_address) = setup_with_node();
        
        // First stake to become active
        client.stake(&node_address, &150i128);
        
        // Test that zero amount fails
        let result = client.try_unstake(&node_address, &0i128);
        assert_eq!(result, Err(ContractError::InsufficientStake));
    }

    #[test]
    fn test_unstake_node_not_active() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Try to unstake without being active
        let result = client.try_unstake(&node_address, &50i128);
        assert_eq!(result, Err(ContractError::NodeNotActive));
    }

    // ==================== slash() Tests ====================

    #[test]
    fn test_slash_success() {
        let (env, client, admin, node_address) = setup_with_node();
        
        // First stake some amount
        client.stake(&node_address, &150i128);
        
        // Slash the node
        let result = client.try_slash(&node_address, &String::from_str(&env, "misbehavior"));
        assert_eq!(result, Ok(()));
        
        // Verify stake was zeroed and status set to Slashed
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.stake, 0);
        assert_eq!(node.status, NodeStatus::Slashed);
    }

    #[test]
    fn test_slash_not_registered() {
        let (env, client, admin) = setup();
        let unregistered_address = Address::generate(&env);
        
        // Test that slashing unregistered node fails
        let result = client.try_slash(&unregistered_address, &String::from_str(&env, "test"));
        assert_eq!(result, Err(ContractError::NotRegistered));
    }

    #[test]
    fn test_slash_already_slashed() {
        let (env, client, admin, node_address) = setup_with_node();
        
        // First slash the node
        client.slash(&node_address, &String::from_str(&env, "first slash"));
        
        // Then try to slash again
        let result = client.try_slash(&node_address, &String::from_str(&env, "second slash"));
        assert_eq!(result, Err(ContractError::NodeSlashed));
    }

    #[test]
    fn test_slash_unauthorized() {
        let (env, client, _) = setup();
        let node_address = Address::generate(&env);
        
        // Register a node first
        let metadata = NodeMetadata {
            region: String::from_str(&env, "test-region"),
            capacity: 1000,
            uptime_commitment: 95,
        };
        client.register(&node_address, &metadata);
        
        // Remove auth mock to test authorization
        env.clear_all_auths();
        
        // Try to slash without admin authorization - should panic due to require_auth
        let result = std::panic::catch_unwind(|| {
            client.try_slash(&node_address, &String::from_str(&env, "unauthorized slash"))
        });
        
        // Should panic due to failed authorization
        assert!(result.is_err());
    }

    // ==================== get_node() Tests ====================

    #[test]
    fn test_get_node_success() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Test successful node retrieval
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.address, node_address);
        assert_eq!(node.status, NodeStatus::Inactive);
        assert_eq!(node.stake, 0);
        assert_eq!(node.metadata.region, String::from_str(&env, "us-east"));
        assert_eq!(node.metadata.capacity, 1000);
        assert_eq!(node.metadata.uptime_commitment, 95);
    }

    #[test]
    fn test_get_node_not_found() {
        let (env, client, _) = setup();
        let unknown_address = Address::generate(&env);
        
        // Test that getting unknown node fails
        let result = client.try_get_node(&unknown_address);
        assert_eq!(result, Err(ContractError::NotRegistered));
    }

    // ==================== is_active() Tests ====================

    #[test]
    fn test_is_active_true() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Stake enough to become active
        client.stake(&node_address, &150i128);
        
        // Test that active node returns true
        let is_active = client.is_active(&node_address);
        assert!(is_active);
    }

    #[test]
    fn test_is_active_false_inactive() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Test that inactive node returns false
        let is_active = client.is_active(&node_address);
        assert!(!is_active);
    }

    #[test]
    fn test_is_active_false_unknown() {
        let (env, client, _) = setup();
        let unknown_address = Address::generate(&env);
        
        // Test that unknown node returns false
        let is_active = client.is_active(&unknown_address);
        assert!(!is_active);
    }

    #[test]
    fn test_is_active_false_slashed() {
        let (env, client, admin, node_address) = setup_with_node();
        
        // First make node active
        client.stake(&node_address, &150i128);
        
        // Then slash it
        client.slash(&node_address, &String::from_str(&env, "test slash"));
        
        // Test that slashed node returns false
        let is_active = client.is_active(&node_address);
        assert!(!is_active);
    }

    // ==================== Additional Helper Tests ====================

    #[test]
    fn test_node_count_increment() {
        let (env, client, _) = setup();
        
        // Register multiple nodes and verify they can be retrieved
        let node1 = Address::generate(&env);
        let node2 = Address::generate(&env);
        let node3 = Address::generate(&env);
        
        let metadata = NodeMetadata {
            region: String::from_str(&env, "test"),
            capacity: 1000,
            uptime_commitment: 95,
        };
        
        client.register(&node1, &metadata);
        assert!(client.get_node(&node1).is_ok());
        
        client.register(&node2, &metadata);
        assert!(client.get_node(&node2).is_ok());
        
        client.register(&node3, &metadata);
        assert!(client.get_node(&node3).is_ok());
    }

    #[test]
    fn test_stake_activates_node_at_threshold() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Stake just below minimum
        client.stake(&node_address, &99i128);
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.status, NodeStatus::Inactive);
        
        // Stake 1 more to reach minimum
        client.stake(&node_address, &1i128);
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.status, NodeStatus::Active);
        assert_eq!(node.stake, 100);
    }

    #[test]
    fn test_unstake_deactivates_node_below_threshold() {
        let (env, client, _, node_address) = setup_with_node();
        
        // Stake above minimum
        client.stake(&node_address, &150i128);
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.status, NodeStatus::Active);
        
        // Unstake below minimum
        client.unstake(&node_address, &60i128); // Leaves 90, below 100
        let node = client.get_node(&node_address).unwrap();
        assert_eq!(node.status, NodeStatus::Inactive);
        assert_eq!(node.stake, 90);
    }
}
