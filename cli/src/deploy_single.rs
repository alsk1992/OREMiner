use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::signature::Signer;
use std::str::FromStr;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use serde_json::json;
use chrono::Utc;

use crate::websocket::WebSocketManager;
use crate::{get_board, get_round, get_treasury, get_miner, submit_transaction};

/// Deploy to a SINGLE round, wait for it to end, checkpoint, and show results
pub async fn deploy_single(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<()> {
    let amount = std::env::var("AMOUNT")
        .map(|s| u64::from_str(&s).expect("Invalid AMOUNT"))
        .unwrap_or(5_000_000); // 0.005 SOL per square (0.01 total)

    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║              SINGLE ROUND ORE MINING TEST                      ║");
    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║ Amount: {:.4} SOL per square ({:.4} SOL total)               ║",
        amount as f64 / 1_000_000_000.0,
        (amount * 2) as f64 / 1_000_000_000.0
    );
    println!("║ Strategy: Deploy to 2 least crowded squares                   ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Initialize WebSocket for real-time monitoring
    let rpc_url = std::env::var("RPC").expect("Missing RPC env var");
    let ws_manager = WebSocketManager::new(&rpc_url);

    println!("🔌 Starting WebSocket connections...");
    ws_manager.subscribe_to_board().await?;
    ws_manager.subscribe_to_slots().await?;
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // FIRST: Check if we need to checkpoint any old rounds
    println!("🔍 Checking miner status...");
    let miner = get_miner(rpc, payer.pubkey()).await?;

    if miner.checkpoint_id != miner.round_id && miner.round_id > 0 {
        println!("⚠️  Found un-checkpointed round #{}", miner.round_id);
        println!("📝 Auto-checkpointing round #{} before starting...\n", miner.round_id);

        let checkpoint_ix = ore_api::sdk::checkpoint(payer.pubkey(), payer.pubkey(), miner.round_id);
        match submit_transaction(rpc, payer, &[checkpoint_ix]).await {
            Ok(_) => {
                println!("✅ Auto-checkpoint completed");
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            }
            Err(e) => {
                println!("⚠️  Auto-checkpoint failed: {}", e);
                println!("   Continuing anyway...");
            }
        }
        println!();
    } else {
        println!("✅ Miner is up to date (round #{}, checkpoint #{})\n", miner.round_id, miner.checkpoint_id);
    }

    // Get current board
    let mut board = get_board(rpc).await?;

    println!("📋 Current board: Round #{}", board.round_id);

    // If in intermission, wait for round to start
    if board.end_slot == u64::MAX {
        println!("⏳ Round in intermission, waiting for next round to start...\n");

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            if let Some(ws_board) = ws_manager.get_board().await {
                if ws_board.end_slot != u64::MAX {
                    board = ws_board;
                    println!("✅ Round #{} started!\n", board.round_id);
                    break;
                }
            }
        }
    }

    let deploy_round_id = board.round_id;
    println!("🎯 Will deploy to Round #{}\n", deploy_round_id);

    // Wait for 10 second window
    println!("⏰ Monitoring countdown via WebSocket...");
    println!("   Waiting for 10 second deployment window...\n");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        if let Some(seconds_remaining) = ws_manager.get_seconds_remaining().await {
            // Deploy when we hit the 10 second window
            if seconds_remaining <= 10.0 && seconds_remaining > 0.0 {
                println!("🎯 DEPLOYMENT WINDOW! {:.1}s remaining\n", seconds_remaining);
                break;
            }

            // Safety check - don't miss the round
            if seconds_remaining <= 0.0 {
                println!("❌ Round ended before deployment window - exiting\n");
                return Ok(());
            }
        }
    }

    // Fetch fresh data
    board = get_board(rpc).await?;

    if board.round_id != deploy_round_id {
        println!("❌ Round changed during wait - exiting\n");
        return Ok(());
    }

    let round = match get_round(rpc, board.round_id).await {
        Ok(r) => r,
        Err(e) => {
            println!("❌ Failed to fetch round data: {}\n", e);
            return Ok(());
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

    let our_squares = vec![square_1.0, square_2.0];

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

    // Build deployment instruction
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

    // Deploy
    println!("📤 Submitting deployment transaction...");
    match submit_transaction(rpc, payer, &[ix]).await {
        Ok(_) => {
            println!("✅ Successfully deployed to squares #{} and #{}!\n", square_1.0, square_2.0);
        }
        Err(e) => {
            println!("❌ Deployment failed: {}\n", e);
            return Ok(());
        }
    }

    // Wait for round to end
    println!("⏰ Waiting for round to end...\n");

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        if let Some(seconds_remaining) = ws_manager.get_seconds_remaining().await {
            if seconds_remaining <= 0.0 {
                println!("🏁 Round #{} ended!\n", deploy_round_id);
                break;
            }
        }
    }

    // Wait a bit for round to be reset
    println!("⏳ Waiting for round reset to complete...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Get miner BEFORE checkpoint
    let miner_before = get_miner(rpc, payer.pubkey()).await?;
    println!("📊 Miner state before checkpoint:");
    println!("   Round ID: {}", miner_before.round_id);
    println!("   Checkpoint ID: {}", miner_before.checkpoint_id);
    println!("   Rewards SOL: {:.8} SOL", miner_before.rewards_sol as f64 / 1_000_000_000.0);
    println!("   Rewards ORE: {:.8} ORE\n", miner_before.rewards_ore as f64 / 100_000_000.0);

    // Check if already checkpointed
    if miner_before.checkpoint_id == miner_before.round_id {
        println!("⚠️  Already checkpointed round #{} - someone else checkpointed for us\n", miner_before.round_id);
        return Ok(());
    }

    // Submit checkpoint
    println!("📝 Submitting checkpoint for round #{}...", deploy_round_id);
    let checkpoint_ix = ore_api::sdk::checkpoint(payer.pubkey(), payer.pubkey(), deploy_round_id);

    match submit_transaction(rpc, payer, &[checkpoint_ix]).await {
        Ok(_) => println!("✅ Checkpoint transaction submitted\n"),
        Err(e) => {
            println!("❌ Checkpoint failed: {}\n", e);
            return Ok(());
        }
    }

    // Wait for checkpoint to confirm
    println!("⏳ Waiting for checkpoint confirmation...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Verify checkpoint confirmed
    let miner_check = get_miner(rpc, payer.pubkey()).await?;
    if miner_check.checkpoint_id != deploy_round_id {
        println!("⚠️  Checkpoint not confirmed yet, waiting extra time...");
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    }

    // Get round data to calculate winning square
    println!("🔍 Fetching round #{} data to determine winner...", deploy_round_id);
    let winning_square = match get_round(rpc, deploy_round_id).await {
        Ok(round) => {
            if let Some(rng) = round.rng() {
                let winner = round.winning_square(rng);
                println!("✅ Winning square calculated: #{}\n", winner);
                Some(winner)
            } else {
                println!("⚠️  Round slot_hash not available yet\n");
                None
            }
        }
        Err(e) => {
            println!("⚠️  Could not fetch round data: {}\n", e);
            None
        }
    };

    // Get miner AFTER checkpoint
    let miner_after = get_miner(rpc, payer.pubkey()).await?;

    let ore_earned = miner_after.rewards_ore.saturating_sub(miner_before.rewards_ore);
    let sol_earned = miner_after.rewards_sol.saturating_sub(miner_before.rewards_sol);
    let won = ore_earned > 0 || sol_earned > 0;

    // Display results
    println!("╔════════════════════════════════════════════════════════════════╗");
    if won {
        println!("║            🎉🎉🎉 WE WON ROUND #{} 🎉🎉🎉                    ║", deploy_round_id);
    } else {
        println!("║                ❌ LOST ROUND #{}                              ║", deploy_round_id);
    }
    println!("╠════════════════════════════════════════════════════════════════╣");

    if let Some(winner) = winning_square {
        println!("║  🎲 Winning Square: #{:<44}║", winner);
        println!("║  📍 Our Squares: #{} and #{:<37}║", our_squares[0], our_squares[1]);

        if our_squares.contains(&winner) {
            println!("║  ✅ We picked the winning square!                             ║");
        } else {
            println!("║  ❌ We missed the winning square                              ║");
        }
    } else {
        println!("║  ⚠️  Winning square: Data not available                       ║");
    }

    println!("╠════════════════════════════════════════════════════════════════╣");

    if won {
        if ore_earned > 0 {
            println!("║  💰 ORE Won This Round: {:<32.8} ORE ║", ore_earned as f64 / 100_000_000.0);
        }
        if sol_earned > 0 {
            println!("║  💰 SOL Won This Round: {:<32.8} SOL ║", sol_earned as f64 / 1_000_000_000.0);
        }
    } else {
        println!("║  No rewards this round                                         ║");
    }

    println!("╠════════════════════════════════════════════════════════════════╣");
    println!("║  📊 Total Accumulated ORE: {:<32.8} ORE ║", miner_after.rewards_ore as f64 / 100_000_000.0);
    println!("║  📊 Total Accumulated SOL: {:<32.8} SOL ║", miner_after.rewards_sol as f64 / 1_000_000_000.0);
    println!("║  📈 Lifetime Rewards ORE: {:<33.8} ORE ║", miner_after.lifetime_rewards_ore as f64 / 100_000_000.0);
    println!("║  📈 Lifetime Rewards SOL: {:<33.8} SOL ║", miner_after.lifetime_rewards_sol as f64 / 1_000_000_000.0);
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    // Log to file
    let log_entry = json!({
        "round_id": deploy_round_id,
        "timestamp": Utc::now().to_rfc3339(),
        "our_squares": our_squares,
        "amount_per_square_lamports": amount,
        "amount_per_square_sol": amount as f64 / 1_000_000_000.0,
        "total_deployed_lamports": amount * 2,
        "total_deployed_sol": (amount * 2) as f64 / 1_000_000_000.0,
        "winning_square": winning_square,
        "ore_won_this_round_lamports": ore_earned,
        "ore_won_this_round": ore_earned as f64 / 100_000_000.0,
        "sol_won_this_round_lamports": sol_earned,
        "sol_won_this_round": sol_earned as f64 / 1_000_000_000.0,
        "won": won,
        "total_accumulated_ore": miner_after.rewards_ore as f64 / 100_000_000.0,
        "total_accumulated_sol": miner_after.rewards_sol as f64 / 1_000_000_000.0,
        "lifetime_ore": miner_after.lifetime_rewards_ore as f64 / 100_000_000.0,
        "lifetime_sol": miner_after.lifetime_rewards_sol as f64 / 1_000_000_000.0,
    });

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ore_mining_single_results.jsonl")?;

    writeln!(file, "{}", log_entry)?;
    println!("📁 Results logged to ore_mining_single_results.jsonl");

    Ok(())
}
