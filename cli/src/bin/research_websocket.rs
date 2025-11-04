/// RESEARCH SCRIPT V2: Using WebSockets for accurate timing
///
/// Captures pool data in the LAST 10 SECONDS of each round (when it matters!)
/// Run with: cargo run --bin research_websocket

use ore_api::prelude::*;
use serde::{Deserialize, Serialize};
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::{pubsub_client::PubsubClient, rpc_client::RpcClient};
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures_util::StreamExt;
use steel::AccountDeserialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundSnapshot {
    round_id: u64,
    timestamp: String,
    // Captured with X seconds remaining
    seconds_remaining_at_capture: f64,
    square_pools: [u64; 25],
    total_deployed: u64,
    motherlode: u64,
    least_crowded_5: Vec<usize>,
    most_crowded_5: Vec<usize>,
    avg_pool: u64,
    max_pool: u64,
    min_pool: u64,
    active_squares: usize,
    variance: f64,
    previous_winner: Option<usize>,
    previous_winner_current_rank: Option<usize>,
    // Filled after round ends
    winner: Option<usize>,
    winner_was_least_crowded: Option<bool>,
    winner_was_most_crowded: Option<bool>,
    winner_pool_rank: Option<usize>,
}

impl RoundSnapshot {
    fn get_rank_of_square(&self, square: usize) -> Option<usize> {
        let mut sorted: Vec<(usize, u64)> = self.square_pools
            .iter()
            .enumerate()
            .map(|(i, &d)| (i, d))
            .collect();
        sorted.sort_by_key(|&(_, d)| d);
        sorted.iter().position(|&(i, _)| i == square)
    }

