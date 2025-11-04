use anyhow::Result;
use futures_util::StreamExt;
use ore_api::state::{Board, Round};
use solana_account_decoder::UiAccountEncoding;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::RpcAccountInfoConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use std::sync::Arc;
use tokio::sync::RwLock;
use steel::AccountDeserialize;

/// WebSocket manager for real-time account updates
pub struct WebSocketManager {
    ws_url: String,
    board_data: Arc<RwLock<Option<Board>>>,
    current_round_data: Arc<RwLock<Option<Round>>>,
    current_slot: Arc<RwLock<u64>>,
}

impl WebSocketManager {
    pub fn new(rpc_url: &str) -> Self {
        // Convert HTTP(S) RPC URL to WebSocket URL
        let ws_url = rpc_url
            .replace("https://", "wss://")
            .replace("http://", "ws://");

        Self {
            ws_url,
            board_data: Arc::new(RwLock::new(None)),
            current_round_data: Arc::new(RwLock::new(None)),
            current_slot: Arc::new(RwLock::new(0)),
        }
    }

    /// Subscribe to slot updates for precise timing (updates every 400ms)
    pub async fn subscribe_to_slots(&self) -> Result<()> {
        let ws_url = self.ws_url.clone();
        let current_slot = self.current_slot.clone();
        let board_data = self.board_data.clone();

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::slot_subscription_loop(&ws_url, current_slot.clone(), board_data.clone()).await {
                    eprintln!("Slot WebSocket error: {}, reconnecting in 2s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }
        });

        Ok(())
    }

    async fn slot_subscription_loop(
        ws_url: &str,
        current_slot: Arc<RwLock<u64>>,
        board_data: Arc<RwLock<Option<Board>>>,
    ) -> Result<()> {
        let pubsub = PubsubClient::new(ws_url).await?;
        let (mut stream, _unsub) = pubsub.slot_subscribe().await?;

        println!("🔌 Slot monitoring active (updates every ~400ms)");

        while let Some(slot_info) = stream.next().await {
            let slot = slot_info.slot;

            // Update current slot
            {
                let mut current = current_slot.write().await;
                *current = slot;
            }

            // Check if we have board data to calculate timing
            if let Some(board) = board_data.read().await.as_ref() {
                if board.end_slot != u64::MAX {
                    let slots_remaining = board.end_slot.saturating_sub(slot);
                    let seconds_remaining = (slots_remaining as f64) * 0.4;

                    // Log important timing milestones
                    if seconds_remaining <= 20.0 && seconds_remaining > 19.5 {
                        println!("⏰ 20 seconds remaining - entering snipe window");
                    } else if seconds_remaining <= 10.0 && seconds_remaining > 9.5 {
                        println!("🎯 10 seconds remaining - DEPLOY NOW!");
                    } else if seconds_remaining <= 1.0 && seconds_remaining > 0.5 {
                        println!("⚠️  1 second remaining - round ending soon");
                    } else if slot >= board.end_slot {
                        println!("🏁 Round #{} ended at slot {}", board.round_id, slot);
                    }
                }
            }
        }

        Ok(())
    }

