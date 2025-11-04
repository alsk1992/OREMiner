#!/bin/bash

# ORE Mining TUI Launch Script
# Usage: ./run-tui.sh

# Check if KEYPAIR is set
if [ -z "$KEYPAIR" ]; then
    echo "Error: KEYPAIR environment variable not set"
    echo "Usage: export KEYPAIR=/path/to/keypair.json"
    echo "       export RPC=https://your-rpc-url"
    echo "       ./run-tui.sh"
    exit 1
fi

# Check if RPC is set
if [ -z "$RPC" ]; then
    echo "Error: RPC environment variable not set"
    echo "Usage: export KEYPAIR=/path/to/keypair.json"
    echo "       export RPC=https://your-rpc-url"
    echo "       ./run-tui.sh"
    exit 1
fi

echo "🚀 Launching ORE Mining Terminal..."
echo "📁 Keypair: $KEYPAIR"
echo "🌐 RPC: $RPC"
echo ""
echo "Controls:"
echo "  q     - Quit"
echo "  ←→    - Switch tabs"
echo "  m     - Toggle auto-mine"
echo "  c     - Toggle auto-checkpoint"
echo "  d     - Manual deploy"
echo "  r     - Claim rewards"
echo "  +/-   - Adjust deploy amount"
echo ""

COMMAND=tui cargo run --release