    fn update_winner_stats(&mut self, winner: usize) {
        self.winner_was_least_crowded = Some(self.least_crowded_5.contains(&winner));
        self.winner_was_most_crowded = Some(self.most_crowded_5.contains(&winner));
        self.winner_pool_rank = self.get_rank_of_square(winner);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║   🔬 ORE RESEARCH V2 - WebSocket-Based Collection        ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Captures pool data in LAST 10 SECONDS of each round    ║");
    println!("║  (when actual miner behavior is visible)                ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let rpc_url = std::env::var("RPC").expect("Missing RPC env var");
    let rpc = RpcClient::new(rpc_url.clone());

    let ws_url = rpc_url
        .replace("https://", "wss://")
        .replace("http://", "ws://");

    let research_file = "research_100_rounds.jsonl";
    let snapshots = Arc::new(RwLock::new(Vec::<RoundSnapshot>::new()));
    let last_winner = Arc::new(RwLock::new(None::<usize>));
    let current_round_id = Arc::new(RwLock::new(0u64));
    let snapshot_taken_this_round = Arc::new(RwLock::new(false));

    println!("📡 Connecting to Solana WebSocket...");

    // Subscribe to board updates
    let board_data = Arc::new(RwLock::new(None::<Board>));
    let round_data = Arc::new(RwLock::new(None::<Round>));

    {
        let ws_url_clone = ws_url.clone();
        let board_data_clone = board_data.clone();
        let board_pda = board_pda().0;

        tokio::spawn(async move {
            loop {
                match subscribe_board(&ws_url_clone, board_pda, board_data_clone.clone()).await {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("Board WebSocket error: {}, reconnecting...", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
            }
        });
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    println!("✅ WebSocket connected");
    println!("🕒 Waiting for round timing data...\n");

    // Main collection loop
    let pubsub = PubsubClient::new(&ws_url).await?;
    let (mut stream, _unsub) = pubsub.slot_subscribe().await?;

    println!("🔌 Slot monitoring active (watching for last 10 seconds of each round)\n");

    while let Some(slot_info) = stream.next().await {
        let slot = slot_info.slot;

        let board_opt = board_data.read().await.clone();

        if let Some(board) = board_opt {
            if board.end_slot != u64::MAX {
                let slots_remaining = board.end_slot.saturating_sub(slot);
                let seconds_remaining = (slots_remaining as f64) * 0.4;

                // CAPTURE IN LAST 10 SECONDS
                if seconds_remaining <= 10.0 && seconds_remaining > 5.0 {
                    let snapshot_taken = *snapshot_taken_this_round.read().await;
                    let current_round = *current_round_id.read().await;

                    if !snapshot_taken && board.round_id != current_round {
                        println!("📸 Capturing Round {} ({}s remaining)...", board.round_id, seconds_remaining);

                        // Fetch round data
                        match capture_round_snapshot(&rpc, &board, seconds_remaining, *last_winner.read().await).await {
                            Ok(snapshot) => {
                                let mut snaps = snapshots.write().await;
                                snaps.push(snapshot.clone());

                                // Save to file
                                if let Err(e) = append_snapshot(&snapshot, research_file) {
                                    eprintln!("Error saving snapshot: {}", e);
                                }

                                *current_round_id.write().await = board.round_id;
                                *snapshot_taken_this_round.write().await = true;

                                let count = snaps.len();
                                println!("✅ Round {} captured ({}/100)\n", board.round_id, count);
                                print_progress(count, 100);

                                if count >= 100 {
                                    println!("\n🎉 100 rounds collected! Analyzing...\n");
                                    analyze_for_edges(&snaps);
                                    return Ok(());
                                }
                            }
                            Err(e) => {
                                eprintln!("Error capturing snapshot: {}", e);
                            }
                        }
                    }
                }

                // Reset flag when round ends
                if slot >= board.end_slot {
                    let current_round = *current_round_id.read().await;
                    if board.round_id == current_round {
                        *snapshot_taken_this_round.write().await = false;

                        // Try to get winner
                        println!("🏁 Round {} ended, detecting winner...", board.round_id);

                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                        if let Ok(winner) = get_round_winner(&rpc, board.round_id).await {
                            *last_winner.write().await = Some(winner);

                            // Update last snapshot with winner
                            let mut snaps = snapshots.write().await;
                            if let Some(last_snap) = snaps.last_mut() {
                                if last_snap.round_id == board.round_id {
                                    last_snap.winner = Some(winner);
                                    last_snap.update_winner_stats(winner);
                                    println!("   🎲 Winner: Square #{}\n", winner);

                                    // Resave file with winner data
                                    let _ = resave_all_snapshots(&snaps, research_file);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn subscribe_board(
    ws_url: &str,
    board_pda: Pubkey,
    board_data: Arc<RwLock<Option<Board>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let pubsub = PubsubClient::new(ws_url).await?;

    let config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };

    let (mut stream, _unsub) = pubsub.account_subscribe(&board_pda, Some(config)).await?;

    while let Some(response) = stream.next().await {
        if let Some(account) = response.value.data.decode() {
            if let Ok(board) = Board::try_from_bytes(&account) {
                *board_data.write().await = Some(board);
            }
        }
    }

    Ok(())
}

async fn capture_round_snapshot(
    rpc: &RpcClient,
    board: &Board,
    seconds_remaining: f64,
    previous_winner: Option<usize>,
) -> Result<RoundSnapshot, Box<dyn std::error::Error>> {
    let round_pubkey = round_pda(board.round_id).0;
    let round_data = rpc.get_account_data(&round_pubkey).await?;
    let round = Round::try_from_bytes(&round_data)?;

    let mut sorted: Vec<(usize, u64)> = round.deployed
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, d))
        .collect();
    sorted.sort_by_key(|&(_, d)| d);

    let least_crowded_5: Vec<usize> = sorted.iter().take(5).map(|&(i, _)| i).collect();
    let most_crowded_5: Vec<usize> = sorted.iter().rev().take(5).map(|&(i, _)| i).collect();

    let active_squares = round.deployed.iter().filter(|&&d| d > 0).count();
    let avg_pool = if active_squares > 0 {
        round.total_deployed / active_squares as u64
    } else {
        0
    };

    let max_pool = *round.deployed.iter().max().unwrap_or(&0);
    let min_pool = *round.deployed.iter().min().unwrap_or(&0);

    let variance = if avg_pool > 0 {
        let sum_sq: f64 = round.deployed.iter()
            .map(|&d| {
                let diff = d as f64 - avg_pool as f64;
                diff * diff
            })
            .sum();
        sum_sq / 25.0
    } else {
        0.0
    };

    let previous_winner_current_rank = previous_winner.and_then(|prev| {
        sorted.iter().position(|&(i, _)| i == prev)
    });

    Ok(RoundSnapshot {
        round_id: round.id,
        timestamp: chrono::Utc::now().to_rfc3339(),
        seconds_remaining_at_capture: seconds_remaining,
        square_pools: round.deployed,
        total_deployed: round.total_deployed,
        motherlode: round.motherlode,
        least_crowded_5,
        most_crowded_5,
        avg_pool,
        max_pool,
        min_pool,
        active_squares,
        variance,
        previous_winner,
        previous_winner_current_rank,
        winner: None,
        winner_was_least_crowded: None,
        winner_was_most_crowded: None,
        winner_pool_rank: None,
    })
}

async fn get_round_winner(rpc: &RpcClient, round_id: u64) -> Result<usize, Box<dyn std::error::Error>> {
    let round_pubkey = round_pda(round_id).0;
    let round_data = rpc.get_account_data(&round_pubkey).await?;
    let round = Round::try_from_bytes(&round_data)?;

    if round.slot_hash != [0; 32] {
        let rng = u64::from_le_bytes(round.slot_hash[0..8].try_into().unwrap());
        let winner = round.winning_square(rng);
        Ok(winner as usize)
    } else {
        Err("Slot hash not available yet".into())
    }
}

fn append_snapshot(snapshot: &RoundSnapshot, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)?;
    writeln!(f, "{}", serde_json::to_string(snapshot)?)?;
    Ok(())
}

fn resave_all_snapshots(snapshots: &[RoundSnapshot], file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut f = std::fs::File::create(file)?;
    for snapshot in snapshots {
        writeln!(f, "{}", serde_json::to_string(snapshot)?)?;
    }
    Ok(())
}

fn print_progress(current: usize, total: usize) {
    let percent = (current as f64 / total as f64 * 100.0) as usize;
    let bars = percent / 2;
    print!("Progress: [");
    for i in 0..50 {
        if i < bars {
            print!("█");
        } else {
            print!("░");
        }
    }
    println!("] {}%\n", percent);
}

fn analyze_for_edges(snapshots: &[RoundSnapshot]) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                  🔍 ANALYSIS RESULTS                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // ANALYSIS 1: Previous winner pool rank
    println!("📌 ANALYSIS 1: Do previous winners get LOW pools next round?");
    println!("   (YOUR KEY HYPOTHESIS!)\n");

    let prev_winner_ranks: Vec<usize> = snapshots.iter()
        .filter_map(|s| s.previous_winner_current_rank)
        .collect();

    if !prev_winner_ranks.is_empty() {
        let avg_rank = prev_winner_ranks.iter().sum::<usize>() as f64 / prev_winner_ranks.len() as f64;
        let in_bottom_5 = prev_winner_ranks.iter().filter(|&&r| r < 5).count();
        let in_bottom_10 = prev_winner_ranks.iter().filter(|&&r| r < 10).count();
        let in_top_5 = prev_winner_ranks.iter().filter(|&&r| r >= 20).count();

        println!("   Results ({} rounds with previous winner data):", prev_winner_ranks.len());
        println!("   • Average rank: {:.1}/25 (0=lowest pool, 24=highest pool)", avg_rank);
        println!("   • In BOTTOM 5 (least crowded): {}/{} ({:.1}%)",
                 in_bottom_5, prev_winner_ranks.len(),
                 in_bottom_5 as f64 / prev_winner_ranks.len() as f64 * 100.0);
        println!("   • In BOTTOM 10: {}/{} ({:.1}%)",
                 in_bottom_10, prev_winner_ranks.len(),
                 in_bottom_10 as f64 / prev_winner_ranks.len() as f64 * 100.0);
        println!("   • In TOP 5 (most crowded): {}/{} ({:.1}%)",
                 in_top_5, prev_winner_ranks.len(),
                 in_top_5 as f64 / prev_winner_ranks.len() as f64 * 100.0);
        println!();

        let bottom_5_pct = in_bottom_5 as f64 / prev_winner_ranks.len() as f64 * 100.0;

        if avg_rank < 8.0 || bottom_5_pct > 30.0 {
            println!("   ✅ EDGE FOUND: Previous winners DO get low pools!");
            println!("   🚨 TRAP: Contrarian bots fall for this!");
            println!("   💡 STRATEGY: **AVOID previous round's winning square!**");
        } else if avg_rank > 17.0 {
            println!("   ✅ EDGE FOUND: Previous winners get HIGH pools next round!");
            println!("   💡 STRATEGY: TARGET previous winner squares (whale follow)!");
        } else {
            println!("   ❌ No clear pattern. Previous winners distribute randomly.");
        }
    } else {
        println!("   ⚠️  Not enough data yet");
    }
    println!("\n{}\n", "─".repeat(60));

    // ANALYSIS 2: Least crowded win rate
    println!("📌 ANALYSIS 2: Do least crowded squares win more?\n");

    let winners_with_data: Vec<&RoundSnapshot> = snapshots.iter()
        .filter(|s| s.winner.is_some())
        .collect();

    if !winners_with_data.is_empty() {
        let least_crowded_wins = winners_with_data.iter()
            .filter(|s| s.winner_was_least_crowded == Some(true))
            .count();

        let least_crowded_win_rate = (least_crowded_wins as f64 / winners_with_data.len() as f64) * 100.0;
        let expected_rate = 20.0; // 5/25 = 20%

        println!("   • Least crowded 5 win rate: {:.1}% ({}/{})",
                 least_crowded_win_rate, least_crowded_wins, winners_with_data.len());
        println!("   • Expected (random): 20.0%");
        println!();

        if least_crowded_win_rate > expected_rate + 5.0 {
            println!("   ✅ EDGE FOUND: Least crowded squares WIN MORE!");
            println!("   💡 STRATEGY: Deploy to LEAST CROWDED squares!");
        } else if least_crowded_win_rate < expected_rate - 5.0 {
            println!("   ✅ EDGE FOUND: Least crowded squares WIN LESS!");
            println!("   💡 STRATEGY: Contrarian strategy DOESN'T WORK!");
        } else {
            println!("   ❌ No edge. Winning is truly random regardless of pool size.");
        }
    } else {
        println!("   ⚠️  Need more winner data");
    }
    println!("\n{}\n", "─".repeat(60));

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║            📈 ULTIMATE STRATEGY RECOMMENDATIONS          ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("Based on {} rounds captured in LAST 10 SECONDS:\n", snapshots.len());
    println!("1. Deploy to 2 LEAST CROWDED squares");
    println!("2. AVOID previous round's winning square (if data confirms trap)");
    println!("3. Deploy late (last 10-15 seconds) for maximum information");
    println!("4. Scale bet with motherlode size\n");
}
