# Contributing to StellarConduit Contracts

> Thank you for taking the time to contribute! Every line of code, documentation improvement, test, and idea brings us closer to a world where payments work everywhere — even where the internet doesn't.

---

## Welcome to StellarConduit

StellarConduit is an open-source, offline-first payment network built on the Stellar blockchain. It enables peer-to-peer financial transactions in environments with no internet connectivity by propagating signed Stellar transactions across a Bluetooth and WiFi-Direct mesh network using a gossip protocol, settling them on Stellar when any node in the mesh reaches connectivity.

This repository — `stellarconduit-contracts` — contains the Soroban smart contracts that power the trustless, on-chain layer of the protocol: relay node registration and staking, fee distribution, dispute resolution, and the protocol treasury.

Contributions to this repository have a direct impact on the financial access of people in areas with limited or no internet connectivity. We deeply value contributions from developers of all backgrounds and experience levels.

---

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you agree to uphold this code. Please report unacceptable behavior to **conduct@stellarconduit.org**.

---

## Ways to Contribute

You don't have to write code to make a meaningful contribution. Here are all the ways you can help:

| Type | Examples |
|---|---|
| **Code** | Implement contract functions, fix bugs, improve performance |
| **Testing** | Write unit and integration tests, improve coverage |
| **Documentation** | Improve inline docs, fill in `contract-specs.md`, write tutorials |
| **Research** | Research cryptographic schemes for relay-chain proofs, analyze fee models |
| **Design** | Design the governance model, the DAO handover process, token economics |
| **Review** | Review open pull requests and leave constructive feedback |
| **Security** | Audit contract logic, report vulnerabilities via responsible disclosure |

---

## Getting Started

### Prerequisites

Make sure you have the following installed before contributing:

