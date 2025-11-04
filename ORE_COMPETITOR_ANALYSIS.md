# Building an ORE Competitor - Complete Analysis

## ORE v3 Source Code

**License:** Apache-2.0 (fully open source, fork-friendly)
**Repository:** https://github.com/HardhatChad/ore (regolith-labs/ore)
**Local Path:** `/home/alsk/ore/`

**Structure:**
- `api/` - State definitions and SDK
- `program/` - On-chain Solana program (Steel framework)
- `cli/` - Command line interface

---

## How ORE Works (Critical Mechanics)

### 1. The 25-Square Game

Every round:
- 25 squares available
- Miners deploy SOL to any squares they choose
- After ~10 minutes (150 slots), round ends
- Winning square selected via **Solana slot hash RNG**: `(rng % 25) as usize`
- **Losers pay winners** - all SOL from losing squares goes to winning square
- Winners split pot **proportionally** based on deployment size

**Key insight:** This is essentially a lottery where you can buy multiple tickets (squares).

### 2. Reward Distribution (Checkpoint Logic)

When you checkpoint after a round:

**SOL Rewards:**
```rust
// Get your deposit back minus 1% admin fee
rewards_sol = original_deployment - (original_deployment / 100).max(1);

// Add your share of the losers' pot
rewards_sol += (round.total_winnings × your_deployment) / total_deployment_on_winning_square;
```

**ORE Rewards (NEW! The top miner system):**
- Each round mints new ORE
- **Top miner selection:** Random sample within the winning square
  ```rust
  top_miner_sample = rng.reverse_bits() % total_deployed_on_winning_square

  // You win if the sample lands in your range
  if sample >= your_cumulative && sample < your_cumulative + your_deployment {
      you_win_all_ore_for_round
  }
  ```
- **Split rounds (50% chance):** ORE split proportionally among ALL winners instead
- **150% APR:** Unclaimed ORE earns staking rewards

**Motherlode (1/625 chance):**
```rust
if rng.reverse_bits() % 625 == 0 {
    distribute_motherlode_to_winning_square_proportionally
}
```

### 3. Economics Breakdown

**Protocol Fees:**
- 1% admin fee on all deployments
- Winners lose nothing (get full pot)
- Losers lose everything

**Unrefined ORE Staking:**
- 150% APR on unclaimed ORE rewards
- Compounds automatically
- Main value driver for holding ORE

---

## Problems with ORE (Attack Vectors for Competitors)

### Problem 1: **Whales Dominate**

Current state:
- Large players deploy 5-10 SOL per square
- Small players (<0.1 SOL) get crushed on share %
- Motherlode rewards heavily favor whales due to proportional distribution

**Example:**
- Whale deploys 5 SOL to square 12
- You deploy 0.05 SOL to square 12
- Both win
- Whale gets 99% of pot, you get 1%

### Problem 2: **Gas Inefficiency**

- Each deploy = separate transaction (~0.00001 SOL)
- Deploying to 18 squares = 18 transactions
- Small miners waste significant % on gas

### Problem 3: **No Skill Element**

Pure RNG. No way to:
- Predict which squares are safer
- Use strategy beyond "deploy to less crowded squares"
- Earn better returns through skill

### Problem 4: **Motherlode is Rare AF**

- 1/625 chance = 0.16% per round
- ~10 minute rounds = ~144 rounds/day
- Expected motherlode every ~4.3 days
- Most miners will NEVER hit it with their specific squares

### Problem 5: **Negative EV for Small Players**

As we calculated:
- 18-square strategy: -$0.55/round expected
- 25-square strategy: -$6/round on SOL (but +ORE accumulation)
- Only profitable if ORE price rises OR you hit motherlode

---

## Competitive Advantages We Could Build

### Option 1: **Fair Mining (Anti-Whale Mechanism)**

**Idea:** Cap individual deployment per square to level the playing field.

**Implementation:**
```rust
const MAX_DEPLOY_PER_SQUARE: u64 = 0.1 SOL; // 100M lamports

// In deploy instruction
if amount > MAX_DEPLOY_PER_SQUARE {
    return Err("Deployment exceeds cap");
}
```

**Benefits:**
- Small players get fair share %
- Whales can't dominate individual squares
- More democratic distribution

