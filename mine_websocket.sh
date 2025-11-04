#!/bin/bash
# WebSocket-Driven Continuous ORE Mining
# Usage: ./mine_websocket.sh [SOL_AMOUNT]
# Example: ./mine_websocket.sh 0.05  (bets 0.05 SOL per round)

# Get bet amount from command line (default 0.02 SOL)
if [ -z "$1" ]; then
    BET_SOL="0.02"
else
    BET_SOL="$1"
fi

# Convert SOL to lamports (multiply by 1 billion)
BET_LAMPORTS=$(echo "$BET_SOL * 1000000000" | bc | cut -d. -f1)

export KEYPAIR="${KEYPAIR:-/path/to/keypair.json}"
export RPC="${RPC:-https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY}"
export COMMAND="deploy_optimal_ev"
export BET_AMOUNT=$BET_LAMPORTS

# Calculate per-square amount (10 squares)
PER_SQUARE=$(echo "scale=6; $BET_SOL / 10" | bc)

# Estimate share percentage (rough: assuming 30 SOL per square average)
SHARE_PCT=$(echo "scale=4; ($PER_SQUARE / 30) * 100" | bc)

# Calculate rounds with current balance
CURRENT_BALANCE=$(solana balance $KEYPAIR --url $RPC 2>/dev/null | awk '{print $1}')
if [ ! -z "$CURRENT_BALANCE" ]; then
    ROUNDS=$(echo "scale=0; $CURRENT_BALANCE / $BET_SOL" | bc)
else
    ROUNDS="unknown"
fi

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║              💎 MOTHERLODE HUNTER MODE 💎                      ║"
echo "╠════════════════════════════════════════════════════════════════╣"
echo "║ Strategy: 10 LEAST CROWDED squares                            ║"
echo "║ Bet Size: $BET_SOL SOL per round ($PER_SQUARE per square)               ║"
echo "║ Est Share: ~$SHARE_PCT% per square                                  ║"
echo "║ Balance: $CURRENT_BALANCE SOL (~$ROUNDS rounds)                           ║"
echo "║                                                                ║"
echo "║ 💰 Motherlode: ~170 ORE (1/625 chance per winning square)     ║"
echo "║ 🎯 Your potential: \$$(echo "scale=0; 170 * 463 * $SHARE_PCT / 100" | bc) if you hit it!                    ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo "🚀 Starting auto-mining with $BET_SOL SOL per round..."
echo "   (Press Ctrl+C to stop)"
echo ""

# Run the optimal EV deployment strategy
cargo run --bin ore-cli --release
