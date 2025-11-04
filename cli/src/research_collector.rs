/// Research Data Collector - Track 100 rounds to discover winning patterns
///
/// This module collects comprehensive round data to analyze:
/// - Do recent winners get low pools next round?
/// - Is there clustering in winning squares?
/// - Do certain squares win more often?
/// - What's the optimal deployment timing?

use ore_api::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResearchData {
    /// Round identifier
    pub round_id: u64,

    /// Timestamp when we observed this round
    pub timestamp: String,

    /// All 25 square pool sizes (in lamports) at observation time
    pub square_pools: [u64; 25],

    /// Total deployed across all squares
    pub total_deployed: u64,

    /// Motherlode pool size
    pub motherlode: u64,

    /// Which square(s) won (filled in after round ends)
    pub winning_squares: Option<Vec<usize>>,

    /// The 5 least crowded squares at observation time
    pub least_crowded_5: Vec<(usize, u64)>,

    /// The 5 most crowded squares at observation time
    pub most_crowded_5: Vec<(usize, u64)>,

    /// Average deployment per square
    pub avg_deployment: u64,

    /// Max deployment on any square
    pub max_deployment: u64,

    /// Min deployment on any square
    pub min_deployment: u64,

    /// How many squares have active deployment
    pub active_squares: usize,

    /// Variance in deployment distribution
    pub deployment_variance: f64,

    /// Did the winner come from the least crowded 5?
    pub winner_was_least_crowded: Option<bool>,

    /// Did the winner come from the most crowded 5?
    pub winner_was_most_crowded: Option<bool>,

    /// Previous round's winning square (if known)
    pub previous_winner: Option<usize>,

    /// Did previous winner have low pool this round?
    pub previous_winner_pool_rank: Option<usize>,
}

impl RoundResearchData {
    pub fn from_round(round: &Round, previous_winner: Option<usize>) -> Self {
        let mut sorted_by_deployment: Vec<(usize, u64)> = round
            .deployed
            .iter()
            .enumerate()
            .map(|(i, &d)| (i, d))
            .collect();

        sorted_by_deployment.sort_by_key(|&(_, d)| d);

        let least_crowded_5: Vec<(usize, u64)> = sorted_by_deployment
            .iter()
            .take(5)
            .copied()
            .collect();

        let most_crowded_5: Vec<(usize, u64)> = sorted_by_deployment
            .iter()
            .rev()
            .take(5)
            .copied()
            .collect();

        let active_squares = round.deployed.iter().filter(|&&d| d > 0).count();
        let total_deployed = round.total_deployed;
        let avg_deployment = if active_squares > 0 {
            total_deployed / active_squares as u64
        } else {
            0
        };

        let max_deployment = *round.deployed.iter().max().unwrap_or(&0);
        let min_deployment = *round.deployed.iter().min().unwrap_or(&0);

        // Calculate variance
        let variance = if avg_deployment > 0 {
            let sum_squared_diff: f64 = round
                .deployed
                .iter()
                .map(|&d| {
                    let diff = d as f64 - avg_deployment as f64;
                    diff * diff
                })
                .sum();
            sum_squared_diff / 25.0
        } else {
            0.0
        };

        // Find rank of previous winner in current round
        let previous_winner_pool_rank = previous_winner.and_then(|prev_idx| {
            sorted_by_deployment
                .iter()
                .position(|&(idx, _)| idx == prev_idx)
        });

        Self {
            round_id: round.id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            square_pools: round.deployed,
            total_deployed,
            motherlode: round.motherlode,
            winning_squares: None,
            least_crowded_5,
            most_crowded_5,
            avg_deployment,
            max_deployment,
            min_deployment,
            active_squares,
            deployment_variance: variance,
            winner_was_least_crowded: None,
            winner_was_most_crowded: None,
            previous_winner,
            previous_winner_pool_rank,
        }
    }

    pub fn update_winner(&mut self, winning_square: usize) {
        self.winning_squares = Some(vec![winning_square]);

        // Check if winner was in least crowded 5
        let least_crowded_indices: Vec<usize> =
            self.least_crowded_5.iter().map(|&(idx, _)| idx).collect();
        self.winner_was_least_crowded = Some(least_crowded_indices.contains(&winning_square));

        // Check if winner was in most crowded 5
        let most_crowded_indices: Vec<usize> =
            self.most_crowded_5.iter().map(|&(idx, _)| idx).collect();
        self.winner_was_most_crowded = Some(most_crowded_indices.contains(&winning_square));
    }
}

/// Research session tracker
pub struct ResearchSession {
    pub target_rounds: usize,
    pub rounds_collected: Vec<RoundResearchData>,
    pub last_winner: Option<usize>,
    pub output_file: String,
}

impl ResearchSession {
    pub fn new(target_rounds: usize) -> Self {
        Self {
            target_rounds,
            rounds_collected: Vec::new(),
            last_winner: None,
            output_file: "ore_research_data.jsonl".to_string(),
        }
    }

    pub fn add_round(&mut self, round: &Round) {
        let data = RoundResearchData::from_round(round, self.last_winner);
        self.rounds_collected.push(data);
        self.save_to_file();
    }

    pub fn update_winner(&mut self, round_id: u64, winning_square: usize) {
        // Find the round data and update it
        if let Some(round_data) = self.rounds_collected.iter_mut().find(|r| r.round_id == round_id) {
            round_data.update_winner(winning_square);
            self.last_winner = Some(winning_square);
            self.save_to_file();
        }
    }

