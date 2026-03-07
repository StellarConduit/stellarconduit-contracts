## feat: Implement SAC token transfers across relay-registry, fee-distributor, and treasury

Closes #36

### Summary

Replaces all `// TODO: SAC transfer` comments with live `token::Client` calls using the Stellar Asset Contract (SAC) interface, wiring real on-chain token movements into the three affected contracts.

### Changes

#### `contracts/relay-registry`
- `storage.rs`: Added `DataKey::TokenAddress` variant and `get_token_address` / `set_token_address` helpers.
- `lib.rs`: Added `use soroban_sdk::token;`. Implemented token transfers in:
  - `stake()` — pulls `amount` tokens from `node_address` → contract
  - `unstake()` — pushes `amount` tokens from contract → `node_address`
  - The slashing-to-treasury transfer is left as a `TODO` (requires a separate treasury address mapping not in scope for this issue).

#### `contracts/fee-distributor`
- `types.rs`: Implemented `FeeEntry`, `EarningsRecord`, and `FeeConfig` structs (prerequisite stubs filled).
- `errors.rs`: Implemented `ContractError` enum with all documented error codes.
- `storage.rs`: Implemented all storage helpers: `get/set_fee_config`, `get/set_fee_entry`, `get/set_earnings`, `get/set_token_address`, `get/set_treasury_address`.
- `lib.rs`: Added `use soroban_sdk::token;`. Implemented token transfers in:
  - `distribute()` — pushes `treasury_share` from contract → treasury address
  - `claim()` — pushes `payout` from contract → relay address

#### `contracts/treasury`
- `lib.rs`: Added `use soroban_sdk::token;`. Implemented token transfers in:
  - `deposit()` — pulls `amount` from `from` → contract
  - `withdraw()` — pushes `amount` from contract → `to`
  - `allocate()` — token transfer commented out with `TODO`; `SpendingProgram` has no `recipient_address` field, so the destination is unknown at call time. Requires a follow-up to add recipient mapping.
- `test.rs`: Updated all tests to register a `StellarAssetClient` token contract, mint balances, and set the token address in storage before exercising deposit/withdraw/allocate paths.

### Testing

```
cargo test --workspace   # 11 passed; 0 failed
cargo fmt --all          # no changes
cargo clippy --all-targets --all-features -- -D warnings   # clean
stellar contract build   # all 4 contracts compile to WASM
```

### Notes

- The `fee-distributor` supporting file implementations (`types.rs`, `errors.rs`, `storage.rs`) were included here because those prerequisite stubs had not yet been merged. They follow the spec documented in each file's module doc comment.
- The `relay-registry` and `fee-distributor` token / treasury address storage helpers will be superseded once the official prerequisite PRs land; the only true new surface area is the `token::Client::transfer()` calls in each contract function.
