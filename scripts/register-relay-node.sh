#!/usr/bin/env bash
# =============================================================================
# register-relay-node.sh
# StellarConduit Contracts — Register and Stake a Relay Node
# =============================================================================
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "Usage: $0 <NODE_ADDRESS_OR_SECRET> <STAKE_AMOUNT> [REGION]" >&2
  exit 1
fi

NODE_ARG="$1"
STAKE_AMOUNT="$2"
REGION="${3:-XX}"

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

# Determine public key and secret key/signer
NODE_SECRET=""
NODE_PUBLIC=""

if [[ "$NODE_ARG" =~ ^S[A-Z2-7]{55}$ ]]; then
  # It's a secret key
  NODE_SECRET="$NODE_ARG"
  echo "==> Deriving public address from secret key..."
  NODE_PUBLIC=$(stellar keys address "$NODE_SECRET")
else
  # It's a public key (G...)
  NODE_PUBLIC="$NODE_ARG"
  if [ "$NODE_PUBLIC" = "$ADMIN_ADDRESS" ]; then
    NODE_SECRET="$ADMIN_SECRET_KEY"
  elif [ -n "${NODE_SECRET_KEY:-}" ]; then
    NODE_SECRET="$NODE_SECRET_KEY"
  else
    NODE_SECRET="$NODE_PUBLIC" # fall back to passing public key (works if loaded in CLI)
  fi
fi

# Fund the node account if local or testnet to ensure it has native XLM (wrapped SAC token)
echo "==> Checking node account funding state..."
if [ "$NETWORK" = "local" ]; then
  echo "    Funding account via local Friendbot..."
  curl -sf "http://localhost:8000/friendbot?addr=$NODE_PUBLIC" >/dev/null || true
elif [ "$NETWORK" = "testnet" ]; then
  echo "    Funding account via Testnet Friendbot..."
  curl -sf "https://friendbot.stellar.org/?addr=$NODE_PUBLIC" >/dev/null || true
fi

echo "==> Registering node $NODE_PUBLIC in region $REGION..."
set +e
output=$(stellar contract invoke \
  --id "$REGISTRY_ID" \
  --source "$NODE_SECRET" \
  --network "$NETWORK" \
  -- register \
  --node_address "$NODE_PUBLIC" \
  --metadata "{\"region\":\"$REGION\",\"capacity\":1000,\"uptime_commitment\":95}" 2>&1)
status=$?
set -e

if [ $status -ne 0 ]; then
  # AlreadyRegistered error code: 2
  if echo "$output" | grep -q -E "Error\(Contract, #2\)|Error\(Contract, 2\)"; then
    echo "    Node $NODE_PUBLIC is already registered. Skipping registration."
  else
    echo "Error: Failed to register node $NODE_PUBLIC" >&2
    echo "$output" >&2
    exit 1
  fi
else
  echo "    Node $NODE_PUBLIC registered successfully."
fi

echo "==> Staking $STAKE_AMOUNT tokens..."
stellar contract invoke \
  --id "$REGISTRY_ID" \
  --source "$NODE_SECRET" \
  --network "$NETWORK" \
  -- stake \
  --node_address "$NODE_PUBLIC" \
  --amount "$STAKE_AMOUNT"

echo "==> Node $NODE_PUBLIC is now Active with stake $STAKE_AMOUNT."