    pub fn is_complete(&self) -> bool {
        self.rounds_collected.len() >= self.target_rounds
    }

    pub fn progress(&self) -> f64 {
        (self.rounds_collected.len() as f64 / self.target_rounds as f64) * 100.0
    }

    fn save_to_file(&self) {
        // Append latest round to JSONL file
        if let Some(latest) = self.rounds_collected.last() {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.output_file)
                .expect("Failed to open research data file");

            let json = serde_json::to_string(latest).expect("Failed to serialize");
            writeln!(file, "{}", json).expect("Failed to write to file");
        }
    }

    /// Analyze collected data and generate insights
    pub fn analyze(&self) -> ResearchInsights {
        let mut insights = ResearchInsights::default();

        if self.rounds_collected.is_empty() {
            return insights;
        }

        // Count how many times each square won
        let mut square_win_counts: HashMap<usize, usize> = HashMap::new();
        let mut least_crowded_wins = 0;
        let mut most_crowded_wins = 0;
        let mut previous_winner_ranks: Vec<usize> = Vec::new();

        for round in &self.rounds_collected {
            if let Some(winners) = &round.winning_squares {
                for &winner in winners {
                    *square_win_counts.entry(winner).insert(0) += 1;
                }

                if let Some(true) = round.winner_was_least_crowded {
                    least_crowded_wins += 1;
                }

                if let Some(true) = round.winner_was_most_crowded {
                    most_crowded_wins += 1;
                }
            }

            if let Some(rank) = round.previous_winner_pool_rank {
                previous_winner_ranks.push(rank);
            }
        }

        insights.total_rounds = self.rounds_collected.len();
        insights.completed_rounds = self.rounds_collected.iter()
            .filter(|r| r.winning_squares.is_some())
            .count();

        insights.least_crowded_win_rate = if insights.completed_rounds > 0 {
            (least_crowded_wins as f64 / insights.completed_rounds as f64) * 100.0
        } else {
            0.0
        };

        insights.most_crowded_win_rate = if insights.completed_rounds > 0 {
            (most_crowded_wins as f64 / insights.completed_rounds as f64) * 100.0
        } else {
            0.0
        };

        // Average rank of previous winner in next round (0 = least crowded)
        insights.avg_previous_winner_rank = if !previous_winner_ranks.is_empty() {
            previous_winner_ranks.iter().sum::<usize>() as f64 / previous_winner_ranks.len() as f64
        } else {
            0.0
        };

        // Find hot/cold squares
        insights.square_win_counts = square_win_counts;

        insights
    }

    pub fn print_progress(&self) {
        println!("\n╔════════════════════════════════════════════╗");
        println!("║      RESEARCH DATA COLLECTION              ║");
        println!("╠════════════════════════════════════════════╣");
        println!("║ Progress: {}/{} rounds ({:.1}%)          ║",
            self.rounds_collected.len(),
            self.target_rounds,
            self.progress()
        );
        println!("║ Last winner: {:?}                          ║", self.last_winner);
        println!("╚════════════════════════════════════════════╝");
    }
}

#[derive(Debug, Default)]
pub struct ResearchInsights {
    pub total_rounds: usize,
    pub completed_rounds: usize,
    pub least_crowded_win_rate: f64,
    pub most_crowded_win_rate: f64,
    pub avg_previous_winner_rank: f64,
    pub square_win_counts: HashMap<usize, usize>,
}

impl ResearchInsights {
    pub fn print(&self) {
        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║           RESEARCH INSIGHTS - 100 ROUNDS               ║");
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║ Total rounds observed: {}                             ║", self.total_rounds);
        println!("║ Completed rounds: {}                                  ║", self.completed_rounds);
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║ LEAST CROWDED STRATEGY:                                ║");
        println!("║   Win rate: {:.1}%                                     ║", self.least_crowded_win_rate);
        println!("║   Expected: 20% (5/25 squares)                         ║");
        println!("║   Verdict: {}                                          ║",
            if self.least_crowded_win_rate > 20.0 { "✓ PROFITABLE" } else { "✗ NOT PROFITABLE" }
        );
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║ MOST CROWDED STRATEGY:                                 ║");
        println!("║   Win rate: {:.1}%                                     ║", self.most_crowded_win_rate);
        println!("║   Expected: 20% (5/25 squares)                         ║");
        println!("║   Verdict: {}                                          ║",
            if self.most_crowded_win_rate > 20.0 { "✓ PROFITABLE" } else { "✗ NOT PROFITABLE" }
        );
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║ PREVIOUS WINNER ANALYSIS:                              ║");
        println!("║   Avg rank in next round: {:.1}/25                     ║", self.avg_previous_winner_rank);
        println!("║   (0 = least crowded, 24 = most crowded)               ║");
        println!("║   Conclusion: {}                                       ║",
            if self.avg_previous_winner_rank < 5.0 {
                "Previous winners DO get low pools!"
            } else if self.avg_previous_winner_rank > 20.0 {
                "Previous winners get HIGH pools!"
            } else {
                "No clear pattern"
            }
        );
        println!("╠════════════════════════════════════════════════════════╣");
        println!("║ HOT SQUARES (won most often):                          ║");

        let mut sorted_squares: Vec<_> = self.square_win_counts.iter().collect();
        sorted_squares.sort_by(|a, b| b.1.cmp(a.1));

        for (square, count) in sorted_squares.iter().take(5) {
            println!("║   Square {}: {} wins                                  ║", square, count);
        }

        println!("╚════════════════════════════════════════════════════════╝");
    }
}
