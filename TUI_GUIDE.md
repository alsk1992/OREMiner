# ORE Mining Terminal UI (TUI)

An interactive terminal interface for mining ORE with live tracking, auto-mining, and real-time statistics.

## Features

### 🎯 Live Dashboard
- Real-time board state monitoring
- Round countdown timer
- Mining statistics (rounds mined, won, earnings)
- Auto-mine and auto-checkpoint status
- Recent activity logs

### 📊 Multiple Views
1. **Dashboard** - Overview of mining operations and stats
2. **Board** - 5x5 grid visualization with SOL deployment per square
3. **Miner** - Your miner account details and pending rewards
4. **Stake** - Staking information and yield
5. **Logs** - Full activity log with timestamps

### 🤖 Auto-Mining
- **Auto-mine mode**: Automatically deploys to new rounds
- **Auto-checkpoint**: Automatically checkpoints previous rounds
- **Multi-round support**: Continuously mines across rounds
- **Configurable amount**: Adjust deployment amount on the fly

### 📈 Real-Time Tracking
- Live board updates (refreshes every second)
- Round timer with seconds remaining
- Pending rewards tracking
- Lifetime earnings statistics

## Installation

Add the TUI dependencies (already added to Cargo.toml):

```toml
ratatui = "0.29.0"
crossterm = "0.28.1"
chrono = "0.4"
```

## Usage

### Launch the TUI

```bash
COMMAND=tui KEYPAIR=/path/to/keypair.json RPC=https://your-rpc-url cargo run
```

### Keyboard Controls

| Key | Action |
|-----|--------|
| `q` | Quit the application |
| `←` / `→` | Switch between tabs |
| `1-5` | Jump to specific tab (1=Dashboard, 2=Board, etc.) |
| `m` | Toggle auto-mine ON/OFF |
| `c` | Toggle auto-checkpoint ON/OFF |
| `d` | Manual deploy to current round |
| `r` | Claim rewards manually |
| `+` | Increase deploy amount (doubles) |
| `-` | Decrease deploy amount (halves) |

## How It Works

### Auto-Mining Flow

1. **Monitor Board**: TUI continuously monitors the board state
2. **Auto-Checkpoint**: Before joining a new round, automatically checkpoints previous rounds
3. **Auto-Deploy**: When a new round starts, automatically deploys SOL to selected squares
4. **Track Progress**: Shows real-time countdown and deployment status
5. **Auto-Claim**: Claims rewards when available

### Dashboard View

```
┌─────────────────────────────────────────────────────┐
│ Status                                              │
│ Auto-mine: ON   Auto-checkpoint: ON                │
│ Deploy amount: 0.01 SOL                            │
│ Current round: #12345                              │
│ Time remaining: 45.2s                              │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────┐
│ Mining Stats                                        │
│ Uptime: 3600s                                      │
│ Rounds mined: 60                                   │
│ Rounds won: 15                                     │
│ Total SOL earned: 0.1234                           │
│ Total ORE earned: 2.5678                           │
└─────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────┐
│ Recent Logs                                         │
│ [12:34:56] Deployed 0.01 SOL to 25 squares        │
│ [12:35:56] Claimed 0.002 SOL + 0.05 ORE           │
└─────────────────────────────────────────────────────┘
```

### Board View

Shows the 5x5 grid with:
- SOL deployed per square
- Number of miners per square
- Visual representation of board state

### Miner View

Displays:
- Your miner account authority
- Current round participation
- Checkpoint status
- Pending rewards (SOL + ORE)
- Lifetime statistics

### Stake View

Shows your staking information:
- Staked ORE balance
- Pending staking yield
- Lifetime staking rewards

### Logs View

Full activity log with:
- Timestamp for each event
- Color-coded by severity (Info, Success, Warning, Error)
- Deploy, claim, checkpoint events
- Error messages

## Configuration

### Deploy Amount

Default: 0.01 SOL per square

