# Dispute Resolver Contract

> Final on-chain arbitration for double-spend conflicts in the StellarConduit mesh.

## Overview

In rare cases where a double-spend conflict cannot be resolved deterministically by the off-chain sync engine — for example, when two relay nodes submit conflicting transactions originating from a partitioned mesh cluster simultaneously — the Dispute Resolver provides the final on-chain arbitration layer. Competing parties submit cryptographic relay chain proofs, and the contract evaluates them deterministically to issue a final, immutable ruling.

## Contract Functions

| Function | Description |
|---|---|
| `raise_dispute(tx_id, proof)` | Submit a new dispute with a relay chain proof |
| `respond(dispute_id, proof)` | Submit a counter-proof to an existing open dispute |
| `resolve(dispute_id)` | Trigger resolution after the evaluation window has passed |
| `get_dispute(dispute_id)` | Fetch dispute details and current status |
| `get_ruling(dispute_id)` | Fetch the final ruling for a resolved dispute |

## Storage Layout

- `Dispute` structs are stored keyed by `dispute_id`
- `Ruling` records are stored separately, written only upon resolution
- `DisputeCount` tracks the next available dispute ID
- `ResolutionWindow` is a governance-configurable parameter in ledgers

## Proof Evaluation

The contract evaluates competing `RelayChainProof` values by:
1. Verifying the Ed25519 signature of each relay chain hash
2. Comparing sequence numbers in the relay chain
3. The proof with the earlier, valid sequence number wins

## Error Codes

See [`src/errors.rs`](src/errors.rs) for the full list of error codes.

## Status

🚧 **Skeleton / Placeholder** — implementation tracked in GitHub issues.

## Related Contracts

- **Relay Registry** — losing party may have their stake slashed post-ruling
