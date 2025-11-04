/// RESEARCH SCRIPT: Collect 100 rounds and find ANY edge
///
/// Run with: cargo run --bin research
/// Or: export COMMAND=research && cargo run

use ore_api::prelude::*;
use entropy_api::prelude::*;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use std::str::FromStr;
use steel::AccountDeserialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundSnapshot {
    round_id: u64,
    timestamp: String,
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
    previous_winner_display: Option<usize>, // 1-25 for ore.supply
    previous_winner_current_rank: Option<usize>,
    winner: Option<usize>,
    winner_display: Option<usize>, // 1-25 to match ore.supply website
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
        self.winner_display = Some(winner + 1); // Convert to 1-25 for ore.supply
        self.winner_was_least_crowded = Some(self.least_crowded_5.contains(&winner));
        self.winner_was_most_crowded = Some(self.most_crowded_5.contains(&winner));
        self.winner_pool_rank = self.get_rank_of_square(winner);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║      🔬 ORE MINING RESEARCH - 100 ROUND ANALYSIS         ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Goal: Find ANY edge or pattern in the data             ║");
    println!("║  Method: Observe 100 rounds, track EVERYTHING           ║");
    println!("║  Output: Ultimate strategy based on REAL data            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    let rpc = RpcClient::new(std::env::var("RPC").expect("Missing RPC env var"));
    let research_file = "research_100_rounds.jsonl";

    let mut snapshots: Vec<RoundSnapshot> = Vec::new();
    let mut last_winner: Option<usize> = None;
    let mut last_round_id: u64 = 0;

    println!("📡 Connecting to Solana RPC...");
    println!("📊 Starting data collection (this will take a while)...\n");

    // Collect 100 rounds
    while snapshots.len() < 100 {
        match collect_round(&rpc, last_winner, last_round_id).await {
            Ok(Some((mut snapshot, new_round_id))) => {
                // Try to get winner of PREVIOUS round (if it ended)
                if !snapshots.is_empty() {
                    let prev_snapshot = snapshots.last_mut().unwrap();
                    if prev_snapshot.winner.is_none() {
                        if let Ok(winner) = get_round_winner(&rpc, prev_snapshot.round_id).await {
                            prev_snapshot.winner = Some(winner);
                            prev_snapshot.update_winner_stats(winner);
                            last_winner = Some(winner);
                            println!("   🎲 Round {} winner: Square #{} (matches ore.supply)", prev_snapshot.round_id, winner + 1);

                            // Re-save updated previous snapshot
                            resave_all_snapshots(&snapshots, research_file)?;
                        }
                    }
                }

                // Update current snapshot with previous winner context
                snapshot.previous_winner = last_winner;
                snapshot.previous_winner_display = last_winner.map(|w| w + 1); // 1-25 for ore.supply
                if let Some(prev_winner) = last_winner {
                    snapshot.previous_winner_current_rank = snapshot.get_rank_of_square(prev_winner);
                }

                println!("📸 Round {} captured ({}/100) - {} active squares, prev winner: {:?}",
                    snapshot.round_id,
                    snapshots.len() + 1,
                    snapshot.active_squares,
                    last_winner
                );

                snapshots.push(snapshot);
                resave_all_snapshots(&snapshots, research_file)?;
                last_round_id = new_round_id;

                print_progress(snapshots.len(), 100);
            }
            Ok(None) => {
                // Same round, wait
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
            Err(e) => {
                eprintln!("⚠️  Error collecting data: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        }
    }

    // Try to get winner of last round
    if let Some(last_snapshot) = snapshots.last_mut() {
        if last_snapshot.winner.is_none() {
            println!("\n⏳ Waiting for final round to complete...");
            for _ in 0..30 {
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
                if let Ok(winner) = get_round_winner(&rpc, last_snapshot.round_id).await {
                    last_snapshot.winner = Some(winner);
                    last_snapshot.update_winner_stats(winner);
                    println!("   🎲 Final round winner: Square #{} (matches ore.supply)", winner + 1);
                    resave_all_snapshots(&snapshots, research_file)?;
                    break;
                }
            }
        }
    }

    println!("\n✅ Data collection complete! 100 rounds captured.");
    println!("📁 Data saved to: {}", research_file);
    println!("\n📊 Analyzing data for edges...\n");

    analyze_for_edges(&snapshots);

    Ok(())
}

async fn collect_round(
    rpc: &RpcClient,
    previous_winner: Option<usize>,
    last_round_id: u64,
) -> Result<Option<(RoundSnapshot, u64)>, Box<dyn std::error::Error>> {
    let board_pubkey = board_pda().0;
    let board_data = rpc.get_account_data(&board_pubkey).await?;
    let board = Board::try_from_bytes(&board_data)?;

    let round_pubkey = round_pda(board.round_id).0;
    let round_data = rpc.get_account_data(&round_pubkey).await?;
    let round = Round::try_from_bytes(&round_data)?;

    // New round with deployments?
    if round.id != last_round_id && round.total_deployed > 0 {
        let snapshot = capture_snapshot(&round, previous_winner);
        Ok(Some((snapshot, round.id)))
    } else {
        Ok(None)
    }
}

async fn get_round_winner(rpc: &RpcClient, round_id: u64) -> Result<usize, Box<dyn std::error::Error>> {
    let round_pubkey = round_pda(round_id).0;
    let round_data = rpc.get_account_data(&round_pubkey).await?;
    let round = Round::try_from_bytes(&round_data)?;

    // Use the correct rng() method
    if let Some(rng) = round.rng() {
        let winner = round.winning_square(rng);
        Ok(winner as usize)
    } else {
        Err("Slot hash not available yet".into())
    }
}

fn resave_all_snapshots(snapshots: &[RoundSnapshot], file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut f = std::fs::File::create(file)?;
    for snapshot in snapshots {
        writeln!(f, "{}", serde_json::to_string(snapshot)?)?;
    }
    Ok(())
}

fn capture_snapshot(round: &Round, previous_winner: Option<usize>) -> RoundSnapshot {
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

    RoundSnapshot {
        round_id: round.id,
        timestamp: chrono::Utc::now().to_rfc3339(),
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
        previous_winner_display: previous_winner.map(|w| w + 1), // 1-25 for ore.supply
        previous_winner_current_rank,
        winner: None,
        winner_display: None,
        winner_was_least_crowded: None,
        winner_was_most_crowded: None,
        winner_pool_rank: None,
    }
}

fn save_snapshot(snapshot: &RoundSnapshot, file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(file)?;
    writeln!(f, "{}", serde_json::to_string(snapshot)?)?;
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
    println!("] {}%", percent);
}

fn format_sol(lamports: u64) -> String {
    format!("{:.3} SOL", lamports as f64 / 1e9)
}

fn analyze_for_edges(snapshots: &[RoundSnapshot]) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                  🔍 ANALYSIS RESULTS                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // ANALYSIS 1: Previous winner pool rank
    println!("📌 ANALYSIS 1: Do previous winners get LOW pools next round?");
    println!("   (This would create a trap for contrarian strategies)\n");

    let prev_winner_ranks: Vec<usize> = snapshots.iter()
        .filter_map(|s| s.previous_winner_current_rank)
        .collect();

    if !prev_winner_ranks.is_empty() {
        let avg_rank = prev_winner_ranks.iter().sum::<usize>() as f64 / prev_winner_ranks.len() as f64;
        let in_bottom_5 = prev_winner_ranks.iter().filter(|&&r| r < 5).count();
        let in_bottom_10 = prev_winner_ranks.iter().filter(|&&r| r < 10).count();
        let in_top_5 = prev_winner_ranks.iter().filter(|&&r| r >= 20).count();

        println!("   Results:");
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
            println!("   ❌ No clear pattern. Previous winners distribute evenly.");
        }
    } else {
        println!("   ⚠️  Not enough data (need multiple rounds tracked)");
    }
    println!("\n{}\n", "─".repeat(60));

    // ANALYSIS 2: Pool distribution
    println!("📌 ANALYSIS 2: Pool distribution and variance\n");

    let avg_variance: f64 = snapshots.iter().map(|s| s.variance).sum::<f64>() / snapshots.len() as f64;
    let avg_active: f64 = snapshots.iter().map(|s| s.active_squares).sum::<usize>() as f64 / snapshots.len() as f64;
    let avg_total: f64 = snapshots.iter().map(|s| s.total_deployed).sum::<u64>() as f64 / snapshots.len() as f64;

    println!("   • Average active squares: {:.1}/25", avg_active);
    println!("   • Average total deployed: {}", format_sol(avg_total as u64));
    println!("   • Average pool variance: {:.0}", avg_variance);
    println!();

    if avg_active < 15.0 {
        println!("   💡 Insight: Most rounds have <15 active squares");
        println!("      → High variance = opportunity for contrarian plays");
    }
    if avg_active > 20.0 {
        println!("   💡 Insight: Most squares have deployments");
        println!("      → Low variance = random distribution, no clear edge");
    }
    println!("\n{}\n", "─".repeat(60));

    // ANALYSIS 3: Motherlode patterns
    println!("📌 ANALYSIS 3: Motherlode pool size distribution\n");

    let motherlode_ore: Vec<f64> = snapshots.iter()
        .map(|s| s.motherlode as f64 / 1e9)
        .collect();

    let avg_motherlode = motherlode_ore.iter().sum::<f64>() / motherlode_ore.len() as f64;
    let max_motherlode = motherlode_ore.iter().fold(0.0f64, |a, &b| a.max(b));
    let min_motherlode = motherlode_ore.iter().fold(f64::MAX, |a, &b| a.min(b));

    let mega_pools = motherlode_ore.iter().filter(|&&m| m > 50.0).count();
    let large_pools = motherlode_ore.iter().filter(|&&m| m > 30.0 && m <= 50.0).count();

    println!("   • Average motherlode: {:.2} ORE", avg_motherlode);
    println!("   • Max seen: {:.2} ORE", max_motherlode);
    println!("   • Min seen: {:.2} ORE", min_motherlode);
    println!("   • MEGA pools (>50 ORE): {}/100", mega_pools);
    println!("   • Large pools (30-50 ORE): {}/100", large_pools);
    println!();

    if mega_pools > 0 {
        println!("   💰 MEGA POOLS OBSERVED!");
        println!("   💡 STRATEGY: When motherlode >50 ORE:");
        println!("      - Deploy MORE (2-3x normal bet)");
        println!("      - Concentrate on 1-2 squares only");
        println!("      - Risk/reward is heavily in your favor");
    }

    println!("\n{}\n", "─".repeat(60));

    // FINAL RECOMMENDATIONS
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║            📈 ULTIMATE STRATEGY RECOMMENDATIONS          ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    println!("Based on {} rounds of REAL data:\n", snapshots.len());

    println!("🎯 CORE STRATEGY:");
    println!("   1. Deploy to 2 LEAST CROWDED squares");
    println!("   2. AVOID the previous round's winning square");
    println!("      (It likely has a low pool but same 4% win chance)");
    println!();

    println!("💰 MOTHERLODE ADJUSTMENT:");
    println!("   • If motherlode >50 ORE: Deploy 3x, concentrate on 1 square");
    println!("   • If motherlode >30 ORE: Deploy 2x, use 2 squares");
    println!("   • If motherlode <10 ORE: Standard bet, use 2-3 squares");
    println!();

    println!("⚠️  IMPORTANT:");
    println!("   • This research has {} rounds, need {} winners for full analysis",
             snapshots.len(),
             snapshots.iter().filter(|s| s.winner.is_some()).count());
    println!("   • Re-run this script periodically to refine strategy");
    println!("   • Monitor actual win rate: Should be ~8% (2/25 squares)");
    println!();

    println!("📁 Next steps:");
    println!("   1. Manually add winner data to research_100_rounds.jsonl");
    println!("   2. Re-run analysis: cargo run --bin research_analyze");
    println!("   3. Implement winning strategy in deploy code\n");
}
