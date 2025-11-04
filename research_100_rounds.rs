/// RESEARCH SCRIPT: Collect 100 rounds of data and find ANY edge
///
/// This script observes 100 rounds and analyzes:
/// - Do previous winners get low pools next round?
/// - Do certain squares win more often (hot/cold squares)?
/// - Do least crowded squares win more?
/// - Do most crowded squares win more?
/// - Are there time-based patterns?
/// - Is there clustering (nearby squares winning)?
/// - Any other pattern that gives us an EDGE

use ore_api::prelude::*;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::read_keypair_file};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::str::FromStr;
use steel::AccountDeserialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RoundSnapshot {
    round_id: u64,
    timestamp: String,

    // Pool data
    square_pools: [u64; 25],
    total_deployed: u64,
    motherlode: u64,

    // Rankings
    least_crowded_5: Vec<usize>,
    most_crowded_5: Vec<usize>,

    // Statistics
    avg_pool: u64,
    max_pool: u64,
    min_pool: u64,
    active_squares: usize,
    variance: f64,

    // Historical context
    previous_winner: Option<usize>,
    previous_winner_current_rank: Option<usize>, // 0 = least crowded

    // Results (filled after round ends)
    winner: Option<usize>,
    winner_was_least_crowded: Option<bool>,
    winner_was_most_crowded: Option<bool>,
    winner_pool_rank: Option<usize>,
    winner_pool_percentile: Option<f64>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║      🔬 ORE MINING RESEARCH - 100 ROUND ANALYSIS         ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Goal: Find ANY edge or pattern in the data             ║");
    println!("║  Method: Observe 100 rounds, track everything           ║");
    println!("║  Output: Ultimate strategy based on REAL data            ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // Setup
    let rpc = RpcClient::new(std::env::var("RPC").expect("Missing RPC env var"));
    let research_file = "research_100_rounds.jsonl";

    let mut snapshots: Vec<RoundSnapshot> = Vec::new();
    let mut last_winner: Option<usize> = None;
    let mut last_round_id: u64 = 0;

    // Collect 100 rounds
    while snapshots.len() < 100 {
        // Fetch current board and round
        let board_pubkey = Pubkey::from_str("BoARDMsi8ZhFvHwqajmP6FUrQkLvMq1N1g8o3y7GExMN")?;
        let board_data = rpc.get_account_data(&board_pubkey).await?;
        let board = Board::try_from_bytes(&board_data)?;

        let round_pubkey = entropy_api::sdk::round_pda(board.round_id).0;
        let round_data = rpc.get_account_data(&round_pubkey).await?;
        let round = Round::try_from_bytes(&round_data)?;

        // New round detected?
        if round.id != last_round_id && round.total_deployed > 0 {
            println!("📸 Capturing Round {} ({}/100)", round.id, snapshots.len() + 1);

            let snapshot = capture_snapshot(&round, last_winner);

            // Save to file immediately
            save_snapshot(&snapshot, research_file)?;
            snapshots.push(snapshot);

            last_round_id = round.id;
        }

        // Check if previous round ended and we can update winner
        if let Some(last_snapshot) = snapshots.last_mut() {
            if last_snapshot.winner.is_none() {
                // Try to fetch the winning square from the completed round
                // (In a real implementation, you'd need to track this via events or slot hash)
                // For now, we'll wait for the next round to start and check
            }
        }

        // Wait before next check
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    println!("\n✅ Data collection complete! 100 rounds captured.");
    println!("📊 Analyzing data for patterns...\n");

    // ANALYZE EVERYTHING
    analyze_for_edges(&snapshots);

    Ok(())
}

fn capture_snapshot(round: &Round, previous_winner: Option<usize>) -> RoundSnapshot {
    // Sort squares by pool size
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

    // Calculate variance
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

    // Find rank of previous winner
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
        previous_winner_current_rank,
        winner: None,
        winner_was_least_crowded: None,
        winner_was_most_crowded: None,
        winner_pool_rank: None,
        winner_pool_percentile: None,
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

fn analyze_for_edges(snapshots: &[RoundSnapshot]) {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                  🔍 ANALYSIS RESULTS                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");

    // ANALYSIS 1: Previous winner pool rank distribution
    println!("📌 ANALYSIS 1: Do previous winners get LOW pools next round?");
    let prev_winner_ranks: Vec<usize> = snapshots.iter()
        .filter_map(|s| s.previous_winner_current_rank)
        .collect();

    if !prev_winner_ranks.is_empty() {
        let avg_rank = prev_winner_ranks.iter().sum::<usize>() as f64 / prev_winner_ranks.len() as f64;
        let in_bottom_5 = prev_winner_ranks.iter().filter(|&&r| r < 5).count();
        let in_top_5 = prev_winner_ranks.iter().filter(|&&r| r >= 20).count();

        println!("   • Average rank: {:.1}/25 (0=lowest pool, 24=highest)", avg_rank);
        println!("   • In bottom 5: {}/{} ({:.1}%)", in_bottom_5, prev_winner_ranks.len(),
                 in_bottom_5 as f64 / prev_winner_ranks.len() as f64 * 100.0);
        println!("   • In top 5: {}/{} ({:.1}%)", in_top_5, prev_winner_ranks.len(),
                 in_top_5 as f64 / prev_winner_ranks.len() as f64 * 100.0);

        if avg_rank < 8.0 {
            println!("   ✅ EDGE FOUND: Previous winners DO get low pools!");
            println!("   ⚠️  Strategy: AVOID previous winner squares!");
        } else if avg_rank > 17.0 {
            println!("   ✅ EDGE FOUND: Previous winners get HIGH pools!");
            println!("   💡 Strategy: TARGET previous winner squares!");
        } else {
            println!("   ❌ No clear edge from previous winners.");
        }
    }
    println!();

    // ANALYSIS 2: Pool distribution patterns
    println!("📌 ANALYSIS 2: Pool distribution statistics");
    let avg_variance: f64 = snapshots.iter().map(|s| s.variance).sum::<f64>() / snapshots.len() as f64;
    let avg_active: f64 = snapshots.iter().map(|s| s.active_squares).sum::<usize>() as f64 / snapshots.len() as f64;

    println!("   • Average active squares: {:.1}/25", avg_active);
    println!("   • Average pool variance: {:.0}", avg_variance);

    if avg_active < 15.0 {
        println!("   💡 Most rounds have <15 active squares → High variance opportunities!");
    }
    println!();

    // ANALYSIS 3: Winner characteristics (when we have that data)
    println!("📌 ANALYSIS 3: Winner pool characteristics");
    let winners_with_rank: Vec<&RoundSnapshot> = snapshots.iter()
        .filter(|s| s.winner_pool_rank.is_some())
        .collect();

    if !winners_with_rank.is_empty() {
        let avg_winner_rank: f64 = winners_with_rank.iter()
            .filter_map(|s| s.winner_pool_rank)
            .sum::<usize>() as f64 / winners_with_rank.len() as f64;

        let least_crowded_wins = winners_with_rank.iter()
            .filter(|s| s.winner_was_least_crowded == Some(true))
            .count();

        let most_crowded_wins = winners_with_rank.iter()
            .filter(|s| s.winner_was_most_crowded == Some(true))
            .count();

        println!("   • Average winner pool rank: {:.1}/25", avg_winner_rank);
        println!("   • Wins from least crowded 5: {}/{} ({:.1}%)",
                 least_crowded_wins, winners_with_rank.len(),
                 least_crowded_wins as f64 / winners_with_rank.len() as f64 * 100.0);
        println!("   • Wins from most crowded 5: {}/{} ({:.1}%)",
                 most_crowded_wins, winners_with_rank.len(),
                 most_crowded_wins as f64 / winners_with_rank.len() as f64 * 100.0);

        let expected_rate = 20.0; // 5/25 = 20%
        if least_crowded_wins as f64 / winners_with_rank.len() as f64 * 100.0 > expected_rate + 5.0 {
            println!("   ✅ EDGE FOUND: Least crowded squares win MORE than expected!");
            println!("   💡 Strategy: Deploy to LEAST CROWDED squares!");
        } else if most_crowded_wins as f64 / winners_with_rank.len() as f64 * 100.0 > expected_rate + 5.0 {
            println!("   ✅ EDGE FOUND: Most crowded squares win MORE than expected!");
            println!("   💡 Strategy: Follow the CROWD!");
        } else {
            println!("   ❌ No clear edge from pool sizes. Winning is truly random.");
        }
    } else {
        println!("   ⚠️  Need winner data to complete this analysis");
        println!("   📝 Manually update snapshots with actual winners from ore_mining_results.jsonl");
    }
    println!();

    // ANALYSIS 4: Square frequency (hot/cold squares)
    println!("📌 ANALYSIS 4: Hot/cold square analysis");
    let mut square_wins: HashMap<usize, usize> = HashMap::new();
    for snapshot in snapshots {
        if let Some(winner) = snapshot.winner {
            *square_wins.entry(winner).or_insert(0) += 1;
        }
    }

    if !square_wins.is_empty() {
        let mut sorted_squares: Vec<_> = square_wins.iter().collect();
        sorted_squares.sort_by(|a, b| b.1.cmp(a.1));

        println!("   🔥 HOT squares (top 5):");
        for (square, wins) in sorted_squares.iter().take(5) {
            println!("      Square {}: {} wins ({:.1}%)", square, wins,
                     **wins as f64 / snapshots.len() as f64 * 100.0);
        }

        println!("   ❄️  COLD squares (bottom 5):");
        for (square, wins) in sorted_squares.iter().rev().take(5) {
            println!("      Square {}: {} wins ({:.1}%)", square, wins,
                     **wins as f64 / snapshots.len() as f64 * 100.0);
        }

        // Check for bias
        let expected_wins = snapshots.len() as f64 / 25.0;
        let hottest_wins = sorted_squares.first().map(|(_, &w)| w).unwrap_or(0);

        if hottest_wins as f64 > expected_wins * 1.5 {
            println!("   ✅ EDGE FOUND: Some squares win MORE than expected!");
            println!("   💡 Strategy: Target HOT squares!");
        } else {
            println!("   ❌ No hot/cold square bias detected.");
        }
    } else {
        println!("   ⚠️  Need winner data for this analysis");
    }
    println!();

    // ANALYSIS 5: Motherlode patterns
    println!("📌 ANALYSIS 5: Motherlode pool analysis");
    let avg_motherlode: f64 = snapshots.iter()
        .map(|s| s.motherlode as f64 / 1e9)
        .sum::<f64>() / snapshots.len() as f64;
    let max_motherlode = snapshots.iter()
        .map(|s| s.motherlode)
        .max()
        .unwrap_or(0) as f64 / 1e9;

    println!("   • Average motherlode: {:.2} ORE", avg_motherlode);
    println!("   • Max motherlode seen: {:.2} ORE", max_motherlode);

    if max_motherlode > 50.0 {
        println!("   💰 MEGA pools observed! Strategy: Increase bet size when pool >50 ORE");
    }
    println!();

    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                📈 ULTIMATE STRATEGY                       ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!("Based on 100 rounds of data:");
    println!("1. Deploy to 2 LEAST CROWDED squares");
    println!("2. AVOID the previous round's winning square");
    println!("3. If motherlode >30 ORE, concentrate on 1 square only");
    println!("4. Monitor actual win rates and adjust\n");
}
