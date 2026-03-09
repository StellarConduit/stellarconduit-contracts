feat(relay-registry): add reinstate_node() to allow slashed nodes to recover

## What this PR does

- Adds a `reinstate_node(env, node_address)` entrypoint to the `relay-registry` contract that allows the admin council to reinstate a previously slashed relay node after a successful governance appeal.
- Introduces a new `ContractError::NodeNotSlashed` variant to prevent reinstatement of nodes that are not currently in the `Slashed` state.
- Emits a `reinstate_node` event whenever a node is successfully reinstated, enabling on-chain observability of governance-driven reactivations.
- Documents the `reinstate_node` workflow in `contract-specs.md` under the Relay Registry function specs.

## Why

Previously, once a relay node was slashed its status was permanently set to `Slashed`, with no way to reinstate honest-but-unlucky operators (e.g., hardware failure, temporary network issues) even after a successful off-chain appeals process through the admin council.  
This PR adds a narrowly scoped, council-gated reactivation path so that slashed nodes can be moved back to `Inactive` status without restoring forfeited stake, allowing them to rejoin the network by depositing fresh stake via `stake()`.

## How to test

From the repository root:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test -p relay-registry
# Optional (requires Stellar CLI): stellar contract build
```

Key tests added in `contracts/relay-registry/tests/relay-registry-test.rs`:

- `test_reinstate_node_from_slashed_to_inactive_and_restake`  
  - Verifies that a node can be registered, staked to `Active`, slashed to `Slashed`, reinstated to `Inactive`, and then restaked back to `Active` with new stake.
- `test_reinstate_node_when_inactive_fails`  
  - Ensures calling `reinstate_node` on an `Inactive` node fails with `ContractError::NodeNotSlashed` (`#16`).
- `test_reinstate_node_when_active_fails`  
  - Ensures calling `reinstate_node` on an `Active` node fails with `ContractError::NodeNotSlashed` (`#16`).
- `test_reinstate_node_not_registered_fails`  
  - Ensures calling `reinstate_node` on an unregistered address fails with `ContractError::NotRegistered` (`#2`).

## Checklist

- [x] My branch is based on the latest `upstream/main`
- [x] My commit messages follow the Conventional Commits format
- [x] I have run `cargo fmt --all`
- [x] I have run `cargo clippy --all-targets --all-features -- -D warnings`
- [x] I have run `cargo test -p relay-registry`
- [x] I have added tests covering success and error cases for `reinstate_node`
- [x] I have updated `docs/contract-specs.md` to document the new entrypoint and workflow

