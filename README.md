# ORE Auto Miner

Automated ORE mining with optimal strategies and real-time WebSocket monitoring.

## Features

- **Automated Mining**: WebSocket-driven continuous mining with automatic round detection
- **Flexible Strategies**: Configurable square selection algorithms (10, 13, or 18 squares)
- **Real-time Monitoring**: Live tracking of deployments and rewards
- **Easy Setup**: Simple environment variable configuration

## Quick Start

### 1. Setup Environment

```bash
# Copy example env file
cp .env.example .env

# Edit .env with your credentials
nano .env
```

Required environment variables:
- `KEYPAIR`: Path to your Solana keypair JSON file
- `RPC`: Your Solana RPC endpoint (Helius recommended)
- `COMMAND`: Mining command (default: `deploy_optimal`)

### 2. Install Dependencies

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build --release
```

### 3. Start Mining

```bash
# Using the automated script (recommended)
./mine_websocket.sh 0.02  # Deploy 0.02 SOL per round

# Or manually
export KEYPAIR=/path/to/keypair.json
export RPC=https://mainnet.helius-rpc.com/?api-key=YOUR_KEY
export COMMAND=deploy_optimal
cargo run --release
```

## Mining Strategies

This miner is built on top of the base ORE protocol and implements square selection algorithms. The core strategy focuses on two key techniques:

1. **Least Crowded Selection**: Analyzes the board and deploys to the least densely bought squares for better share percentage per deployment
2. **Late Snipe Timing**: Deploys in the last 5-10 seconds before round end, giving you maximum information about board state before committing

This combination allows you to see exactly where the least competition is and deploy right before the round closes.

### Build Your Own Strategy

The codebase is designed to be extensible. Check out [cli/src/deploy_optimal_ev.rs](cli/src/deploy_optimal_ev.rs) to see how strategies are implemented. You can:

- Modify the `select_optimal_squares()` function to implement your own selection logic
- Add filters based on previous winners, square patterns, or custom heuristics
- Adjust timing, deployment amounts, and risk parameters
- Experiment with different combinations of squares

The base implementation provides WebSocket monitoring, automatic checkpointing, and transaction submission - you just focus on the strategy!

### Configurable Square Coverage

You can configure how many squares to deploy to based on your preferences and bankroll:

#### 18-Square Strategy
- **Coverage**: 72% of board
- **Config**: `export NUM_SQUARES=18`

#### 13-Square Strategy
- **Coverage**: 52% of board
- **Config**: `export NUM_SQUARES=13`

#### 10-Square Strategy
- **Coverage**: 40% of board
- **Config**: `export NUM_SQUARES=10` (default)

Test different configurations and track your own results to find what works best for your situation.

## Available Commands

### Mining Commands
```bash
# Deploy optimal strategy
COMMAND=deploy_optimal cargo run --release

# Check current board state
COMMAND=board cargo run --release

# Check your miner status
COMMAND=miner cargo run --release

# Claim rewards
COMMAND=claim cargo run --release

# Check treasury/motherlode
COMMAND=treasury cargo run --release
```

### Helper Scripts

- `./mine_websocket.sh [SOL_AMOUNT]` - Continuous automated mining
- `./deploy_optimal.sh` - Single round optimal deployment
- `./setup_keypair.sh` - Interactive keypair setup
- `./run-tui.sh` - Terminal UI (experimental)

## How It Works

1. **WebSocket Monitoring**: Connects to Solana WebSocket for real-time round updates
2. **Board Analysis**: Continuously monitors which squares have the least density (deployments)
3. **Late Snipe**: Waits until 5-10 seconds before round end to deploy
4. **Optimal Selection**: Deploys to N least densely bought squares for best share ratio
5. **Auto Checkpoint**: Automatically claims rewards from completed rounds
6. **Continuous Loop**: Repeats for next round

## Configuration

### Environment Variables

```bash
# Required
KEYPAIR=/path/to/keypair.json
RPC=https://mainnet.helius-rpc.com/?api-key=YOUR_KEY

# Optional
COMMAND=deploy_optimal          # Command to run
NUM_SQUARES=10                  # Number of squares (10, 13, or 18)
BET_AMOUNT=20000000            # Total deployment in lamports (0.02 SOL)
```

### Strategy Selection

Edit `NUM_SQUARES` environment variable to switch strategies:

```bash
# 10 squares (default)
export NUM_SQUARES=10

# 13 squares
export NUM_SQUARES=13

# 18 squares
export NUM_SQUARES=18
```

## Project Structure

```
ore/
├── api/              # ORE protocol API
├── cli/              # Mining CLI tool
│   └── src/
│       ├── main.rs                    # Entry point
│       ├── websocket.rs               # WebSocket monitoring
│       ├── deploy_optimal_ev.rs       # Main mining logic
│       └── strategies.rs              # Square selection algorithms
├── program/          # Smart contract code
├── .env.example      # Environment template
├── mine_websocket.sh # Auto mining script
└── README.md         # This file
```

## Documentation

- [Quick Start Guide](QUICKSTART_AUTO_MINE.md) - Step-by-step setup
- [TUI Guide](TUI_GUIDE.md) - Terminal UI usage
- [Strategy Documentation](STRATEGY_18_SQUARES.md) - Strategy examples and analysis

## Security

- Never commit your `.env` file
- Never share your keypair JSON files
- Use environment variables for all secrets
- The `.gitignore` is configured to protect sensitive files

## Support

For issues or questions, please open an issue on GitHub.

## Contributing

Feel free to fork this repository and experiment with your own strategies! If you discover something interesting, open a PR or share your findings.

The beauty of this codebase is that it provides all the infrastructure (WebSocket monitoring, transaction handling, checkpointing) so you can focus purely on strategy development.

## License

See upstream ORE protocol for license information.

---

**Happy Mining!** ⛏️

May your squares be least crowded and your motherlode hits be frequent!
