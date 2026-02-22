# Relay Registry Contract

> Relay node registration and staking for the StellarConduit protocol.

## Overview

The Relay Registry is the foundation of StellarConduit's trust model. Any node that wants to participate as a relay — bridging the offline mesh network to the Stellar blockchain — must first register on-chain and lock a minimum stake of protocol tokens. This stake acts as a cryptographic security deposit: if a relay node misbehaves (by submitting tampered transactions, front-running, or going persistently offline), its stake is slashed by the protocol.

## Contract Functions

| Function | Description |
|---|---|
| `register(node_address, metadata)` | Register a new relay node with mandatory metadata |
| `stake(amount)` | Deposit stake tokens to reach or maintain the minimum threshold |
| `unstake(amount)` | Initiate stake withdrawal, subject to a lock period |
| `slash(node_address, reason)` | Slash a misbehaving node's stake (authorized callers only) |
| `get_node(address)` | Fetch relay node details including status and metadata |
| `is_active(address)` | Check whether a relay node is currently in active status |

## Storage Layout

- `RelayNode` structs are stored keyed by node `Address`
- A global `NodeCount` tracks total registered nodes
- `MinStake` and `StakeLockPeriod` are governance-configurable parameters

## Error Codes

See [`src/errors.rs`](src/errors.rs) for the full list of error codes.

## Status

🚧 **Skeleton / Placeholder** — implementation tracked in GitHub issues.

## Related Contracts

- **Fee Distributor** — reads relay node status to gate fee distribution
- **Dispute Resolver** — references relay node registration for arbitration authority