**Trade-offs:**
- Whales will just deploy to ALL 25 squares at max
- Might reduce total TVL

---

### Option 2: **Skill-Based Mining (Proof of Work Hybrid)**

**Idea:** Winner determined by RNG + hashpower contribution.

**Implementation:**
```rust
pub struct SquareDeploy {
    pub sol_deployed: u64,
    pub hashes_submitted: u64,  // NEW
}

// Winning calculation
let total_score = (sol_deployed * SOL_WEIGHT) + (hashes_submitted * HASH_WEIGHT);
let your_share = your_score / total_score;
```

**Benefits:**
- Miners can improve odds by submitting PoW
- CPUs/GPUs become valuable again
- Skill-based edge for technical users

**Trade-offs:**
- More complex
- Requires off-chain hash submission infrastructure
- May favor botnets

---

### Option 3: **Guaranteed Returns Pool (The "Index Fund" Approach)**

**Idea:** Instead of winner-takes-all, create pools with guaranteed yield.

**Implementation:**
```rust
pub struct GuaranteedPool {
    pub total_deposited: u64,
    pub min_apy: u16,  // e.g., 5% APY guaranteed
    pub bonus_apy: u16, // e.g., up to 50% APY if lucky
}

// Every round:
// - Take small cut from losers
// - Distribute guaranteed base yield to ALL depositors
// - Winners get bonus yield
```

**Benefits:**
- Eliminates pure gambling feel
- Attracts risk-averse capital
- More predictable returns

**Trade-offs:**
- Lower upside
- Requires treasury management
- Less exciting

---

### Option 4: **Dynamic Motherlode (Frequent Rewards)**

**Idea:** Instead of 1/625 motherlode, have smaller frequent bonuses.

**Current ORE:**
```rust
if rng % 625 == 0 {  // 0.16% chance
    distribute_motherlode  // HUGE reward
}
```

**Our Version:**
```rust
// Mini motherlode: 10% chance, small bonus
if rng % 10 == 0 {
    distribute_mini_bonus(round.total_deployed * 0.02)  // 2% bonus pool
}

// Medium motherlode: 2% chance, medium bonus
if rng % 50 == 0 {
    distribute_medium_bonus(round.total_deployed * 0.10)  // 10% bonus pool
}

// Mega motherlode: 0.5% chance, huge bonus
if rng % 200 == 0 {
    distribute_mega_bonus(round.total_deployed * 0.50)  // 50% bonus pool
}
```

