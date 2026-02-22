//! # Dispute Resolver Contract — `types.rs`
//!
//! Defines all data structures used by the Dispute Resolver contract.
//!
//! ## Types to implement
//! - `Dispute` — The primary struct representing a submitted dispute:
//!   - `dispute_id: u64` — Unique dispute identifier
//!   - `tx_id: BytesN<32>` — The on-chain transaction ID under dispute
//!   - `initiator: Address` — Address that raised the dispute
//!   - `respondent: Option<Address>` — Address of the counter-party (if responded)
//!   - `initiator_proof: RelayChainProof` — Cryptographic relay chain proof from initiator
//!   - `respondent_proof: Option<RelayChainProof>` — Counter-proof from respondent
//!   - `status: DisputeStatus` — Open, Responded, Resolved, or Expired
//!   - `raised_at: u64` — Ledger when the dispute was raised
//!   - `resolve_by: u64` — Ledger deadline for resolution
//! - `DisputeStatus` — Enum: `Open`, `Responded`, `Resolved`, `Expired`
//! - `Ruling` — Final arbitration outcome:
//!   - `winner: Address` — Address that won the dispute
//!   - `loser: Address` — Address that lost and will be penalized
//!   - `reason: String` — Brief explanation of the ruling
//!   - `resolved_at: u64` — Ledger when the ruling was issued
//! - `RelayChainProof` — Cryptographic proof struct:
//!   - `signature: BytesN<64>` — Ed25519 signature of the relay chain hash
//!   - `chain_hash: BytesN<32>` — Hash of the relay chain at the point of signing
//!   - `sequence: u64` — Sequence number in the relay chain
//!
//! implementation tracked in GitHub issue

#![allow(unused)]

use soroban_sdk::{contracttype, Address, Bytes, BytesN, String};

// implementation tracked in GitHub issue
