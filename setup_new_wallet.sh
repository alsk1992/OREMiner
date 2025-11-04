#!/bin/bash

echo "════════════════════════════════════════════════════════════════"
echo "  SETTING UP NEW MINING WALLET FOR CLEAN TRACKING"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Generate new keypair
echo "Generating new keypair..."
KEYPAIR_PATH="${KEYPAIR:-./keypair_test.json}"
solana-keygen new --outfile $KEYPAIR_PATH --no-bip39-passphrase --force

echo ""
echo "New wallet created!"
echo ""

# Get the pubkey
NEW_WALLET=$(solana-keygen pubkey $KEYPAIR_PATH)

echo "New wallet address: $NEW_WALLET"
echo ""
echo "════════════════════════════════════════════════════════════════"
echo "  NEXT STEPS:"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "1. Send exactly 0.5 SOL to this wallet:"
echo "   $NEW_WALLET"
echo ""
echo "2. We'll track:"
echo "   Starting balance: 0.5 SOL"
echo "   Strategy: 10 squares, 0.003 SOL per round"
echo "   Target: 100 rounds"
echo ""
echo "3. After 100 rounds we'll check final balance:"
echo "   If > 0.5 SOL = PROFITABLE ✅"
echo "   If < 0.5 SOL = LOSING ❌"
echo ""
echo "Clean data, no confusion!"
echo "════════════════════════════════════════════════════════════════"

