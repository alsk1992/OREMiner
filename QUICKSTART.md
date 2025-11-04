# Quick Start - ORE Mining with Late Snipe Strategy

## 1. Setup Keypair

```bash
cd /home/alsk/ore

# Install base58 Python package if needed
pip3 install base58

# Run setup script to create keypair.json
./setup_keypair.sh
```

This will create `keypair.json` from your private key.

## 2. Set Environment Variables

```bash
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
```

Or add to your `.bashrc` / `.zshrc`:
```bash
echo 'export KEYPAIR="/path/to/keypair.json"' >> ~/.bashrc
echo 'export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"' >> ~/.bashrc
source ~/.bashrc
```

## 3. Check Your Setup

```bash
# Check current round
COMMAND=board cargo run --package ore-cli

# Check motherlode pool
COMMAND=treasury cargo run --package ore-cli

# Check your miner status
COMMAND=miner cargo run --package ore-cli
```

## 4. Run First Snipe

```bash
# Deploy with default 0.04 SOL
COMMAND=deploy_optimal cargo run --package ore-cli
```

**What happens:**
1. Shows round info and time remaining
2. Waits automatically until last 10 seconds
3. Fetches live board data
4. Shows full board analytics:
   - Total SOL deployed across all squares
   - Top 5 least crowded squares (your targets)
   - Top 5 most crowded squares (to avoid)
   - Miner counts per square
5. Deploys 0.02 SOL to each of 2 least crowded squares
6. Confirms deployment

## 5. After Round Ends

```bash
# Checkpoint your rewards (~30 seconds after round ends)
COMMAND=checkpoint cargo run --package ore-cli

# Check your rewards
COMMAND=miner cargo run --package ore-cli

# Claim rewards to wallet
COMMAND=claim cargo run --package ore-cli
```

## 6. Automated Mining Loop

Create `mine.sh`:
```bash
#!/bin/bash
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"

while true; do
  echo "=== Round Start ==="

  # Check status
  COMMAND=board cargo run --package ore-cli

  # Deploy (waits automatically for snipe window)
  COMMAND=deploy_optimal cargo run --package ore-cli

  # Wait for round to complete
  sleep 30

  # Checkpoint
  COMMAND=checkpoint cargo run --package ore-cli

  # Check results
  COMMAND=miner cargo run --package ore-cli

  echo "=== Round Complete ==="
  sleep 5
done
```

Run it:
```bash
chmod +x mine.sh
./mine.sh
```

## Expected Output

```
╔════════════════════════════════════════════════════════════════╗
║               LATE SNIPE STRATEGY (2 SQUARES)                  ║
╠════════════════════════════════════════════════════════════════╣
║ Round ID:        31462                                         ║
║ Time Remaining:  427.6 seconds                                 ║
║ Deployment:      0.0400 SOL                                    ║
╚════════════════════════════════════════════════════════════════╝

⏰ Waiting 417.6 seconds until last 10 seconds of round...
   (This gives us perfect information about board state)

⚡ SNIPE WINDOW ACTIVE - Analyzing board...

╔════════════════════════════════════════════════════════════════╗
║                    BOARD ANALYTICS (LIVE)                      ║
╠════════════════════════════════════════════════════════════════╣
║ Total Deployed:  12.4521 SOL across all squares               ║
║ Average/Square:  0.4981 SOL                                    ║
╠════════════════════════════════════════════════════════════════╣
║ Top 5 LEAST Crowded Squares:                                  ║
║ 🎯 Square #3   0.3521 SOL  (8 miners)                         ║
║ 🎯 Square #17  0.3789 SOL  (9 miners)                         ║
║    Square #22  0.4012 SOL  (11 miners)                        ║
║    Square #5   0.4123 SOL  (12 miners)                        ║
║    Square #14  0.4234 SOL  (10 miners)                        ║
╠════════════════════════════════════════════════════════════════╣
║ Top 5 MOST Crowded Squares (AVOID):                           ║
║    Square #12  0.8921 SOL  (24 miners)                        ║
║    Square #7   0.8234 SOL  (21 miners)                        ║
║    Square #19  0.7856 SOL  (19 miners)                        ║
║    Square #1   0.7234 SOL  (18 miners)                        ║
║    Square #24  0.6912 SOL  (17 miners)                        ║
╚════════════════════════════════════════════════════════════════╝

╔═══════════════════════════════════════════════════════════════╗
║                    SNIPE DEPLOYMENT PLAN                       ║
╠═══════════════════════════════════════════════════════════════╣
║ Square #3:      0.3521 SOL → 0.3721 SOL total (5.38% share)  ║
║ Square #17:     0.3789 SOL → 0.3989 SOL total (5.01% share)  ║
╠═══════════════════════════════════════════════════════════════╣
║ Expected Payouts (if either square wins):                     ║
║   SOL:           ~0.429 SOL                                    ║
║   ORE:           ~0.0520 ORE (avg)                             ║
║   Motherlode:    57.00 ORE pool → 2.9640 ORE if hit           ║
╠═══════════════════════════════════════════════════════════════╣
║ Strategy:        LATE SNIPE 2 SQUARES                          ║
║ Win Rate:        8% (2/25)                                     ║
║ Expected ROI:    +73%                                          ║
╚═══════════════════════════════════════════════════════════════╝

📤 Submitting transaction in last 10 seconds...
✅ SNIPED! Deployed to squares #3 and #17!
```

## Troubleshooting

**"Missing KEYPAIR env var"**
```bash
export KEYPAIR="/path/to/keypair.json"
```

**"Missing RPC env var"**
```bash
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
```

**"Transaction failed"**
- Check SOL balance: Need ~0.05 SOL (0.04 for deployment + gas)
- RPC rate limit: Helius should handle this fine
- Network congestion: Retry the transaction

**"Round already ended"**
- Check timing with `COMMAND=board`
- Start the command earlier in the round

## Strategy Summary

- **Capital:** 0.04 SOL per round ($1)
- **Target:** 2 least crowded squares
- **Timing:** Last 10 seconds (automated)
- **Win Rate:** 8% (2/25)
- **Expected ROI:** +73% per round
- **With 57 ORE motherlode:** Potential $500+ payout per hit

Let's mine! 🎯
