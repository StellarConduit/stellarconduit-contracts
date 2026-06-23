#!/usr/bin/env bash
# =============================================================================
# run-integration-tests.sh
# StellarConduit Contracts — E2E Sandbox Test Runner
# =============================================================================
set -euo pipefail

SCRIPT_DIR=$(dirname "$0")

echo "==> Starting Stellar Quickstart (local sandbox)..."
# Pull and run the quickstart container
docker run --rm -d \
    --name stellar-sandbox \
    -p 8000:8000 \
    stellar/quickstart:latest --local --enable-soroban-rpc

# Helper to stop sandbox on exit
cleanup() {
  echo "==> Cleaning up..."
  docker stop stellar-sandbox || true
  rm -f "$SCRIPT_DIR/.env" "$SCRIPT_DIR/.deployed-addresses.env"
}
trap cleanup EXIT

# Wait for RPC to be ready
echo "==> Waiting for local RPC to be ready..."
until curl -sf http://localhost:8000/soroban/rpc >/dev/null; do
  echo "    RPC not ready yet, sleeping 2 seconds..."
  sleep 2
done
echo "==> RPC is ready!"

# Configure network in Stellar CLI
echo "==> Configuring local network in Stellar CLI..."
stellar network add --global local \
  --rpc-url http://localhost:8000/soroban/rpc \
  --network-passphrase "Standalone Network ; Submissions Let's Play"

# Generate and fund local-admin identity
echo "==> Generating and funding local-admin identity..."
stellar keys generate --global local-admin --network local

export NETWORK=local
export ADMIN_SECRET_KEY=local-admin
export ADMIN_ADDRESS=$(stellar keys address local-admin)
export RPC_URL=http://localhost:8000/soroban/rpc
export HORIZON_URL=http://localhost:8000

# Create a temporary scripts/.env for deployment/initialize scripts to source
echo "==> Writing temporary scripts/.env for local deployment..."
cat > "$SCRIPT_DIR/.env" <<EOF
NETWORK=$NETWORK
ADMIN_SECRET_KEY=$ADMIN_SECRET_KEY
ADMIN_ADDRESS=$ADMIN_ADDRESS
SAC_TOKEN_ADDRESS=
RPC_URL=$RPC_URL
HORIZON_URL=$HORIZON_URL
EOF

echo "==> Deploying contracts to local sandbox..."
bash "$SCRIPT_DIR/deploy-all.sh"

echo "==> Initializing contracts..."
bash "$SCRIPT_DIR/initialize-all.sh"

echo "==> Running cargo test..."
cargo test -- --include-ignored

echo "==> Integration tests complete."
