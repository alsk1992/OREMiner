# ORE Mining Bet Sizing Analysis

## Current Balance: 0.0312 SOL

---

## Bet Size Options

### Option 1: 0.02 SOL (0.01 per square) - CURRENT
**Rounds you can play:** ~1.5 rounds
**Pros:**
- Higher absolute returns when you win
- Takes fewer wins to get meaningful ORE accumulation
- 1.14% average share = better ORE/SOL ratio per win

**Cons:**
- ❌ EXTREME risk of ruin - you'll bust in ~2 rounds without a win
- High variance - need to win early or you're done
- No room for losing streaks (92% of rounds you lose!)

**Expected outcomes per round:**
- Win chance: 8% (2/25 squares)
- Loss chance: 92%
- Expected SOL return: 8% × 0.02 SOL × 1.14 share edge = +0.18% EV
- But variance will kill you before EV matters!

---

### Option 2: 0.004 SOL (0.002 per square) - 5x SMALLER
**Rounds you can play:** ~7.8 rounds
**Pros:**
- ✅ 7-8 rounds gives reasonable chance to see a win
- With 8% win rate, probability of ≥1 win in 8 rounds = 48.9%
- Still too risky but better survival

**Cons:**
- Still high risk of busting before first win
- Returns are small when you do win

**Expected outcomes:**
- Same 8% win rate, same +EV per round
- But 5x more rounds played = 5x more chances to realize the edge

---

### Option 3: 0.002 SOL (0.001 per square) - 10x SMALLER ✅ RECOMMENDED
**Rounds you can play:** ~15.6 rounds
**Pros:**
- ✅ ~16 rounds gives good chance to see multiple wins
- Probability of ≥1 win in 16 rounds = 71.9%
- Probability of ≥2 wins in 16 rounds = 34.8%
- Lower variance = strategy edge can actually manifest
- Room to weather the expected 92% loss rate

**Cons:**
- Slower absolute accumulation
- Takes more wins to build meaningful stack
- But you'll actually SURVIVE to get those wins!

**Expected outcomes:**
- Same 8% win rate, same +0.18% EV per round
- Expected wins in 16 rounds: 1.28 wins
- Survive long enough to actually realize your edge!

---

### Option 4: 0.001 SOL (0.0005 per square) - 20x SMALLER
**Rounds you can play:** ~31 rounds
**Pros:**
- ✅ Excellent survival odds
- Probability of ≥1 win in 31 rounds = 91.9%
- Expected wins: 2.48 wins
- Very low risk of ruin
- Strategy edge has time to work

**Cons:**
- Very slow accumulation
- Might take days to build meaningful stack
- Share % slightly worse in super small pools

**Expected outcomes:**
- Almost guaranteed to see multiple wins
- Edge compounds over many rounds

---

## MATHEMATICS: Why Smaller is Better Right Now

### Kelly Criterion (Optimal Bet Size):
```
Kelly % = (Edge × Win_Prob) / (1 - Win_Prob)
Edge = +30% share advantage
Win_Prob = 8% (2/25)

Kelly = (0.30 × 0.08) / 0.92 = 2.6% of bankroll
```

**With 0.0312 SOL bankroll:**
- Optimal Kelly bet = 0.0312 × 0.026 = **0.00081 SOL per round**
- This is closest to **Option 4: 0.001 SOL**

### Risk of Ruin Analysis:

**Current (0.02 SOL bets):**
- Probability of ZERO wins in first 1.5 rounds = (0.92)^1.5 = 88%
- **88% chance you bust before seeing a single win!**

**Option 3 (0.002 SOL bets):**
- Probability of ZERO wins in first 16 rounds = (0.92)^16 = 28%
- 72% chance you survive and see wins
- Much better!

**Option 4 (0.001 SOL bets):**
- Probability of ZERO wins in first 31 rounds = (0.92)^31 = 8%
- 92% chance you survive and realize your edge
- **This is proper bankroll management!**

---

## SIMULATION: 50 Round Outcomes

### At 0.02 SOL/round (current):
```
Round 1: Lose (-0.02) → 0.0112 SOL
Round 2: Lose (-0.02) → BUST (can't play anymore)
GAME OVER - 88% probability
```

### At 0.002 SOL/round:
```
Rounds 1-7: 6 losses, 1 win (+0.0023 net)
Expected at Round 16: 1-2 wins, down ~0.024 SOL, still alive
Survivability: 72%
```

### At 0.001 SOL/round:
```
Rounds 1-31: ~2-3 wins expected
Each win returns ~1.14x your deployment
Expected at Round 31: 2.5 wins, up ~0.002 SOL (small profit)
Survivability: 92%
Then you have runway to keep grinding the edge!
```

---

## RECOMMENDATION: Start at 0.001 SOL (Option 4)

### Why:
1. **Kelly optimal** - matches theoretical best bet size
2. **92% survival rate** vs 12% with current sizing
3. **Room for variance** - strategy needs volume to work
4. **Compound growth** - once you build to 0.05+ SOL, you can scale up

### Migration Path:
```
Phase 1 (Now): 0.001 SOL bets with 0.0312 bankroll
├─ Play ~30 rounds
├─ Expected outcome: 2-3 wins, ~break even or small profit
└─ Build confidence in strategy

Phase 2 (at 0.05 SOL): Scale to 0.002 SOL bets
├─ Play ~25 rounds
├─ Expected: 2 wins, small growth
└─ Lower risk of ruin now

Phase 3 (at 0.10+ SOL): Scale to 0.005-0.01 SOL bets
├─ Meaningful returns per win
├─ Still safe bankroll management
└─ Sustainable long-term grinding
```

---

## TLDR:

| Bet Size | Rounds | Win Prob | Risk of Ruin | Recommendation |
|----------|--------|----------|--------------|----------------|
| 0.02 SOL | 1.5    | 12%      | 88%          | ❌ TOO RISKY   |
| 0.004    | 7.8    | 49%      | 51%          | ❌ STILL RISKY |
| 0.002    | 15.6   | 72%      | 28%          | ⚠️ MARGINAL   |
| 0.001    | 31     | 92%      | 8%           | ✅ OPTIMAL     |

**Change BET_AMOUNT to 2000000 (0.002 SOL total = 0.001 per square)**

This gives you proper Kelly sizing and actual runway to realize your +EV edge!
