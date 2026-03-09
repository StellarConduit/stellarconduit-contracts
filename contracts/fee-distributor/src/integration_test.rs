//! # Fee Distributor — Integration Tests
//!
//! Tests cross-contract relay validation between fee-distributor and relay-registry.
//! Both contracts are deployed in the same Soroban test environment.

#[cfg(test)]
extern crate std;

// ── Mock contracts in separate submodules to avoid symbol collisions ──────────

pub mod mock_active {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    /// Mock relay-registry that always reports nodes as Active.
    #[contract]
    pub struct MockActiveRegistry;

    #[contractimpl]
    impl MockActiveRegistry {
        /// Always returns true — simulates an Active relay node.
        pub fn is_active(_env: Env, _address: Address) -> bool {
            true
        }
    }
}

pub mod mock_inactive {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    /// Mock relay-registry that always reports nodes as Inactive or Slashed.
    #[contract]
    pub struct MockInactiveRegistry;

    #[contractimpl]
    impl MockInactiveRegistry {
        /// Always returns false — simulates an Inactive or Slashed relay node.
        pub fn is_active(_env: Env, _address: Address) -> bool {
            false
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::Address as _, Address, Env};

    use crate::integration_test::{
        mock_active::MockActiveRegistry,
        mock_inactive::MockInactiveRegistry,
    };
    use crate::{FeeDistributorContract, FeeDistributorContractClient};

    // ── helper ────────────────────────────────────────────────────────────

    /// Deploy and initialize the fee-distributor wired to the given registry address.
    fn setup_distributor<'a>(
        env: &'a Env,
        registry_address: &Address,
    ) -> (Address, FeeDistributorContractClient<'a>) {
        let contract_id = env.register(FeeDistributorContract, ());
        let client = FeeDistributorContractClient::new(env, &contract_id);

        let admin = Address::generate(env);
        client.initialize(
            &admin,
            &500_u32,
            &1000_u32,
            registry_address,
        );

        (contract_id, client)
    }

    /// An Active relay node should receive fees without error.
    #[test]
    fn distribute_succeeds_for_active_relay_node() {
        let env = Env::default();
        env.mock_all_auths();

        let registry_id = env.register(MockActiveRegistry, ());
        let (_id, client) = setup_distributor(&env, &registry_id);

        let relay = Address::generate(&env);
        let result = client.try_distribute(&relay, &1_u64, &100_u32);

        assert!(result.is_ok(), "distribute must succeed for an Active relay node");

        let earnings = client.get_earnings(&relay);
        assert!(earnings.total_earned > 0, "relay node must have earned fees");
        assert!(earnings.unclaimed > 0, "relay node must have unclaimed fees");
    }

    /// An Inactive relay node must be rejected with RelayNodeInactive.
    #[test]
    fn distribute_fails_for_inactive_relay_node() {
        let env = Env::default();
        env.mock_all_auths();

        let registry_id = env.register(MockInactiveRegistry, ());
        let (_id, client) = setup_distributor(&env, &registry_id);

        let relay = Address::generate(&env);
        let result = client.try_distribute(&relay, &1_u64, &100_u32);

        assert_eq!(
            result,
            Err(Ok(crate::errors::ContractError::RelayNodeInactive)),
            "distribute must return RelayNodeInactive for an Inactive relay node"
        );
    }

    /// A Slashed relay node must also be rejected with RelayNodeInactive.
    #[test]
    fn distribute_fails_for_slashed_relay_node() {
        let env = Env::default();
        env.mock_all_auths();

        let registry_id = env.register(MockInactiveRegistry, ());
        let (_id, client) = setup_distributor(&env, &registry_id);

        let relay = Address::generate(&env);
        let result = client.try_distribute(&relay, &2_u64, &100_u32);

        assert_eq!(
            result,
            Err(Ok(crate::errors::ContractError::RelayNodeInactive)),
            "distribute must return RelayNodeInactive for a Slashed relay node"
        );
    }

    /// Rejected nodes must have zero earnings — no partial state written.
    #[test]
    fn rejected_node_has_zero_earnings() {
        let env = Env::default();
        env.mock_all_auths();

        let registry_id = env.register(MockInactiveRegistry, ());
        let (_id, client) = setup_distributor(&env, &registry_id);

        let relay = Address::generate(&env);
        let _ = client.try_distribute(&relay, &3_u64, &100_u32);

        let earnings = client.get_earnings(&relay);
        assert_eq!(earnings.total_earned, 0, "rejected node must have zero total_earned");
        assert_eq!(earnings.unclaimed, 0, "rejected node must have zero unclaimed");
    }

    /// BatchAlreadyDistributed must fire before the registry check on replay.
    #[test]
    fn duplicate_batch_rejected_before_registry_check() {
        let env = Env::default();
        env.mock_all_auths();

        let registry_id = env.register(MockActiveRegistry, ());
        let (_id, client) = setup_distributor(&env, &registry_id);

        let relay = Address::generate(&env);
        client.distribute(&relay, &10_u64, &100_u32);

        let result = client.try_distribute(&relay, &10_u64, &100_u32);
        assert_eq!(
            result,
            Err(Ok(crate::errors::ContractError::BatchAlreadyDistributed)),
            "duplicate batch_id must return BatchAlreadyDistributed"
        );
    }
}
