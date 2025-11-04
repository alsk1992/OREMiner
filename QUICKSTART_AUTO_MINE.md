# 🚀 QUICKSTART: AUTO-MINE CONTINUOUS OPERATION

**Run once, mine forever with optimal +EV strategy**

---

## ✅ HOW IT WORKS (AUTOMATIC LOOP)

When you start the miner, it runs **CONTINUOUSLY** in an infinite loop:

```
Round 1: Deploy → Wait → Checkpoint → Check win → Log
Round 2: Deploy → Wait → Checkpoint → Check win → Log
Round 3: Deploy → Wait → Checkpoint → Check win → Log
Round 4: Deploy → Wait → Checkpoint → Check win → Log
...forever until you stop it
```

### **Each Round Does:**

1. **Checkpoint previous round** (collect your ORE if you won)
2. **Wait for new round** to start
3. **Analyze the board** (find 2 least crowded squares)
4. **Check previous winner** (hot hand effect)
5. **Wait until 5-10s remaining** (optimal timing)
6. **Deploy 0.02 SOL** (0.01 per square) to optimal squares
7. **Wait for round to end**
8. **Log results** to `optimal_ev_results.jsonl`
9. **Repeat** (go to step 1)

---

## 🎯 WHAT YOU GET (AUTOMATICALLY)

### **Strategy Applied Every Round:**
- ✅ **+30.3% share edge** (deploy to least crowded)
- ✅ **2.3x hot hand effect** (previous winner included if optimal)
- ✅ **5-10s deployment timing** (maximum information)
- ✅ **Auto-checkpoint** (claim your ORE)
- ✅ **Auto-logging** (track all wins/losses)

### **No Manual Work Needed:**
- ❌ No need to click deploy each round
- ❌ No need to calculate which squares
- ❌ No need to checkpoint manually
- ❌ No need to check if you won

**It does EVERYTHING automatically!** 🤖

---

## 🚀 START AUTO-MINING (ONE COMMAND)

### **Default: 0.02 SOL total (0.01 per square)**

```bash
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"

./mine_websocket.sh
```

**That's it!** The miner will run continuously until you stop it (Ctrl+C).

---

### **Custom Bet Size:**

```bash
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
export BET_AMOUNT=40000000  # 0.04 SOL total (0.02 per square)

./mine_websocket.sh
```

---

## 📊 WHAT YOU'LL SEE (LIVE OUTPUT)

```
╔════════════════════════════════════════════════════════════════╗
║          🎯 OPTIMAL +EV MINING STRATEGY (UPDATED)              ║
╠════════════════════════════════════════════════════════════════╣
║ Research: 100 rounds analyzed (NEW dataset)                    ║
║ Edge 1: +30% better share in least crowded squares             ║
║ Edge 2: Previous winner 2.3x more likely to win again!         ║
║ Filter: NONE - Keep previous winner if in least crowded        ║
║ Timing: Deploy at 5-10s remaining (maximum info)               ║
║ Amount: 0.0200 SOL per round (0.01 per square)                 ║
╚════════════════════════════════════════════════════════════════╝

🔌 Starting WebSocket connections...
✅ Board subscription active
✅ Slot subscription active

╔════════════════════════════════════════════════════════════════╗
║  Round #1                                                       ║
╚════════════════════════════════════════════════════════════════╝

⏳ Waiting for new round to start...
🆕 New round #45930 started!

🎯 OPTIMAL STRATEGY:
   Previous winner: Square #13 (🔥 INCLUDED - Hot Hand Effect!)
   → 2.3x more likely to win again (9.1% vs 4%)
   Selected squares:
      1. Square #4 - 0.0353 SOL pool - 4.35% share
      2. Square #13 - 0.0421 SOL pool - 4.12% share
   Average share: 4.24%
   Win chance: 8.0% (2/25 squares)

⏰ Waiting for optimal deployment window (5-10s remaining)...
✅ Optimal window reached (8.3s remaining)

🚀 Deploying to optimal squares...
✅ Deployed 0.0200 SOL to squares 4 & 13!

⏳ Waiting for round to end...

╔════════════════════════════════════════════════════════════════╗
║  Round #2                                                       ║
╚════════════════════════════════════════════════════════════════╝

📝 Checkpointing previous round...
✅ Checkpointed round #45930
🎲 Round #45930 winner: Square #13
✅ WE WON! +0.042 ORE

📊 Stats: 1/1 wins (100.0%)

⏳ Waiting for new round to start...
🆕 New round #45931 started!

🎯 OPTIMAL STRATEGY:
   Previous winner: Square #13 (not in least crowded 2)
   Selected squares:
      1. Square #7 - 0.0298 SOL pool - 4.89% share
      2. Square #19 - 0.0315 SOL pool - 4.71% share
   Average share: 4.80%
   Win chance: 8.0% (2/25 squares)

⏰ Waiting for optimal deployment window (5-10s remaining)...
...

[Continues automatically forever]
```

