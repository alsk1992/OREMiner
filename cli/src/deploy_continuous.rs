use anyhow::Result;
use ore_api::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use std::str::FromStr;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use serde_json::json;
use chrono::Utc;

use crate::websocket::WebSocketManager;
use crate::{get_board, get_round, get_treasury, get_miner, submit_transaction};

const ONE_ORE: f64 = 100_000_000.0;

#[derive(Debug, Clone)]
struct RoundResult {
    round_id: u64,
    timestamp: String,
    our_squares: Vec<usize>,
    amount_deployed: u64,
    winning_squares: Option<Vec<usize>>,
    ore_won: u64,
    sol_won: u64,
    won: bool,
}

/// Continuous mining loop - uses WebSocket to track timing and deploy at optimal moment
pub async fn deploy_continuous(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<()> {
    let amount = std::env::var("AMOUNT")
        .map(|s| u64::from_str(&s).expect("Invalid AMOUNT"))
        .unwrap_or(15_000_000); // 0.015 SOL default

    // Initialize WebSocket manager for real-time updates
    let rpc_url = std::env::var("RPC").expect("Missing RPC env var");
    let ws_manager = WebSocketManager::new(&rpc_url);

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║          CONTINUOUS ORE MINING - WEBSOCKET DRIVEN              ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ Strategy: Late snipe (deploy at 10s remaining)                ║");
    println!("║ Amount: {:.4} SOL per round                                  ║", amount as f64 / 1_000_000_000.0);
    println!("║ Tracking: Real-time WebSocket (every 400ms)                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    println!("🔌 Starting WebSocket connections...");
    ws_manager.subscribe_to_board().await?;
    ws_manager.subscribe_to_slots().await?;

    println!("   Waiting for WebSocket initialization...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    let mut rounds_played = 0;
    let mut rounds_won = 0;
    let mut our_deployed_squares: Vec<usize> = Vec::new();
    let mut last_checkpoint_round: Option<u64> = None;

    // INFINITE LOOP - continuously mine every round
    loop {
        rounds_played += 1;

        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║  Round #{:<57}║", rounds_played);
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        // Checkpoint previous round and check results
        if rounds_played > 1 {
            println!("📝 Checkpointing previous round...");

            // Get miner BEFORE checkpoint to see deployed squares
            let miner_before = match get_miner(rpc, payer.pubkey()).await {
                Ok(m) => m,
                Err(e) => {
                    println!("⚠️  Could not get miner data: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            };

            let checkpoint_round = miner_before.round_id;

            // CRITICAL: Check if miner has already checkpointed
            if miner_before.checkpoint_id == checkpoint_round {
                println!("✅ Already checkpointed round #{}, skipping...", checkpoint_round);
                rounds_played -= 1; // Don't count this as a played round
                continue;
            }
            last_checkpoint_round = Some(checkpoint_round);

            // Submit checkpoint
            let checkpoint_ix = ore_api::sdk::checkpoint(payer.pubkey(), payer.pubkey(), checkpoint_round);
            match submit_transaction(rpc, payer, &[checkpoint_ix]).await {
                Ok(_) => println!("✅ Checkpointed round #{}", checkpoint_round),
                Err(e) => {
                    println!("⚠️  Checkpoint failed: {}", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    continue;
                }
            }

            // Wait for checkpoint to be confirmed (3 seconds to be safe)
            println!("⏳ Waiting for checkpoint confirmation...");
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            // Verify checkpoint was actually confirmed
            match get_miner(rpc, payer.pubkey()).await {
                Ok(miner) => {
                    if miner.checkpoint_id != checkpoint_round {
                        println!("⚠️  Checkpoint not confirmed yet (checkpoint_id: {}, expected: {})", miner.checkpoint_id, checkpoint_round);
                        println!("   Waiting extra 2 seconds...");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    } else {
                        println!("✅ Checkpoint confirmed (checkpoint_id: {})", miner.checkpoint_id);
                    }
                }
                Err(e) => println!("⚠️  Could not verify checkpoint: {}", e),
            }

            // Wait for round reset to complete (so slot_hash is available)
            println!("⏳ Waiting for round reset to complete...");
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

            // Get the round data to see the winning square
            let winning_square = match get_round(rpc, checkpoint_round).await {
                Ok(round) => {
                    // Calculate winning square from slot_hash
                    if let Some(rng) = round.rng() {
                        let winner = round.winning_square(rng);
                        println!("✅ Round #{} reset complete, slot_hash available", checkpoint_round);
                        Some(winner)
                    } else {
                        println!("⚠️  Round #{} slot_hash not ready yet (still zeros)", checkpoint_round);
                        None
                    }
                }
                Err(e) => {
                    println!("⚠️  Could not fetch round #{} data: {}", checkpoint_round, e);
                    None
                }
            };

            // Get miner AFTER checkpoint to see if rewards increased
            match get_miner(rpc, payer.pubkey()).await {
                Ok(miner_after) => {
                    let ore_earned = miner_after.rewards_ore.saturating_sub(miner_before.rewards_ore);
                    let sol_earned = miner_after.rewards_sol.saturating_sub(miner_before.rewards_sol);
                    let won = ore_earned > 0 || sol_earned > 0;

                    println!("\n╔════════════════════════════════════════════════════════════════╗");
                    if won {
                        println!("║  🎉 WON ROUND #{:<49}║", checkpoint_round);
                        println!("╠════════════════════════════════════════════════════════════════╣");
                        if ore_earned > 0 {
                            println!("║  💰 ORE Won: {:<48.8} ORE ║", ore_earned as f64 / 100_000_000.0);
                        }
                        if sol_earned > 0 {
                            println!("║  💰 SOL Won: {:<48.8} SOL ║", sol_earned as f64 / 1_000_000_000.0);
                        }
                        rounds_won += 1;
                    } else {
                        println!("║  ❌ LOST ROUND #{:<47}║", checkpoint_round);
                        println!("╠════════════════════════════════════════════════════════════════╣");
                        println!("║  No rewards this round                                         ║");
                    }

                    if let Some(winner) = winning_square {
                        println!("║  🎲 Winning Square: #{:<44}║", winner);
                        println!("║  📍 Our Squares: #{} and #{:<37}║", our_deployed_squares.get(0).unwrap_or(&0), our_deployed_squares.get(1).unwrap_or(&0));
                        if our_deployed_squares.contains(&winner) {
                            println!("║  ✅ We picked the winning square!                             ║");
                        } else {
                            println!("║  ❌ We missed the winning square                              ║");
                        }
                    } else {
                        println!("║  ⚠️  Winning square: Not available yet                        ║");
                    }

                    println!("╠════════════════════════════════════════════════════════════════╣");
                    println!("║  📊 Total Accumulated ORE: {:<32.8} ORE ║", miner_after.rewards_ore as f64 / 100_000_000.0);
                    println!("║  📈 Session Win Rate: {}/{} ({:.1}%)                              ║", rounds_won, rounds_played - 1, (rounds_won as f64 / (rounds_played - 1) as f64) * 100.0);
                    println!("╚════════════════════════════════════════════════════════════════╝\n");

                    // LOG TO FILE
                    let result = RoundResult {
                        round_id: checkpoint_round,
                        timestamp: Utc::now().to_rfc3339(),
                        our_squares: our_deployed_squares.clone(),
                        amount_deployed: amount,
                        winning_squares: winning_square.map(|w| vec![w]),
                        ore_won: ore_earned,
                        sol_won: sol_earned,
                        won,
                    };

                    if let Err(e) = log_round_result(&result, rounds_won, rounds_played - 1) {
                        println!("⚠️  Failed to log result: {}", e);
                    }
                }
                Err(e) => println!("⚠️  Could not check results: {}", e),
            }

            println!();
        }

        // Get current board state
        let mut board = get_board(rpc).await?;

        // CRITICAL: If we just checkpointed, wait for NEXT round to start
        if let Some(last_id) = last_checkpoint_round {
            if board.round_id <= last_id {
                println!("⏳ Waiting for round to advance from #{} to #{}...", last_id, last_id + 1);

                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    if let Some(ws_board) = ws_manager.get_board().await {
                        if ws_board.round_id > last_id {
                            board = ws_board;
                            println!("✅ Round advanced to #{}!", board.round_id);
                            break;
                        }
                    }
                }
            }
        }

        // WAIT FOR ROUND TO START if in intermission
        if board.end_slot == u64::MAX {
            println!("⏳ Round #{} in intermission, waiting for next round...", board.round_id);

            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                if let Some(ws_board) = ws_manager.get_board().await {
                    if ws_board.end_slot != u64::MAX {
                        board = ws_board;
                        println!("✅ Round #{} started!", board.round_id);
                        break;
                    }
                }
            }
        }

        // Now we have an active round - wait for the snipe window using WebSocket timing
        println!("⏰ Round #{} active - monitoring countdown via WebSocket...", board.round_id);
        println!("   Waiting for 10 second snipe window...\n");

        // WAIT UNTIL 10 SECONDS REMAINING (using WebSocket slot updates)
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if let Some(seconds_remaining) = ws_manager.get_seconds_remaining().await {
                // Deploy when we hit the 10 second window
                if seconds_remaining <= 10.0 && seconds_remaining > 0.0 {
                    println!("🎯 SNIPE WINDOW! {:.1}s remaining - deploying now...\n", seconds_remaining);
                    break;
                }

                // Check if round already ended (missed window)
                if seconds_remaining <= 0.0 {
                    println!("❌ Round ended before we could deploy, waiting for next round...");

                    // Wait for next round to start
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        if let Some(ws_board) = ws_manager.get_board().await {
                            if ws_board.round_id > board.round_id {
                                println!("🔄 New round detected, restarting loop...\n");
                                break; // Break from this loop to restart outer loop
                            }
                        }
                    }

                    // Now continue the outer loop to process the new round
                    continue;
                }
            }
        }

        // FETCH LATEST BOARD DATA at snipe time (CRITICAL: must be fresh!)
        board = get_board(rpc).await?;

        println!("📋 Board round_id: {}, end_slot: {}", board.round_id, board.end_slot);

        // Verify round is still active
        if board.end_slot == u64::MAX {
            println!("❌ Round in intermission, waiting for next round...\n");
            continue;
        }

        // Try to fetch round data
        let round = match get_round(rpc, board.round_id).await {
            Ok(r) => r,
            Err(e) => {
                println!("❌ Failed to fetch round {} data: {}", board.round_id, e);
                println!("   This usually means the round account doesn't exist yet.");
                println!("   Waiting for next round...\n");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                continue;
            }
        };
        let _treasury = get_treasury(rpc).await?;

        // Find 2 LEAST crowded squares
        let mut squares_by_deployment: Vec<(usize, u64, u64)> = round
            .deployed
            .iter()
            .zip(round.count.iter())
            .enumerate()
            .map(|(i, (&d, &c))| (i, d, c))
            .collect();

        squares_by_deployment.sort_by_key(|&(_, d, _)| d);

        let square_1 = squares_by_deployment[0];
        let square_2 = squares_by_deployment[1];

        // Save our deployed squares for logging
        our_deployed_squares = vec![square_1.0, square_2.0];

        println!("╔════════════════════════════════════════════════════════════════╗");
        println!("║                    DEPLOYMENT TARGETS                          ║");
        println!("╠════════════════════════════════════════════════════════════════╣");
        println!("║ 🎯 Square #{:<2}  {:.4} SOL  ({} miners)                    ║",
            square_1.0,
            square_1.1 as f64 / 1_000_000_000.0,
            square_1.2
        );
        println!("║ 🎯 Square #{:<2}  {:.4} SOL  ({} miners)                    ║",
            square_2.0,
            square_2.1 as f64 / 1_000_000_000.0,
            square_2.2
        );
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        // Deploy to 2 LEAST crowded squares
        let mut squares = [false; 25];
        squares[square_1.0] = true;
        squares[square_2.0] = true;

        let ix = ore_api::sdk::deploy(
            payer.pubkey(),
            payer.pubkey(),
            amount,
            board.round_id,
            squares,
        );

        // FINAL VERIFICATION - Check board one more time before submit
        let final_board = get_board(rpc).await?;

        if final_board.round_id != board.round_id {
            println!("❌ Round changed during preparation (was {}, now {}), skipping...\n", board.round_id, final_board.round_id);
            continue;
        }

        if final_board.end_slot == u64::MAX {
            println!("❌ Round ended during preparation, skipping to next round...\n");
            continue;
        }

        // Check timing one more time before submit
        if let Some(seconds_remaining) = ws_manager.get_seconds_remaining().await {
            if seconds_remaining <= 0.0 {
                println!("❌ Round ended during preparation, skipping to next round...\n");
                continue;
            }

            println!("📤 Submitting transaction ({:.1}s remaining)...", seconds_remaining);
        }

        match submit_transaction(rpc, payer, &[ix]).await {
            Ok(_) => {
                println!("✅ SNIPED! Deployed to squares #{} and #{}!\n", square_1.0, square_2.0);
            }
            Err(e) => {
                println!("❌ Deployment failed: {}\n", e);
                continue;
            }
        }

        // Wait for round to end (WebSocket will notify us)
        println!("⏰ Waiting for round to end (WebSocket monitoring)...");
        let current_round_id = board.round_id;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

            // Check if round ended and new round started
            if let Some(ws_board) = ws_manager.get_board().await {
                if ws_board.round_id > current_round_id {
                    println!("🏁 Round #{} ended, starting Round #{}...\n", current_round_id, ws_board.round_id);
                    break;
                }
            }

            // Also check slot-based timing
            if let Some(seconds_remaining) = ws_manager.get_seconds_remaining().await {
                if seconds_remaining <= 0.0 {
                    // Round should be over, wait a bit for reset
                    println!("⏱️  Round time expired, waiting for reset...");
                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    break;
                }
            }
        }

        // Loop continues automatically to next round!
    }
}

/// Log round result to JSON file
fn log_round_result(result: &RoundResult, total_wins: usize, total_rounds: usize) -> Result<()> {
    let log_file = "ore_mining_results.jsonl";

    let win_rate = if total_rounds > 0 {
        (total_wins as f64 / total_rounds as f64) * 100.0
    } else {
        0.0
    };

    let log_entry = json!({
        "round_id": result.round_id,
        "timestamp": result.timestamp,
        "our_squares": result.our_squares,
        "amount_deployed_lamports": result.amount_deployed,
        "amount_deployed_sol": result.amount_deployed as f64 / 1_000_000_000.0,
        "winning_squares": result.winning_squares,
        "ore_won_lamports": result.ore_won,
        "ore_won": result.ore_won as f64 / 100_000_000.0,
        "sol_won_lamports": result.sol_won,
        "sol_won": result.sol_won as f64 / 1_000_000_000.0,
        "won": result.won,
        "session_stats": {
            "total_wins": total_wins,
            "total_rounds": total_rounds,
            "win_rate_percent": win_rate,
        }
    });

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    writeln!(file, "{}", log_entry)?;

    Ok(())
}
