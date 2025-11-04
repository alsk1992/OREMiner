# Fix Keypair Issue

## Problem
The error: `signature error: keypair bytes do not specify same pubkey as derived from their secret key`

This means the keypair.json format is wrong.

## Solution Options

### Option 1: Use Phantom/Solflare Export (EASIEST)
1. Import your private key into Phantom or Solflare wallet
2. Export as JSON keypair file
3. Save as `/path/to/keypair.json`

### Option 2: Use Solana CLI
```bash
# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/stable/install)"

# Add to PATH
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"

# Recover keypair from seed phrase or private key
solana-keygen recover -o keypair.json
# Then paste your private key when prompted
```

### Option 3: Online Tool
1. Go to https://www.solaneyes.com/tools/privatekey-to-keypair
2. Paste your private key: `YOUR_BASE58_PRIVATE_KEY_HERE`
3. Download the keypair.json
4. Save to `/path/to/keypair.json`

### Option 4: Ask for Correct Format
If you already have the keypair in correct format from another tool, just paste it into:
`/path/to/keypair.json`

It should be a JSON array of 64 numbers, like:
```json
[12,34,56,78,...]
```

## Once Fixed

Test it:
```bash
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
COMMAND=board cargo run --package ore-cli --release
```

If it works (shows the board without error), you're ready to mine!

```bash
./mine_continuous.sh
```
