#!/usr/bin/env bash
# =============================================================================
# deploy-all.sh
# StellarConduit Contracts — Automated Deployment Script
# =============================================================================
set -euo pipefail

# Load config
SCRIPT_DIR=$(dirname "$0")
ENV_FILE="$SCRIPT_DIR/.env"

if [ ! -f "$ENV_FILE" ]; then
  echo "Error: $ENV_FILE not found. Please copy .env.example to .env and configure it." >&2
  exit 1
fi

source "$ENV_FILE"

# Validate required variables
if [ -z "${NETWORK:-}" ]; then
  echo "Error: NETWORK must be set in .env" >&2
  exit 1
fi
if [ -z "${ADMIN_SECRET_KEY:-}" ]; then
  echo "Error: ADMIN_SECRET_KEY must be set in .env" >&2
  exit 1
fi

echo "==> Building contracts for wasm32-unknown-unknown..."
cargo build --target wasm32-unknown-unknown --release

# Setup network in CLI if it's local
if [ "$NETWORK" = "local" ]; then
  RPC_URL="${RPC_URL:-http://localhost:8000/soroban/rpc}"
  if ! stellar network list | grep -q "^local$"; then
    echo "==> Configuring local network in Stellar CLI..."
    stellar network add --global local \
      --rpc-url "$RPC_URL" \
      --network-passphrase "Standalone Network ; Submissions Let's Play"
  fi
fi

# Determine SAC Token Address
if [ -n "${SAC_TOKEN_ADDRESS:-}" ]; then
  USED_SAC_TOKEN_ADDRESS="$SAC_TOKEN_ADDRESS"
else
  echo "==> SAC_TOKEN_ADDRESS not set. Auto-deploying/wrapping native XLM token..."
  if [ "$NETWORK" = "local" ]; then
    echo "    Funding admin account via local friendbot..."
    ADMIN_ADDR=$(stellar keys address "$ADMIN_SECRET_KEY")
    curl -sf "http://localhost:8000/friendbot?addr=$ADMIN_ADDR" >/dev/null || true
  fi
  USED_SAC_TOKEN_ADDRESS=$(stellar contract asset deploy \
    --asset native \
    --source "$ADMIN_SECRET_KEY" \
    --network "$NETWORK")
  echo "    Wrapped Native SAC Token Address: $USED_SAC_TOKEN_ADDRESS"
fi

echo "==> Deploying treasury..."
TREASURY_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/treasury.wasm \
    --source "$ADMIN_SECRET_KEY" \
    --network "$NETWORK")
echo "    Treasury ID: $TREASURY_ID"

echo "==> Deploying relay-registry..."
REGISTRY_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/relay_registry.wasm \
    --source "$ADMIN_SECRET_KEY" \
    --network "$NETWORK")
echo "    Relay Registry ID: $REGISTRY_ID"

echo "==> Deploying fee-distributor..."
FEE_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/fee_distributor.wasm \
    --source "$ADMIN_SECRET_KEY" \
    --network "$NETWORK")
echo "    Fee Distributor ID: $FEE_ID"

echo "==> Deploying dispute-resolver..."
DISPUTE_ID=$(stellar contract deploy \
    --wasm target/wasm32-unknown-unknown/release/dispute_resolver.wasm \
    --source "$ADMIN_SECRET_KEY" \
    --network "$NETWORK")
echo "    Dispute Resolver ID: $DISPUTE_ID"

# Save deployed addresses
cat > "$SCRIPT_DIR/.deployed-addresses.env" <<EOF
TREASURY_ID=$TREASURY_ID
REGISTRY_ID=$REGISTRY_ID
FEE_ID=$FEE_ID
DISPUTE_ID=$DISPUTE_ID
SAC_TOKEN_ADDRESS=$USED_SAC_TOKEN_ADDRESS
EOF

echo "==> All contracts deployed successfully."
echo "    Addresses written to $SCRIPT_DIR/.deployed-addresses.env"