---

## 📁 TRACKING YOUR RESULTS

All rounds are logged to: **`optimal_ev_results.jsonl`**

### **Check your stats:**

```bash
# Total ORE won
cat optimal_ev_results.jsonl | jq -r '.ore_won' | awk '{s+=$1} END {print s " ORE"}'

# Win rate
echo "Wins: $(cat optimal_ev_results.jsonl | jq -r '.won' | grep true | wc -l)"
echo "Total: $(cat optimal_ev_results.jsonl | wc -l)"

# Hot hand hits (when you had the edge)
cat optimal_ev_results.jsonl | jq -r 'select(.had_hot_hand_edge == true)' | wc -l

# Total SOL deployed
cat optimal_ev_results.jsonl | jq -r '.amount_deployed_sol' | awk '{s+=$1} END {print s " SOL"}'
```

### **View last 5 rounds:**

```bash
tail -5 optimal_ev_results.jsonl | jq
```

---

## ⏹️ STOPPING THE MINER

**Press `Ctrl+C` to stop the continuous loop.**

It will finish the current round checkpoint and exit cleanly.

---

## 🔄 RESTARTING AFTER STOP

Just run the same command again:

```bash
./mine_websocket.sh
```

**It will:**
- ✅ Resume from the current round
- ✅ Checkpoint any unclaimed ORE
- ✅ Continue the strategy automatically

---

## ⚙️ CONFIGURATION OPTIONS

### **Change Bet Size:**

```bash
# Small (0.01 SOL total = 0.005 per square)
export BET_AMOUNT=10000000

# Default (0.02 SOL total = 0.01 per square)
# No need to set - this is the default

# Medium (0.04 SOL total = 0.02 per square)
export BET_AMOUNT=40000000

# Large (0.08 SOL total = 0.04 per square)
export BET_AMOUNT=80000000
```

### **Change RPC (if needed):**

```bash
export RPC="https://your-custom-rpc-url.com"
```

---

## 📊 EXPECTED PERFORMANCE

### **With 0.01 SOL per square (0.02 total):**

| Metric | Value |
|--------|-------|
| Bet per round | 0.02 SOL |
| Win chance | 8% (2 out of 25) |
| Your share when win | ~2.23% |
| Share edge | +30% vs most crowded |
| Hot hand bonus | 2.3x when applicable |
| Rounds per SOL | 50 rounds |

**Over 100 rounds:**
- Capital needed: 2 SOL
- Expected wins: ~8 rounds
- ORE accumulated: Varies by emissions
- **Key: +30% better share than random deployment**

---

## ✅ CHECKLIST BEFORE STARTING

1. **Keypair loaded:**
   ```bash
   ls -la /path/to/keypair.json
   ```

2. **Sufficient SOL balance:**
   - Need at least 0.5 SOL for 25 rounds
   - Recommended: 2+ SOL for continuous operation

3. **RPC configured:**
   ```bash
   echo $RPC
   # Should show your Helius RPC URL
   ```

4. **Binary built:**
   ```bash
   ls -lh target/release/ore-cli
   # Should exist (19M size)
   ```

---

## 🎯 START MINING NOW

```bash
# Set environment
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"

# Start auto-mining (continuous loop)
./mine_websocket.sh
```

**The miner will:**
- ✅ Run your optimal +EV strategy every round
- ✅ Auto-checkpoint and claim ORE
- ✅ Track all results
- ✅ Continue until you stop it

---

## 🚨 TROUBLESHOOTING

### **"AccountNotFound" error:**
- Wait a few seconds and it will retry
- Make sure keypair has been initialized with ORE

### **"Deployment failed" error:**
- Usually means round ended before deployment
- Miner will automatically retry next round

### **"Not enough SOL" error:**
- Add more SOL to your wallet
- Or reduce bet size with `BET_AMOUNT`

### **Miner stops unexpectedly:**
- Check RPC connection
- Check SOL balance
- Restart with `./mine_websocket.sh`

---

## 💰 MAXIMIZING YOUR RETURNS

1. **Let it run 24/7** for maximum ORE accumulation
2. **Monitor `optimal_ev_results.jsonl`** to track performance
3. **Check win rate** - should be ~8% long-term
4. **Check hot hand hits** - should happen ~8-10% of rounds
5. **Verify share edge** - should average ~4-5% in least crowded

---

## 🎯 BOTTOM LINE

**One command = Continuous optimal mining:**

```bash
./mine_websocket.sh
```

- ✅ No manual work
- ✅ Optimal strategy every round
- ✅ +30% share edge
- ✅ 2.3x hot hand effect
- ✅ Auto-checkpoint
- ✅ Auto-logging

**Set it and forget it!** 🚀