    /// Subscribe to Board account updates - triggers on round start/end
    pub async fn subscribe_to_board(&self) -> Result<()> {
        let ws_url = self.ws_url.clone();
        let board_data = self.board_data.clone();
        let board_pda = ore_api::state::board_pda().0;

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::board_subscription_loop(&ws_url, &board_pda, board_data.clone()).await {
                    eprintln!("Board WebSocket error: {}, reconnecting in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });

        Ok(())
    }

    async fn board_subscription_loop(
        ws_url: &str,
        board_pda: &Pubkey,
        board_data: Arc<RwLock<Option<Board>>>,
    ) -> Result<()> {
        let pubsub = PubsubClient::new(ws_url).await?;
        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            data_slice: None,
            min_context_slot: None,
        };

        let (mut stream, _unsub) = pubsub.account_subscribe(board_pda, Some(config)).await?;

        while let Some(response) = stream.next().await {
            if let solana_account_decoder::UiAccountData::Binary(data, _encoding) = response.value.data {
                if let Ok(bytes) = base64::decode(&data) {
                    if let Ok(board) = Board::try_from_bytes(&bytes) {
                        let mut cache = board_data.write().await;
                        *cache = Some(*board);

                        // Log important state changes
                        if board.end_slot == u64::MAX {
                            println!("🔔 WebSocket: Round #{} in intermission", board.round_id);
                        } else {
                            println!("🔔 WebSocket: Round #{} active, end_slot: {}", board.round_id, board.end_slot);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Subscribe to specific Round account updates - triggers on deployments
    pub async fn subscribe_to_round(&self, round_id: u64) -> Result<()> {
        let ws_url = self.ws_url.clone();
        let round_data = self.current_round_data.clone();
        let round_pda = ore_api::state::round_pda(round_id).0;

        tokio::spawn(async move {
            loop {
                if let Err(e) = Self::round_subscription_loop(&ws_url, &round_pda, round_data.clone()).await {
                    eprintln!("Round WebSocket error: {}, reconnecting in 5s...", e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        });

        Ok(())
    }

    async fn round_subscription_loop(
        ws_url: &str,
        round_pda: &Pubkey,
        round_data: Arc<RwLock<Option<Round>>>,
    ) -> Result<()> {
        let pubsub = PubsubClient::new(ws_url).await?;
        let config = RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            commitment: Some(CommitmentConfig::confirmed()),
            data_slice: None,
            min_context_slot: None,
        };

        let (mut stream, _unsub) = pubsub.account_subscribe(round_pda, Some(config)).await?;

        while let Some(response) = stream.next().await {
            if let solana_account_decoder::UiAccountData::Binary(data, _encoding) = response.value.data {
                if let Ok(bytes) = base64::decode(&data) {
                    if let Ok(round) = Round::try_from_bytes(&bytes) {
                        let mut cache = round_data.write().await;
                        *cache = Some(*round);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get latest board data from WebSocket cache
    pub async fn get_board(&self) -> Option<Board> {
        self.board_data.read().await.clone()
    }

    /// Get latest round data from WebSocket cache
    pub async fn get_round(&self) -> Option<Round> {
        self.current_round_data.read().await.clone()
    }

    /// Get current slot from WebSocket
    pub async fn get_current_slot(&self) -> u64 {
        *self.current_slot.read().await
    }

    /// Get seconds remaining in current round (using real-time slot data)
    pub async fn get_seconds_remaining(&self) -> Option<f64> {
        let board = self.board_data.read().await;
        let slot = *self.current_slot.read().await;

        if let Some(board) = board.as_ref() {
            if board.end_slot != u64::MAX && slot > 0 {
                let slots_remaining = board.end_slot.saturating_sub(slot);
                return Some((slots_remaining as f64) * 0.4);
            }
        }

        None
    }

    /// Wait for round to start (end_slot != u64::MAX)
    pub async fn wait_for_round_start(&self, timeout_secs: u64) -> Result<Board> {
        let start = std::time::Instant::now();

        loop {
            if let Some(board) = self.get_board().await {
                if board.end_slot != u64::MAX {
                    return Ok(board);
                }
            }

            if start.elapsed().as_secs() > timeout_secs {
                anyhow::bail!("Timeout waiting for round to start");
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Wait for round number to change (round reset)
    pub async fn wait_for_round_reset(&self, current_round_id: u64, timeout_secs: u64) -> Result<Board> {
        let start = std::time::Instant::now();

        loop {
            if let Some(board) = self.get_board().await {
                if board.round_id > current_round_id {
                    return Ok(board);
                }
            }

            if start.elapsed().as_secs() > timeout_secs {
                anyhow::bail!("Timeout waiting for round reset");
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    /// Wait for optimal deployment window (X-Y seconds remaining)
    pub async fn wait_for_deploy_window(&self, max_seconds: u64, min_seconds: u64) {
        loop {
            if let Some(seconds_remaining) = self.get_seconds_remaining().await {
                if seconds_remaining <= max_seconds as f64 && seconds_remaining > min_seconds as f64 {
                    println!("✅ Optimal window reached ({:.1}s remaining)", seconds_remaining);
                    return;
                }

                if seconds_remaining > max_seconds as f64 {
                    // Still too early
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                } else {
                    // Window passed, deploy now!
                    return;
                }
            } else {
                // No timing data, wait a bit
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        }
    }
}
