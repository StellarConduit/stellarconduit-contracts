#!/usr/bin/env bash
# =============================================================================
# initialize-all.sh
# StellarConduit Contracts — Automated Initialization Script
# =============================================================================
set -euo pipefail

# Load config
SCRIPT_DIR=$(dirname "$0")
ENV_FILE="$SCRIPT_DIR/.env"
DEPLOYED_FILE="$SCRIPT_DIR/.deployed-addresses.env"

if [ ! -f "$ENV_FILE" ]; then
  echo "Error: $ENV_FILE not found." >&2
  exit 1
fi
if [ ! -f "$DEPLOYED_FILE" ]; then
  echo "Error: $DEPLOYED_FILE not found. Run scripts/deploy-all.sh first." >&2
  exit 1
fi

source "$ENV_FILE"
source "$DEPLOYED_FILE"

# Validate required variables
if [ -z "${NETWORK:-}" ]; then
  echo "Error: NETWORK must be set in .env" >&2
  exit 1
fi
if [ -z "${ADMIN_SECRET_KEY:-}" ]; then
  echo "Error: ADMIN_SECRET_KEY must be set in .env" >&2
  exit 1
fi
if [ -z "${ADMIN_ADDRESS:-}" ]; then
  echo "Error: ADMIN_ADDRESS must be set in .env" >&2
  exit 1
fi

TOKEN_ADDRESS="${SAC_TOKEN_ADDRESS:-}"
if [ -z "$TOKEN_ADDRESS" ]; then
  echo "Error: SAC_TOKEN_ADDRESS is not set." >&2
  exit 1
fi

# A helper function to invoke initialization command and check if already initialized
invoke_and_initialize() {
  local contract_name="$1"
  local contract_id="$2"
  local err_code="$3"
  shift 3

  echo "==> Initializing $contract_name..."
  set +e
  local output
  output=$(stellar contract invoke \
      --id "$contract_id" \
      --source "$ADMIN_SECRET_KEY" \
      --network "$NETWORK" \
      -- "$@" 2>&1)
  local status=$?
  set -e

  if [ $status -ne 0 ]; then
    # Check if the output contains the specific contract error for AlreadyInitialized
    if echo "$output" | grep -q -E "Error\(Contract, #$err_code\)|Error\(Contract, $err_code\)"; then
      echo "    $contract_name is already initialized. Skipping."
    else
      echo "Error: Failed to initialize $contract_name" >&2
      echo "$output" >&2
      exit 1
    fi
  else
    echo "    $contract_name initialized successfully."
  fi
}

# 1. Initialize Treasury
# Parameters: council: AdminCouncil, token_address: Address
# AlreadyInitialized error code: 10
invoke_and_initialize "treasury" "$TREASURY_ID" 10 \
    initialize \
    --council "{\"members\":[\"$ADMIN_ADDRESS\"],\"threshold\":1}" \
    --token_address "$TOKEN_ADDRESS"

# 2. Initialize Relay Registry
# Parameters: council: AdminCouncil, token_address: Address, treasury_address: Address, min_stake: i128, stake_lock_period: u32
# AlreadyInitialized error code: 10
invoke_and_initialize "relay-registry" "$REGISTRY_ID" 10 \
    initialize \
    --council "{\"members\":[\"$ADMIN_ADDRESS\"],\"threshold\":1}" \
    --token_address "$TOKEN_ADDRESS" \
    --treasury_address "$TREASURY_ID" \
    --min_stake 1000 \
    --stake_lock_period 1000

# 3. Initialize Fee Distributor
# Parameters: council: AdminCouncil, fee_rate_bps: u32, treasury_share_bps: u32, treasury: Address, token: Address
# AlreadyInitialized error code: 9
invoke_and_initialize "fee-distributor" "$FEE_ID" 9 \
    initialize \
    --council "{\"members\":[\"$ADMIN_ADDRESS\"],\"threshold\":1}" \
    --fee_rate_bps 50 \
    --treasury_share_bps 2000 \
    --treasury "$TREASURY_ID" \
    --token "$TOKEN_ADDRESS"

# 4. Initialize Dispute Resolver
# Parameters: council: AdminCouncil, resolution_window: u32
# AlreadyInitialized error code: 14
invoke_and_initialize "dispute-resolver" "$DISPUTE_ID" 14 \
    initialize \
    --council "{\"members\":[\"$ADMIN_ADDRESS\"],\"threshold\":1}" \
    --resolution_window 5000

echo "==> All contracts initialized and wired."
