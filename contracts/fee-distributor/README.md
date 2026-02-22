# Fee Distributor Contract

> Protocol fee calculation and distribution to relay nodes upon successful transaction settlement.

## Overview

When a relay node successfully submits a batch of mesh transactions to Stellar and settlement is confirmed on-chain, the Fee Distributor contract automatically calculates and distributes the relay fee to that node. Fees originate from a small protocol fee added to each transaction at the point of offline signing within the mesh. The Fee Distributor also allocates a share of collected fees to the Protocol Treasury.

## Contract Functions

| Function | Description |
|---|---|
| `distribute(relay_address, batch_id)` | Distribute fee for a confirmed settled batch |
| `calculate_fee(batch_size)` | Calculate the fee amount for a given transaction batch |
| `claim(relay_address)` | Claim all accumulated, unclaimed fees for a relay node |
| `get_earnings(relay_address)` | View lifetime and unclaimed earnings for a relay node |
| `set_fee_rate(rate)` | Update the protocol fee rate in basis points (governance only) |

## Storage Layout

- `EarningsRecord` per relay address tracks total earned and unclaimed balances
- `FeeEntry` per batch ID records individual distribution events
- `FeeConfig` stores the global fee rate and treasury share percentage

## Fee Formula

```
relay_fee = batch_size * fee_rate_bps / 10000
treasury_share = relay_fee * treasury_share_bps / 10000
relay_payout = relay_fee - treasury_share
```

## Error Codes

See [`src/errors.rs`](src/errors.rs) for the full list of error codes.

## Status

🚧 **Skeleton / Placeholder** — implementation tracked in GitHub issues.

## Related Contracts

- **Relay Registry** — queried to verify node is active before distributing fees
- **Treasury** — receives its share of each distribution