**Benefits:**
- More frequent "wins" = better UX
- Dopamine hits keep users engaged
- Actual statistical relevance (you'll hit bonuses)

**Trade-offs:**
- Less "moonshot" appeal
- More complex payout structure

---

### Option 5: **Improved Tokenomics (Better Staking)**

**Current ORE:** 150% APR on unrefined ORE

**Our Version:**
```rust
pub struct ImprovedStaking {
    pub base_apy: u16,         // 200% base APR
    pub loyalty_multiplier: u8, // +10% per month staked
    pub max_multiplier: u16,    // Cap at 500% APR
    pub compound_frequency: u8, // Compound every block
}

// APR increases the longer you hold
let your_apy = base_apy + (months_staked * loyalty_multiplier).min(max_multiplier);
```

**Benefits:**
- Rewards long-term holders
- Reduces sell pressure
- Incentivizes accumulation

**Trade-offs:**
- Could create token inflation issues
- Requires careful economic modeling

---

## The Best Strategy: COMBINE MULTIPLE IMPROVEMENTS

### "PROSPEKT" - Fair Mining Protocol

**Core Innovations:**

1. **Fair Square Caps**
   - Max 0.25 SOL per square per miner
   - Prevents whale domination

2. **Frequent Bonuses**
   - 10% chance mini bonus (2% pot)
   - 2% chance medium bonus (10% pot)
   - 0.5% chance mega bonus (50% pot)
   - Better UX than rare motherlodes

3. **Loyalty Rewards**
   - Base 200% APR on unclaimed tokens
   - +20% APR per month staked (max 500%)
   - Compounds per block

4. **Gas Optimization**
   - Batch deploy instruction (1 tx for all squares)
   - 95% gas savings vs ORE

5. **Optional: Skill Mining**
   - Submit PoW hashes for bonus multiplier
   - Max 1.5x boost from hashpower
   - Still SOL-weighted, but skill helps

---

## Technical Implementation Plan

### Phase 1: Core Protocol (Week 1-2)

Using **Steel framework** (same as ORE):

**State Accounts:**
```rust
pub struct ProspektBoard {
    pub round_id: u64,
    pub start_slot: u64,
    pub end_slot: u64,
    pub strategy: Strategy,  // Fair, Skill, or Hybrid
}

pub struct ProspektRound {
    pub id: u64,
    pub deployed: [u64; 25],
    pub miner_count: [u64; 25],
    pub slot_hash: [u8; 32],
    pub mini_bonus_pool: u64,
    pub medium_bonus_pool: u64,
    pub mega_bonus_pool: u64,
}

pub struct ProspektMiner {
    pub authority: Pubkey,
    pub deployed: [u64; 25],
    pub round_id: u64,
    pub checkpoint_id: u64,
    pub rewards_sol: u64,
    pub rewards_token: u64,
    pub total_hashes: u64,  // Optional for skill mining
    pub staking_start: u64,
}
```

**Key Instructions:**
- `initialize` - Create board
- `batch_deploy` - Deploy to multiple squares in ONE tx
- `submit_hashes` - (Optional) Submit PoW for bonus
- `checkpoint` - Claim rewards
- `stake` - Stake unclaimed tokens
- `unstake` - Claim staked tokens

### Phase 2: Token Launch (Week 3)

**Tokenomics:**
- Ticker: **PSPK** (Prospekt)
- Max supply: 21M (same scarcity as BTC)
- Emission: 100 PSPK per round initially, halves every 100k rounds
- Staking APR: 200-500% on unclaimed

**Distribution:**
- 90% mining rewards
- 5% treasury
- 5% team (6 month lockup)

### Phase 3: Frontend (Week 4)

- Web app (Next.js + Solana Wallet Adapter)
- Real-time round visualization
- Profitability calculator
- Auto-mine bot (like your current setup)

---

## Why This Beats ORE

| Feature | ORE v3 | PROSPEKT |
|---------|--------|----------|
| **Whale domination** | Yes (unlimited SOL per square) | No (0.25 SOL cap) |
| **Gas costs** | High (1 tx per square) | Low (1 tx for all) |
| **Bonus frequency** | 0.16% (motherlode) | 12.5% (combined bonuses) |
| **Staking APR** | 150% flat | 200-500% scaling |
| **Skill element** | None (pure RNG) | Optional (PoW boost) |
| **Expected value** | -EV for small players | +EV with loyalty staking |

---

## Next Steps

### Option A: Fork ORE Directly
- Clone `/home/alsk/ore/`
- Modify deploy.rs, checkpoint.rs, and state
- Change constants and add batch instruction
- Deploy as "PROSPEKT"

### Option B: Build From Scratch with Steel
- Use ORE as reference
- Implement only the best features
- Cleaner codebase
- More flexibility

### Option C: Hybrid - Pool on Top of ORE
- Build smart pools that use ORE underneath
- Add skill-based entry (pay to join good pools)
- Less risky, faster to market

---

## My Recommendation

**Build PROSPEKT as a direct ORE fork with:**

1. ✅ Batch deploy (easy win, massive UX improvement)
2. ✅ Fair square caps (0.25 SOL max)
3. ✅ Frequent bonus tiers (mini/medium/mega)
4. ✅ Loyalty staking (200-500% APR)
5. ⏳ Skip PoW for v1 (add later if demand)

**Timeline:**
- Week 1-2: Core protocol
- Week 3: Token + deploy to devnet
- Week 4: Frontend + testing
- Week 5: Mainnet launch

**Capital needed:**
- ~2 SOL for devnet testing
- ~50 SOL for initial liquidity pool
- ~10 SOL for audits (optional but recommended)

---

## Ready to Build?

We have:
- ✅ Full ORE source code locally
- ✅ Steel framework experience
- ✅ Understanding of the mechanics
- ✅ Your existing mining infrastructure

**Want me to start building?** I can have the core protocol done in 2 weeks.

What features should we prioritize?
