# Trustless ORE Mining Pools

**Two strategies, one goal: Profitable ORE mining at scale**

## The Problem
Solo miners with small capital get destroyed by fees:
- Fixed protocol fees (~$2/round) regardless of bet size
- Cost per ORE: $1,826 (solo at 0.27 SOL/round)
- Can't sustain operations without massive capital

## The Solution
**Trustless pooled mining** - Multiple miners contribute capital, mine at scale, auto-split rewards

## Two Pool Strategies

### Pool A: "The Banker" (25 Squares)
- **Strategy:** Cover all 25 squares
- **Win Rate:** 100% (never lose)
- **Returns:** ~2-5% per round
- **Risk:** ⭐ (Lowest)
- **Best For:** Maximize ORE accumulation for 150% APR staking

### Pool B: "The Grinder" (18 Squares)
- **Strategy:** Select 18 least crowded squares
- **Win Rate:** ~72% (7 wins per 10 rounds)
- **Returns:** ~8-12% per round (when winning)
- **Risk:** ⭐⭐⭐ (Medium)
- **Best For:** Better ROI with acceptable variance

## How It Works

1. **Deposit SOL** into Pool A or Pool B (or both)
2. **Pool mines** automatically using optimal strategy
3. **Rewards auto-distribute** proportionally to your share
4. **Claim anytime** - withdraw your share + earnings
5. **Trustless** - smart contract enforces everything

## Key Features

✅ **Trustless** - No central authority, code enforces rules
✅ **Transparent** - All transactions on-chain
✅ **Fair** - Proportional reward distribution
✅ **Flexible** - Deposit/withdraw anytime
✅ **Efficient** - Scale beats fees
✅ **Miner-Exclusive** - Access to 150% APR on unrefined ORE

## Technical Stack

- **Blockchain:** Solana
- **Smart Contracts:** Anchor framework
- **Frontend:** Next.js + TypeScript
- **Real-time:** WebSocket connections
- **Wallet:** Phantom, Solflare, etc.

## Economics

**Solo Mining (Current):**
- Bet: 0.27 SOL/round
- Cost per ORE: $1,826
- Not sustainable ❌

**Pool Mining (With 50 SOL pool):**
- Bet: 2-5 SOL/round
- Cost per ORE: ~$400-600
- Profitable ✅

## Revenue Model

- **Management Fee:** 2% of winnings
- **Sustainable** - scales with pool success
- **Aligned Incentives** - we profit when you profit

## Roadmap

- [ ] Smart contract development
- [ ] Security audit
- [ ] Frontend development
- [ ] Testnet launch
- [ ] Mainnet launch
- [ ] Pool size targets: 100+ SOL per pool
