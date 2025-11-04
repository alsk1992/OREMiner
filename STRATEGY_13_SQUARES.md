# Optimal Number of Squares Analysis

## Research Data: 100 Rounds

---

## Win Rate vs Theory

| Squares | Actual Win Rate | Theoretical (Random) | Difference |
|---------|-----------------|---------------------|------------|
| 1       | 1.0%            | 4.0%                | **-75%** ❌ |
| 2       | 7.0%            | 8.0%                | **-12.5%** ⚠️ |
| 3       | 13.0%           | 12.0%               | **+8.3%** ✅ |
| 4       | 17.0%           | 16.0%               | **+6.3%** ✅ |
| 5       | 21.0%           | 20.0%               | **+5.0%** ✅ |

### Key Insight:
- **1 square:** TERRIBLE win rate (1% vs 4% expected) - Too few data points or bad luck
- **2 squares:** Slightly below expected (7% vs 8%)
- **3+ squares:** At or above expected win rates

---

## Survival Analysis (with 0.0312 SOL bankroll)

| Squares | Cost/Round | Rounds Possible | Expected Wins | Survival Rate | Risk of Ruin |
|---------|------------|-----------------|---------------|---------------|--------------|
| 1       | 0.001      | 31.2            | 0.31          | **26.9%** ❌  | 73.1%        |
| 2       | 0.002      | 15.6            | 1.09          | **67.8%** ⚠️  | 32.2%        |
| 3       | 0.003      | 10.4            | 1.35          | **76.5%** ✅  | 23.5%        |
| 4       | 0.004      | 7.8             | 1.33          | **76.6%** ✅  | 23.4%        |
| 5       | 0.005      | 6.2             | 1.31          | **77.0%** ✅  | 23.0%        |

### Key Insight:
- **3-5 squares** all have ~77% survival rate (similar!)
- **2 squares** has 68% survival (decent but lower)
- **1 square** has only 27% survival (terrible!)

---

## Share Efficiency (Average Share per Square)

| Squares | Avg Share/Square | Edge vs Baseline |
|---------|------------------|------------------|
| 1       | 1.264%           | Baseline         |
| 2       | 1.215%           | -3.9%            |
| 3       | 1.169%           | -7.5%            |
| 4       | 1.133%           | -10.4%           |
| 5       | 1.106%           | -12.5%           |

### Key Insight:
- **Fewer squares = higher share per square** (less crowding)
- But this doesn't matter if you never win!
- Diminishing returns: 1→2 loses 3.9%, but 4→5 only loses 1.9%

---

## The Tradeoff: Share vs Win Rate

### 1 Square Strategy:
- ✅ **Best share:** 1.264% per square
- ❌ **Worst win rate:** 1% (need 100 rounds to expect 1 win!)
- ❌ **Worst survival:** 27%
- **Verdict:** TOO RISKY - you'll bust before seeing wins

### 2 Square Strategy (CURRENT):
- ✅ **Good share:** 1.215% per square
- ⚠️ **Okay win rate:** 7% (1 win per 14 rounds)
- ⚠️ **Okay survival:** 68%
- **Verdict:** BALANCED but still risky with small bankroll

### 3 Square Strategy:
- ✅ **Good share:** 1.169% per square (only 3.9% worse than 2-square)
- ✅ **Better win rate:** 13% (1 win per 7-8 rounds)
- ✅ **Best survival:** 76.5%
- ✅ **Most expected wins:** 1.35
- **Verdict:** BEST RISK/REWARD for small bankroll!

### 4-5 Square Strategies:
- ⚠️ **Lower share:** 1.133-1.106% per square
- ✅ **Good win rates:** 17-21%
- ✅ **Good survival:** ~77%
- ❌ **Fewer total rounds:** 6-8 rounds only
- **Verdict:** Good survival but less statistical sample

---

## Expected Wins Comparison

With your 0.0312 SOL bankroll:

| Squares | Rounds | Expected Wins | Win Frequency |
|---------|--------|---------------|---------------|
| 1       | 31.2   | 0.31          | 1 per 100 rounds |
| 2       | 15.6   | 1.09          | 1 per 14 rounds |
| 3       | 10.4   | **1.35** ✅   | 1 per 8 rounds |
| 4       | 7.8    | 1.33          | 1 per 6 rounds |
| 5       | 6.2    | 1.31          | 1 per 5 rounds |

**3 squares gives you the MOST expected wins before running out of money!**

---

## Variance Analysis

### 2 Squares:
- Win 7% of rounds
- Lose 93% of rounds
- Typical sequence: L L L L L L L W L L L L L L (1 win per 14)
- High variance - long losing streaks

### 3 Squares:
- Win 13% of rounds
- Lose 87% of rounds
- Typical sequence: L L L L L L L W L L L L L (1 win per 8)
- Lower variance - shorter losing streaks
- **More consistent feedback on if strategy is working**

### 4-5 Squares:
- Win 17-21% of rounds
- Even lower variance
- But sample size too small (only 6-8 rounds)

---

## RECOMMENDATION

### For YOUR Situation (0.0312 SOL bankroll):

## 🏆 **USE 3 SQUARES** 🏆

**Why 3 is optimal:**

1. **Best Expected Wins:** 1.35 wins (highest of all strategies!)
2. **Best Survival Rate:** 76.5% (vs 68% for 2 squares)
3. **Reasonable Sample:** 10.4 rounds (vs 6-8 for 4-5 squares)
4. **Share efficiency:** Only 3.9% worse than 2 squares
5. **Win frequency:** 1 per 8 rounds (vs 1 per 14 for 2 squares)
6. **Lower variance:** 13% win rate = less painful losing streaks

**Configuration:**
```bash
export BET_AMOUNT=3000000  # 0.003 SOL total (0.001 per square)
```

### Alternative: Stay with 2 Squares IF:

You're willing to accept:
- ❌ 32% risk of ruin (vs 23% for 3 squares)
- ❌ Longer losing streaks (1 win per 14 rounds vs 1 per 8)
- ✅ Slightly better share (1.215% vs 1.169%)
- ✅ More total rounds (15.6 vs 10.4)

**2 squares is safer than I initially thought, but 3 squares is OPTIMAL for small bankroll**

---

## When to Use Each Strategy:

| Strategy | Best For |
|----------|----------|
| 1 square | ❌ Never - too high risk of ruin |
| 2 squares | Small bankroll, want max share efficiency, can tolerate variance |
| 3 squares | **Small-medium bankroll, optimal balance** ✅ |
| 4 squares | Medium bankroll (0.05+ SOL), want consistent wins |
| 5 squares | Large bankroll (0.10+ SOL), minimize variance |

---

## TLDR:

**Current (2 squares):** 68% survival, 1.09 expected wins
**Recommended (3 squares):** 77% survival, 1.35 expected wins (+24% more wins!)

**The math says: Switch to 3 squares for your bankroll size!**
