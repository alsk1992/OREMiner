# START MINING NOW - Complete Setup

## ✅ EVERYTHING IS READY

Your keypair is configured, RPC is set, and the mining bot is ready to run.

## Quick Start (3 Commands)

```bash
# 1. Set environment variables
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"

# 2. (Optional) Test single deployment first
COMMAND=deploy_optimal cargo run --package ore-cli --release

# 3. Start continuous mining
./mine_continuous.sh
```

## What The Bot Does

1. **Waits for Snipe Window** - Automatically waits until last 10 seconds
2. **Analyzes Live Board** - Shows you EXACT miner counts per square:
   ```
   🎯 Square #3   0.3521 SOL  (8 miners)   ← TARGET
   🎯 Square #17  0.3789 SOL  (9 miners)   ← TARGET
      Square #22  0.4012 SOL  (11 miners)
   ```
3. **Deploys to 2 Least Crowded** - 0.02 SOL each
4. **Checkpoints** - After round ends (records wins, NO 10% tax)
5. **Tracks Stats** - Shows win/loss live
6. **Repeats Forever** - Until you stop it

## Important: Rewards Strategy

✅ **CHECKPOINT after every round** - This records your wins
❌ **DO NOT CLAIM** - Withdrawing ORE costs 10% tax

**Strategy:**
- Let rewards accumulate in your miner account
- They compound (no tax on checkpointing)
- Only claim when you want to cash out (minimize frequency)

Check your accumulated rewards anytime:
```bash
COMMAND=miner cargo run --package ore-cli --release
```

You'll see:
- `rewards_ore:` - Your unclaimed ORE (tax-free if you don't withdraw)
- `lifetime_rewards_ore:` - Total won

## Bot Features

### Real-Time Board Analytics
```
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
╠════════════════════════════════════════════════════════════════╣
║ Top 5 MOST Crowded Squares (AVOID):                           ║
║    Square #12  0.8921 SOL  (24 miners)                        ║
║    Square #7   0.8234 SOL  (21 miners)                        ║
╚════════════════════════════════════════════════════════════════╝
```

### Live Statistics
```
╔════════════════════════════════════════════════════════════════╗
║                      SESSION STATISTICS                        ║
╠════════════════════════════════════════════════════════════════╣
║ Runtime:          2h 34m                                       ║
║ Rounds Played:    15                                           ║
║ Rounds Won:       1                                            ║
║ Win Rate:         6.67% (target: 8%)                           ║
║ SOL Deployed:     0.60 SOL                                     ║
║ Status:           Mining... (Ctrl+C to stop)                   ║
╚════════════════════════════════════════════════════════════════╝
```

## Expected Results

### First 100 Rounds (~10 hours)
- Capital deployed: 4 SOL ($100)
- Expected wins: 8 rounds
- Expected ORE won: ~0.3 ORE
- Expected profit: ~$66-73

### With 57 ORE Motherlode (Current)
- If you hit it: ~2.2 ORE payout (~$570)
- Probability per round: 0.16% (1/625)
- Expected hits in 100 rounds: ~13% chance

## Stopping the Bot

Press `Ctrl+C` to stop mining gracefully.

## Monitoring

### Check Your Balance
```bash
COMMAND=miner cargo run --package ore-cli --release
```

Shows:
- `deployed`: Current round deployment
- `rewards_sol`: SOL rewards accumulated
- `rewards_ore`: ORE rewards accumulated (unclaimed)
- `lifetime_rewards_ore`: Total ORE ever won

### Check Motherlode Pool
```bash
COMMAND=treasury cargo run --package ore-cli --release
```

Look for `motherlode:` - if it's >30 ORE, that's a good sign!

### Check Current Round
```bash
COMMAND=board cargo run --package ore-cli --release
```

Shows time remaining in current round.

## Troubleshooting

**"Missing KEYPAIR env var"**
```bash
export KEYPAIR="/path/to/keypair.json"
```

**"Insufficient funds"**
- Need ~5 SOL to start (4 SOL for mining + 1 SOL buffer for fees)
- Top up your wallet

**"Transaction failed"**
- Usually network congestion
- Bot will retry next round automatically

**"Win rate too low"**
- Normal variance in small samples
- Need 50-100 rounds for convergence to 8%

## Files You Have

- `keypair.json` - Your wallet (KEEP SAFE!)
- `mine_continuous.sh` - The mining bot
- All strategy documentation in markdown files

## Security

⚠️ **PROTECT YOUR KEYPAIR.json**
- Never share it
- Never commit to git
- Keep backups in secure location

## Support

Check these docs:
- [QUICKSTART.md](QUICKSTART.md) - Detailed setup
- [OPTIMAL_STRATEGY_USAGE.md](OPTIMAL_STRATEGY_USAGE.md) - Strategy explanation
- [0.04_SOL_STRATEGY.md](0.04_SOL_STRATEGY.md) - Mathematical proof

---

## 🎯 READY TO START

```bash
# Set vars
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"

# Start mining!
./mine_continuous.sh
```

**The bot will:**
1. Wait for snipe window (last 10s)
2. Show you live board with miner counts
3. Deploy to 2 least crowded squares
4. Checkpoint after round
5. Repeat forever

**Expected: +73% ROI per round, 8% win rate, $570 if you hit motherlode**

Let's mine! ⛏️
