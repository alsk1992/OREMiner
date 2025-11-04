#!/bin/bash
# Setup script for ORE mining - No Python dependencies

echo "Creating keypair.json from your private key..."

# Your private key (base58) - Replace with your actual key
PRIVATE_KEY="YOUR_BASE58_PRIVATE_KEY_HERE"

# Method 1: Try using solana-keygen if available
if command -v solana-keygen &> /dev/null; then
    echo "Using solana-keygen to recover keypair..."
    echo "$PRIVATE_KEY" | solana-keygen recover -o keypair.json prompt://
    if [ $? -eq 0 ]; then
        echo "✅ keypair.json created with solana-keygen!"
        export KEYPAIR="$PWD/keypair.json"
        export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
        echo ""
        echo "Environment configured:"
        echo "  KEYPAIR=$KEYPAIR"
        echo "  RPC=$RPC"
        echo ""
        echo "Add to your shell config:"
        echo "  echo 'export KEYPAIR=\"$KEYPAIR\"' >> ~/.bashrc"
        echo "  echo 'export RPC=\"$RPC\"' >> ~/.bashrc"
        exit 0
    fi
fi

# Method 2: Manual creation instructions
echo ""
echo "⚠️  solana-keygen not found. Please create keypair manually:"
echo ""
echo "Option 1: Install Solana CLI"
echo "  sh -c \"\$(curl -sSfL https://release.solana.com/stable/install)\""
echo "  Then run this script again"
echo ""
echo "Option 2: Use Solana Playground"
echo "  1. Go to https://beta.solpg.io/"
echo "  2. Import your private key"
echo "  3. Export as keypair.json"
echo "  4. Copy to $PWD/keypair.json"
echo ""
echo "Option 3: Create from another wallet"
echo "  If you have Phantom/Solflare:"
echo "  1. Export private key"
echo "  2. Use: solana-keygen recover -o keypair.json"
echo ""

# Set environment variables anyway
export KEYPAIR="$PWD/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"

echo "Environment pre-configured:"
echo "  KEYPAIR=$KEYPAIR"
echo "  RPC=$RPC"
echo ""
echo "Once you have keypair.json, run:"
echo "  COMMAND=board cargo run --package ore-cli"
