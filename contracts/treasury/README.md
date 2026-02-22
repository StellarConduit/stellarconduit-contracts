# Treasury Contract

> Protocol treasury for relay node incentive programs and operator grants.

## Overview

The Protocol Treasury holds funds that support the health and growth of the StellarConduit network. It receives a share of every relay fee collected by the Fee Distributor, and disburses funds for relay node incentive programs, grants for operators running nodes in underserved and remote regions, and protocol development. In a future version, treasury governance will be handed over to a DAO of protocol stakeholders.

## Contract Functions

| Function | Description |
|---|---|
| `deposit(amount)` | Deposit funds into the protocol treasury |
| `withdraw(amount, recipient, reason)` | Withdraw funds (authorized callers only) |
| `allocate(program, amount)` | Allocate a budget to a named spending program |
| `get_balance()` | Fetch the current treasury token balance |
| `get_history()` | Fetch the full on-chain transaction history |

## Storage Layout

- `Balance` stores the current token balance (i128)
- `Entry(id)` stores individual `TreasuryEntry` records for every deposit and withdrawal
- `Allocation(program)` stores named `AllocationRecord` structs for spending programs
- `Admin` stores the address authorized to perform withdrawals

## Spending Programs

| Program | Description |
|---|---|
| `RelayIncentives` | Performance rewards for high-uptime relay nodes |
| `UnderservedGrants` | Grants for nodes in underserved or remote regions |
| `ProtocolDevelopment` | Core development and infrastructure costs |
| `Custom(name)` | Governance-defined programs |

## Error Codes

See [`src/errors.rs`](src/errors.rs) for the full list of error codes.

## Status

🚧 **Skeleton / Placeholder** — implementation tracked in GitHub issues.

## Related Contracts

- **Fee Distributor** — sends treasury allocations on every fee distribution event
