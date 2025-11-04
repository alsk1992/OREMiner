/// DEPLOY WITH OPTIMAL +EV STRATEGY
/// Uses research findings: filter previous winner, deploy to least crowded

use chrono::Utc;
use ore_api::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    native_token::LAMPORTS_PER_SOL,
    signature::Signer,
    signer::keypair::Keypair,
    transaction::Transaction,
};
use crate::optimal_strategy::*;

const DEPLOYMENT_AMOUNT: u64 = 40_000_000; // 0.04 SOL total
const NUM_SQUARES: usize = 2; // Deploy to 2 least crowded

pub async fn deploy_optimal_strategy(
    rpc: &RpcClient,
    payer: &Keypair,
) -> Result<(), anyhow::Error> {
    println!("\n╔════════════════════════════════════════════════════════╗");
    println!("║     🎯 OPTIMAL +EV MINING STRATEGY DEPLOYMENT          ║");
    println!("╠════════════════════════════════════════════════════════╣");
    println!("║  Based on 100 rounds of real data                     ║");
    println!("║  Edge: +28% better share in least crowded squares     ║");
    println!("╚════════════════════════════════════════════════════════╝\n");

    // Get current board and round
    let board = get_board(rpc).await?;
    let round = get_round(rpc, board.round_id).await?;

    println!("📊 Current Round: #{}", board.round_id);
    println!("💰 Total Deployed: {:.4} SOL\n", round.total_deployed as f64 / 1e9);

    // Get previous winner
    let previous_winner = get_previous_winner(rpc, board.round_id).await;

    // Select optimal squares (filters out previous winner)
    let selected_squares = select_optimal_squares(&round, previous_winner, NUM_SQUARES);

    if selected_squares.len() < NUM_SQUARES {
        println!("⚠️  Not enough squares available after filtering");
        return Ok(());
    }

    let deployment_per_square = DEPLOYMENT_AMOUNT / NUM_SQUARES as u64;

    // Display strategy
    print_strategy_info(&round, &selected_squares, previous_winner, deployment_per_square);

    // Build and send transaction
    println!("🚀 Building transaction...");

    let mut instructions = vec![];

    // Compute budget
    instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(500_000));
    instructions.push(ComputeBudgetInstruction::set_compute_unit_price(50_000));

    // Deploy to each selected square
    for &square_idx in &selected_squares {
        let ix = ore_api::sdk::deploy(
            payer.pubkey(),
            square_idx,
            deployment_per_square,
        );
        instructions.push(ix);
    }

    // Get recent blockhash and send
    let recent_blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );

    println!("📤 Sending transaction...");
    match rpc.send_and_confirm_transaction(&tx).await {
        Ok(sig) => {
            println!("✅ Deployment successful!");
            println!("🔗 Signature: {}", sig);

            // Log results
            log_deployment_result(
                board.round_id,
                &selected_squares,
                previous_winner,
                DEPLOYMENT_AMOUNT,
            );

            println!("\n💡 Now wait for round to end and check results!");
            println!("   Run: export COMMAND=checkpoint && cargo run --bin ore-cli\n");
        }
        Err(e) => {
            println!("❌ Deployment failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn get_board(rpc: &RpcClient) -> Result<Board, anyhow::Error> {
    let board_pubkey = board_pda().0;
    let data = rpc.get_account_data(&board_pubkey).await?;
    Ok(Board::try_from_bytes(&data)?)
}

async fn get_round(rpc: &RpcClient, round_id: u64) -> Result<Round, anyhow::Error> {
    let round_pubkey = round_pda(round_id).0;
    let data = rpc.get_account_data(&round_pubkey).await?;
    Ok(Round::try_from_bytes(&data)?)
}

fn log_deployment_result(
    round_id: u64,
    selected_squares: &[usize],
    previous_winner: Option<usize>,
    amount: u64,
) {
    use std::fs::OpenOptions;
    use std::io::Write;

    let result = serde_json::json!({
        "timestamp": Utc::now().to_rfc3339(),
        "round_id": round_id,
        "strategy": "optimal_ev",
        "selected_squares": selected_squares,
        "selected_squares_display": selected_squares.iter().map(|&s| s + 1).collect::<Vec<_>>(),
        "previous_winner": previous_winner,
        "previous_winner_display": previous_winner.map(|w| w + 1),
        "amount_deployed_lamports": amount,
        "amount_deployed_sol": amount as f64 / LAMPORTS_PER_SOL as f64,
    });

    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("optimal_strategy_results.jsonl")
    {
        let _ = writeln!(file, "{}", result);
    }
}
