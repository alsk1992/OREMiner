# ORE Mining Research - 100 Round Data Collection

## 🎯 Objective

Collect **100 rounds** of real mining data to discover ANY edge or pattern that gives us an advantage:
- Do previous winners get low pools next round? (Your suspicion!)
- Do certain squares win more often?
- Do least crowded squares actually win more?
- Are there time-based or clustering patterns?

## 🚀 How to Run

```bash
./run_research.sh
```

Or manually:
```bash
export KEYPAIR="/path/to/keypair.json"
export RPC="https://mainnet.helius-rpc.com/?api-key=YOUR_API_KEY"
cargo run --bin research --release
```

## ⏱️ Time Required

- **~2-3 hours** (depends on round frequency)
- Script checks every 10 seconds for new rounds
- Collects 100 rounds automatically
- Saves progress to `research_100_rounds.jsonl`

## 📊 What It Tracks

For each round:
- All 25 square pool sizes
- Total deployment
- Motherlode size
- Which squares are least/most crowded
- Previous round's winner and its current pool rank
- Statistical variance and distribution

## 🔍 Analysis Performed

After 100 rounds, the script analyzes:

### 1. Previous Winner Trap
**YOUR KEY QUESTION!**
- Does the previous winning square get a low pool next round?
- If YES → It's a TRAP for contrarian strategies!
- Strategy: **AVOID previous winner!**

### 2. Pool Distribution Patterns
- How many squares typically have deployments?
- Is there high variance (opportunity) or low variance (random)?

### 3. Least Crowded Strategy
- Do the 5 least crowded squares win more than 20% of the time?
- If YES → Contrarian strategy works!
- If NO → Winning is truly random

### 4. Hot/Cold Squares
- Do certain square positions win more often?
- Are there biased squares?

### 5. Motherlode Patterns
- How often do MEGA pools (>50 ORE) appear?
- Should we increase bet size for big pools?

## 📁 Output Files

- `research_100_rounds.jsonl` - Raw data (one JSON object per line)
- Analysis printed to console at the end

## 🎯 Next Steps After Collection

1. **Review the analysis** - Look for ANY edge
2. **Implement winning strategy** - Based on real data
3. **Monitor results** - Track actual win rate
4. **Re-run periodically** - Patterns may change as miners adapt

## 💡 Expected Insights

Based on your observation, we expect to find:

✅ **Previous winners DO get low pools next round**
- This creates a trap for naive contrarian bots
- **Our edge: Avoid previous winner + deploy to OTHER least crowded squares**

✅ **High variance in deployments**
- Most rounds have <20 active squares
- Creates opportunities for smart contrarian plays

✅ **Motherlode opportunities**
- When pool >50 ORE, deploy 3x normal bet
- Risk/reward heavily in our favor

## 🚨 Important Notes

- Script is READ-ONLY (no transactions)
- Can run safely without affecting your balance
- Uses minimal RPC calls (every 10 seconds)
- Progress is saved - can resume if interrupted
- No keypair needed for observation

## 🔬 The Scientific Method

1. **Hypothesis**: Previous winners get low pools next round
2. **Experiment**: Track 100 rounds of real data
3. **Analysis**: Calculate statistics and find patterns
4. **Strategy**: Implement based on proven edges
5. **Validation**: Monitor actual results

## 📈 Ultimate Strategy (Preliminary)

Based on game theory, we expect:
1. Deploy to 2 LEAST CROWDED squares
2. **AVOID previous round's winning square** (your key insight!)
3. Increase bet when motherlode >30 ORE
4. Concentrate (1 square) for MEGA pools (>50 ORE)

The research will **PROVE or DISPROVE** this with real data!
