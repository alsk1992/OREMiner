# Deployment Guide

## Prerequisites

- Rust & Cargo installed
- Solana CLI installed
- Anchor CLI installed (`cargo install --git https://github.com/coral-xyz/anchor avm --locked --force`)
- Node.js 18+ and yarn
- Solana wallet with SOL for deployment

## Step 1: Build Smart Contract

```bash
cd mining-pools

# Install Anchor if not already
avm install latest
avm use latest

# Build the program
anchor build

# Get the program ID
solana address -k target/deploy/mining_pool-keypair.json
```

## Step 2: Update Program ID

1. Copy the program ID from above
2. Update in:
   - `programs/mining-pool/src/lib.rs` (declare_id!)
   - `Anchor.toml` (all [programs.XXX] sections)

## Step 3: Rebuild with Correct ID

```bash
anchor build
```

## Step 4: Deploy to Devnet (Testing)

```bash
# Set to devnet
solana config set --url devnet

# Airdrop SOL for testing
solana airdrop 2

# Deploy
anchor deploy
```

## Step 5: Initialize Pools

```bash
# Run initialization script
anchor run initialize-pools
```

This will create:
- Pool A (25-square strategy)
- Pool B (18-square strategy)

## Step 6: Deploy Frontend

```bash
cd app

# Install dependencies
yarn install

# Build for production
yarn build

# Deploy to Vercel/Netlify
# Or run locally:
yarn start
```

## Step 7: Set Up Mining Bot

The bot needs to:
1. Monitor pool balances
2. Execute mining rounds for each pool
3. Record results back to contracts

```bash
# Copy your mining bot code
cp -r ../cli mining-bot

# Update to call pool contracts
# See integration guide below
```

## Integration with ORE Mining Bot

### Modify `deploy_optimal_ev.rs`:

```rust
// Instead of mining for self, mine for pool
pub async fn mine_for_pool(
    pool_address: Pubkey,
    pool_type: PoolType,
) -> Result<()> {
    // 1. Get pool state
    let pool = get_pool_account(pool_address).await?;

    // 2. Calculate bet size based on pool balance
    let bet_amount = calculate_pool_bet(&pool);

    // 3. Select squares based on pool strategy
    let squares = match pool_type {
        PoolType::TwentyFiveSquare => select_all_squares(),
        PoolType::EighteenSquare => select_18_least_crowded(),
    };

    // 4. Execute mining round
    let result = execute_mining_round(bet_amount, squares).await?;

    // 5. Record results to pool contract
    record_pool_results(pool_address, result).await?;

    Ok(())
}
```

## Step 8: Mainnet Deployment

**⚠️ ONLY AFTER THOROUGH TESTING**

```bash
# Switch to mainnet
solana config set --url mainnet-beta

# Ensure wallet has enough SOL
solana balance

# Deploy (costs ~2-5 SOL)
anchor deploy --provider.cluster mainnet
```

## Step 9: Security Checklist

Before mainnet launch:

- [ ] Smart contract audited
- [ ] Tested on devnet for 100+ rounds
- [ ] Frontend security review
- [ ] Bot tested extensively
- [ ] Emergency pause mechanism tested
- [ ] Withdrawal mechanism verified
- [ ] Share calculation verified
- [ ] Fee collection tested

## Step 10: Launch

1. Deploy to mainnet
2. Initialize both pools
3. Deposit initial liquidity (10-20 SOL each pool)
4. Start mining bot
5. Launch frontend
6. Monitor for first 24 hours

## Monitoring

```bash
# Watch pool balances
solana account <POOL_A_ADDRESS>
solana account <POOL_B_ADDRESS>

# Check program logs
solana logs <PROGRAM_ID>
```

## Emergency Procedures

If something goes wrong:

```bash
# Pause pools
anchor run pause-pools

# Users can still withdraw
# But no new deposits or mining
```

## Costs

- **Devnet:** Free (use airdrop)
- **Mainnet deployment:** ~2-5 SOL
- **Per-transaction:** ~0.000005 SOL
- **Rent:** Minimal (already in deposit amounts)

## Support

- Check logs: `solana logs <PROGRAM_ID>`
- Test transactions on devnet first
- Monitor pool health with dashboard