Adjust using `+` and `-` keys:
- `+` doubles the amount
- `-` halves the amount
- Range: 0.001 SOL to 100 SOL

### Selected Squares

Currently deploys to all 25 squares by default. Can be customized in the code:

```rust
selected_squares: [true; 25], // All squares
```

To select specific squares:
```rust
selected_squares: [
    true, false, true, false, true,  // Row 0
    false, true, false, true, false, // Row 1
    true, false, true, false, true,  // Row 2
    false, true, false, true, false, // Row 3
    true, false, true, false, true,  // Row 4
]
```

## Mining Strategy

### Conservative
- Lower deploy amount (0.001-0.01 SOL)
- Select fewer squares (5-10)
- Focus on winning small amounts consistently

### Aggressive
- Higher deploy amount (0.1-1 SOL)
- Deploy to all 25 squares
- Maximize chance of winning

### Balanced
- Medium deploy amount (0.01-0.1 SOL)
- Deploy to 15-20 squares
- Balance risk and reward

## Tips

1. **Start with Auto-Mine OFF**: Get familiar with the interface first
2. **Monitor for a few rounds**: Watch the board dynamics before deploying
3. **Check your balance**: Ensure you have enough SOL for multiple rounds
4. **Enable auto-checkpoint**: Prevents missing rewards from previous rounds
5. **Adjust amount dynamically**: Use `+`/`-` keys to optimize based on competition
6. **Watch the logs**: Monitor for errors or issues

## Troubleshooting

### "Board not loaded"
- RPC might be slow or unavailable
- Check your RPC endpoint
- Wait a few seconds for initial data fetch

### "Miner not found"
- You haven't deployed yet
- Press `d` to create miner account and deploy

### Transaction Failed
- Insufficient SOL balance
- RPC rate limiting
- Network congestion
- Check logs tab for specific error

### Auto-mine not working
- Ensure `m` shows "ON"
- Check that you're not already deployed in current round
- Verify checkpoint status

## Performance

- **Refresh rate**: 1 second
- **RPC calls**: ~6 per refresh (board, clock, miner, stake, treasury, round)
- **Transaction costs**:
  - Deploy: ~0.000005 SOL + deployed amount
  - Claim: ~0.000005 SOL
  - Checkpoint: ~0.000005 SOL

## Example Session

```bash
# 1. Start the TUI
COMMAND=tui KEYPAIR=~/.config/solana/id.json RPC=https://api.mainnet-beta.solana.com cargo run

# 2. Enable auto-mine (press 'm')
# 3. Adjust amount if needed (press '+' or '-')
# 4. Watch the dashboard
# 5. TUI will automatically:
#    - Checkpoint previous rounds
#    - Deploy to new rounds
#    - Claim rewards when available
# 6. Monitor stats and earnings
# 7. Press 'q' to quit when done
```

## Advanced: Custom Strategies

Edit `tui.rs` to implement custom strategies:

### Random squares per round
```rust
// In auto_mine_logic(), before deploy:
let num_squares = 10; // Random selection of 10 squares
let r = hashv(&[&payer.pubkey().to_bytes(), &board.round_id.to_le_bytes()]).0;
app.selected_squares = generate_random_mask(num_squares, &r);
```

### Avoid crowded squares
```rust
// Select only squares with < average deployment
let avg = round.total_deployed / 25;
for i in 0..25 {
    app.selected_squares[i] = round.deployed[i] < avg;
}
```

### Follow the whale
```rust
// Deploy to same squares as largest miner
// (requires fetching all miners and analyzing their deployments)
```

## Future Enhancements

Potential features to add:
- [ ] Square selection UI
- [ ] Multiple strategy presets
- [ ] Historical charts
- [ ] Win rate analysis
- [ ] ROI calculator
- [ ] Notification sounds
- [ ] Export stats to CSV
- [ ] Multi-wallet support

---

**Happy Mining!** 🎉⛏️
