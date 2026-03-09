## feat(storage): implement TTL extension across all contracts

### Summary
- **Introduce shared TTL constants and helpers** in each contract's `storage.rs` to manage Soroban storage expiration (`LEDGER_BUMP_THRESHOLD` and `LEDGER_BUMP_AMOUNT` plus `extend_instance_ttl(&Env)`).
- **Bump per-key persistent storage TTLs on access** by updating all `persistent()` read/write helpers to call `extend_ttl` whenever active data (e.g., relay registrations, disputes, earnings, treasury entries/programs) is loaded or modified.
- **Extend instance storage TTL on every contract call** by invoking `storage::extend_instance_ttl(&env)` at the start of each public entrypoint, keeping global config/admin state alive as long as the contracts are used.

### Details
- **Relay Registry**
  - Added TTL constants and `extend_instance_ttl` to `storage.rs`.
  - Updated relay node and stake lock entry helpers to bump persistent TTL on both reads and writes.
  - Call `storage::extend_instance_ttl(&env)` at the start of all public functions (`initialize`, `register`, `update_metadata`, `stake`, `unstake`, `finalize_unstake`, `slash`, `reinstate_node`, `get_node`, `is_active`).

- **Fee Distributor**
  - Added TTL constants and `extend_instance_ttl` to `storage.rs`.
  - Updated `get_earnings`/`set_earnings` and `get_fee_entry`/`set_fee_entry` to extend per-key TTL whenever an `EarningsRecord` or `FeeEntry` is read or written.
  - Call `storage::extend_instance_ttl(&env)` at the start of all public functions (`initialize`, `calculate_fee`, `distribute`, `claim`, `get_earnings`, `set_fee_rate`), ensuring the fee config and admin council instance state never expires.

- **Dispute Resolver**
  - Added TTL constants and `extend_instance_ttl` to `storage.rs`.
  - Updated dispute, ruling, tx→dispute mapping, and public key helpers to bump TTL on every persistent read/write:
    - `get_dispute`/`set_dispute`
    - `get_ruling`/`set_ruling`
    - `get_dispute_by_tx`/`set_dispute_by_tx`
    - `get_public_key`/`set_public_key`
  - Call `storage::extend_instance_ttl(&env)` at the start of all public functions (`raise_dispute`, `respond`, `resolve`, `get_dispute`, `get_ruling`, `initialize`).

- **Treasury**
  - Added TTL constants and `extend_instance_ttl` to `storage.rs`.
  - Updated all persistent history/allocation/program helpers to extend TTL on access:
    - Entries: `get_entry`, `set_entry`, `append_entry`
    - Allocations: `get_allocation`/`set_allocation`
    - Programs: `get_spending_program`/`set_spending_program`
  - Call `storage::extend_instance_ttl(&env)` at the start of all public functions (`get_balance`, `get_history`, `initialize`, `deposit`, `withdraw`, `create_program`, `update_program_budget`, `deactivate_program`, `get_program`, `allocate`, `get_treasury_stats`) so instance fields like admin council, token address, counters, and stats stay alive.

### Rationale
- Soroban does not keep storage forever; without explicit TTL bumps, active protocol data (relay registrations, disputes, earnings, treasury records) can expire and be garbage-collected.
- This change centralizes TTL handling in the storage modules and ensures:
  - **Per-key persistent entries** are kept alive as long as they are actively read or written.
  - **Instance/global state** is kept alive as long as contracts continue to be invoked.

### Testing / Verification
- **Formatting & linting**
  - `cargo fmt --all`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- **Build (recommended to run locally)**
  - `stellar contract build`