**Required:**
- [Rust](https://www.rust-lang.org/tools/install) `>= 1.74.0`

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/stellar-cli) `>= 22.0.0`

  Follow the installation instructions in the [Stellar Developer Docs](https://developers.stellar.org/docs/tools/developer-tools/stellar-cli).

- `wasm32-unknown-unknown` Rust target:

  ```bash
  rustup target add wasm32-unknown-unknown
  ```

**Recommended:**
- `rustfmt` and `clippy` (included with Rust toolchain):

  ```bash
  rustup component add rustfmt clippy
  ```

- A funded Stellar testnet account:

  ```bash
  stellar keys generate --global testnet-deployer --network testnet
  stellar keys fund testnet-deployer --network testnet
  ```

### Verify Your Setup

```bash
rustc --version       # Should be >= 1.74.0
stellar --version     # Should be >= 22.0.0
cargo clippy --version
```

### Clone and Build

```bash
git clone https://github.com/StellarConduit/stellarconduit-contracts.git
cd stellarconduit-contracts
stellar contract build
cargo test
```

---

## Finding Issues to Work On

We use GitHub Issues to track all work. The best places to start:

- **[`good first issue`](https://github.com/StellarConduit/stellarconduit-contracts/issues?q=label%3A%22good+first+issue%22)** — Smaller, well-scoped issues that are great for first-time contributors. Clear acceptance criteria and context are provided.
- **[`help wanted`](https://github.com/StellarConduit/stellarconduit-contracts/issues?q=label%3A%22help+wanted%22)** — Issues where we could use community expertise.
- **[`documentation`](https://github.com/StellarConduit/stellarconduit-contracts/issues?q=label%3Adocumentation)** — Documentation improvements and gaps.
- **[`tests`](https://github.com/StellarConduit/stellarconduit-contracts/issues?q=label%3Atests)** — Test coverage improvements.

Before starting work on an issue, please **comment on it** to let maintainers know you are working on it. This prevents duplicate effort.

---

## Development Workflow

### 1. Fork and Clone

Fork the repository on GitHub, then clone your fork:

```bash
git clone https://github.com/<your-username>/stellarconduit-contracts.git
cd stellarconduit-contracts
git remote add upstream https://github.com/StellarConduit/stellarconduit-contracts.git
```

### 2. Create a Branch

Always create a new branch for your work from the latest `main`:

```bash
git fetch upstream
git checkout -b <branch-name> upstream/main
```

#### Branch Naming Conventions

Use the following naming pattern:

```
<type>/<short-description>
```

Where `<type>` is one of:

| Type | When to use |
|---|---|
| `feat` | A new feature or function implementation |
| `fix` | A bug fix |
| `test` | Adding or improving tests |
| `docs` | Documentation changes only |
| `refactor` | Code changes that neither fix a bug nor add a feature |
| `chore` | Build system, CI, or tooling changes |
| `security` | Security-related changes or patches |

**Examples:**
```
feat/relay-registry-register
fix/fee-distributor-overflow
test/treasury-withdrawal-coverage
docs/contract-specs-relay-registry
```

### 3. Write Your Code

- Follow the [Rust Style Guidelines](#rust-style-guidelines) described below
- Keep each commit focused on a single logical change
- Write or update tests for any code you change
- Update inline documentation (`///` doc comments) for any public functions you add or modify

### 4. Commit Your Changes

We follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification for all commit messages.

#### Commit Message Format

```
<type>(<scope>): <short description>

[optional body]

[optional footer(s)]
```

Where:
- **`type`**: `feat`, `fix`, `test`, `docs`, `refactor`, `chore`, `security`, `perf`
- **`scope`**: The contract or area affected (`relay-registry`, `fee-distributor`, `dispute-resolver`, `treasury`, `workspace`, `scripts`, `docs`, `ci`)
- **`short description`**: Imperative mood, lowercase, no period at the end, max 72 characters

#### Commit Examples

```
feat(relay-registry): implement register() function with metadata validation

fix(fee-distributor): prevent double-distribution for the same batch_id

test(treasury): add withdrawal coverage for InsufficientBalance error case

docs(dispute-resolver): add relay chain proof evaluation algorithm spec

chore(workspace): upgrade soroban-sdk to 22.0.1

security(relay-registry): enforce min stake check before node activation
```

#### Breaking Changes

If your change introduces a breaking change to an existing public contract interface, add a footer:

```
feat(relay-registry): add uptime_commitment field to NodeMetadata

BREAKING CHANGE: NodeMetadata struct now requires uptime_commitment field.
Existing deployments must be re-initialized.
```

### 5. Run Tests Before Submitting

Always make sure all tests pass and the codebase is clean before opening a PR:

```bash
# Format code
cargo fmt --all

# Lint with clippy (zero warnings policy)
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test

# Run tests for a specific contract
cargo test -p relay-registry

# Build WASM (ensures the contract compiles for the target)
stellar contract build
```

### 6. Push and Open a Pull Request

```bash
git push origin <branch-name>
```

Then open a Pull Request on GitHub against the `main` branch of `StellarConduit/stellarconduit-contracts`.

---

## Pull Request Process

### PR Title

Follow the same Conventional Commits format as your commit messages:

```
feat(relay-registry): implement stake() and unstake() functions
```

### PR Description

Your PR description should include:

1. **What this PR does** — A concise summary of the changes
2. **Why** — The motivation and context (link the related issue with `Closes #<issue-number>`)
3. **How to test** — Steps for reviewers to verify the changes
4. **Checklist** (see below)

### PR Checklist

Before marking your PR as ready for review, verify the following:

- [ ] My branch is based on the latest `upstream/main`
- [ ] My commit messages follow the Conventional Commits format
- [ ] I have run `cargo fmt --all` and the code is formatted correctly
- [ ] I have run `cargo clippy --all-targets --all-features -- -D warnings` with no warnings
- [ ] I have run `cargo test` and all tests pass
- [ ] I have run `stellar contract build` and the WASM compiles successfully
- [ ] I have added new tests for the functionality I implemented
- [ ] I have updated inline documentation for public functions I added or changed
- [ ] I have linked the relevant GitHub issue (`Closes #<number>`)
- [ ] I have NOT committed any secret keys, environment files, or `.stellar/identity/` files

### Review Process

- At least **one core maintainer** must approve your PR before it can be merged
- For changes to contract logic, at least **two approvals** are required
- Maintainers may request changes; please address all review comments before re-requesting review
- Once approved, a maintainer will merge your PR using **squash merge** to keep the history clean

---

## Proposing a New Feature or RFC

For significant new features — new contract functions, changes to the protocol design, new contracts — please open an RFC (Request for Comments) before writing code:

1. Open a GitHub Issue with the title format: `[RFC] <Feature Name>`
2. Describe the feature, the problem it solves, and the proposed design
3. Add the `RFC` label to the issue
4. Engage with the community discussion in the issue comments
5. If the RFC is accepted by maintainers, a follow-up implementation issue will be created

We want everyone's ideas to be heard. Don't be discouraged if your RFC goes through many rounds of discussion — that's normal and healthy.

---

## Reporting a Bug

If you find a bug, please open a GitHub Issue using the following template:

```markdown
## Bug Report

**Summary**
A clear and concise description of the bug.

**Contract / Function**
Which contract and function are affected? (e.g., `relay-registry::stake()`)

**Steps to Reproduce**
1. Deploy the contract with...
2. Call function with arguments...
3. Observe...

**Expected Behavior**
What should have happened?

**Actual Behavior**
What actually happened? Include error codes and output.

**Environment**
- Stellar CLI version: `stellar --version`
- Rust version: `rustc --version`
- Operating System:
- Network: testnet / mainnet / local

**Additional Context**
Transaction hashes, contract IDs, relevant log output.
```

> ⚠️ **Security vulnerabilities:** If the bug is a security vulnerability, **do not open a public issue**. Please send a responsible disclosure to **security@stellarconduit.org**. We will respond within 48 hours. See the [Security section of the README](README.md#security) for our full disclosure policy.

---

## Rust Style Guidelines

We follow standard Rust community conventions with a few project-specific rules:

### Formatting

We use `rustfmt` with default settings. Always run before committing:

```bash
cargo fmt --all
```

### Linting

We use `clippy` with zero-warning policy. All clippy warnings must be resolved before merging:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Code Guidelines

- **`#![no_std]`** — All contract `lib.rs` files must be `no_std` (required by Soroban)
- **Error handling** — Use the `ContractError` enum defined in `errors.rs`; avoid `unwrap()` and `expect()` in contract code
- **Documentation** — All public contract functions must have `///` doc comments describing parameters, return values, and error cases
- **Module organization** — Keep to the established pattern: `lib.rs` (interface), `types.rs` (structs/enums), `storage.rs` (storage helpers), `errors.rs` (error codes)
- **Storage keys** — Define all storage keys as a `DataKey` enum in `storage.rs`; never use raw string literals as storage keys
- **Authorization** — Always call `require_auth()` on the appropriate address before performing privileged operations
- **Overflow** — Use checked arithmetic (`checked_add`, `checked_mul`) for all arithmetic on token amounts

### Testing Guidelines

- **Minimum coverage** — Aim for **80% test coverage** across all contracts; CI will report coverage
- **Test names** — Use the pattern `test_<function_name>_<scenario>`, e.g., `test_stake_below_minimum`
- **Test isolation** — Each test function should set up its own `Env` and contract — do not share state between tests
- **Happy path + error path** — Every function should have tests for the success case and every defined error case
- **Use `soroban-sdk` testutils** — Use `testutils` feature for `Address::generate(&env)`, `register_contract()`, etc.

---

## Community and Communication

| Channel | Purpose |
|---|---|
| **GitHub Issues** | Bug reports, feature requests, RFCs |
| **GitHub Discussions** | Questions, ideas, general discussion |
| **Discord** | Real-time community chat (link in repo description) |
| **Twitter / X** | Announcements and project updates |

For questions about contributing, feel free to open a GitHub Discussion or join our Discord. We're a friendly community and happy to help you get started.

---

## Recognition and Credits

Every contributor to StellarConduit is recognized:

- All contributors are listed in the GitHub contributors graph
- Significant contributions are highlighted in release notes
- Contributors who help with security disclosures are credited in our security acknowledgements
- We celebrate our community: if you've made a meaningful contribution and haven't been recognized, let us know!

**Thank you for building the future of offline payments with us.** 🌍

---

*This CONTRIBUTING.md was inspired by best practices from the Stellar Developer Community, the Rust community guide, and the Contributor Covenant.*
