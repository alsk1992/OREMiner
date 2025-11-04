use std::{
    io,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ore_api::prelude::*;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Tabs, Wrap,
    },
    Frame, Terminal,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    native_token::{lamports_to_sol, LAMPORTS_PER_SOL},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
use spl_token::amount_to_ui_amount;
use steel::Clock;

use crate::{get_board, get_clock, get_miner, get_round, get_stake, get_treasury, submit_transaction};

// Re-export needed constants
use ore_api::consts::TOKEN_DECIMALS;

const REFRESH_RATE: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tab {
    Dashboard,
    Board,
    Miner,
    Stake,
    Logs,
}

impl Tab {
    fn next(&self) -> Self {
        match self {
            Tab::Dashboard => Tab::Board,
            Tab::Board => Tab::Miner,
            Tab::Miner => Tab::Stake,
            Tab::Stake => Tab::Logs,
            Tab::Logs => Tab::Dashboard,
        }
    }

    fn prev(&self) -> Self {
        match self {
            Tab::Dashboard => Tab::Logs,
            Tab::Board => Tab::Dashboard,
            Tab::Miner => Tab::Board,
            Tab::Stake => Tab::Miner,
            Tab::Logs => Tab::Stake,
        }
    }
}

#[derive(Debug)]
struct AppState {
    tab: Tab,
    board: Option<Board>,
    clock: Option<Clock>,
    miner: Option<Miner>,
    stake: Option<Stake>,
    treasury: Option<Treasury>,
    round: Option<Round>,
    logs: Vec<LogEntry>,
    auto_mine: bool,
    auto_checkpoint: bool,
    deploy_amount: u64,
    selected_squares: [bool; 25],
    last_update: Instant,
    stats: MiningStats,
}

#[derive(Debug, Clone)]
struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    message: String,
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, Default)]
struct MiningStats {
    rounds_mined: u64,
    total_sol_earned: u64,
    total_ore_earned: u64,
    rounds_won: u64,
    uptime: Duration,
}

impl AppState {
    fn new() -> Self {
        Self {
            tab: Tab::Dashboard,
            board: None,
            clock: None,
            miner: None,
            stake: None,
            treasury: None,
            round: None,
            logs: Vec::new(),
            auto_mine: false,
            auto_checkpoint: true,
            deploy_amount: LAMPORTS_PER_SOL / 100, // Default: 0.01 SOL
            selected_squares: [true; 25], // Default: all squares
            last_update: Instant::now(),
            stats: MiningStats::default(),
        }
    }

    fn add_log(&mut self, level: LogLevel, message: String) {
        self.logs.push(LogEntry {
            timestamp: Utc::now(),
            level,
            message,
        });
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }
}

pub async fn run_tui(rpc: RpcClient, payer: Keypair) -> Result<(), anyhow::Error> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new();
    app.add_log(LogLevel::Info, "ORE Mining Terminal started".to_string());
    app.add_log(
        LogLevel::Info,
        format!("Wallet: {}", payer.pubkey().to_string()),
    );

    let start_time = Instant::now();

    let res = run_app(&mut terminal, &mut app, &rpc, &payer, start_time).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error: {:?}", err);
    }

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut AppState,
    rpc: &RpcClient,
    payer: &Keypair,
    start_time: Instant,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        // Handle input with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Right => app.tab = app.tab.next(),
                    KeyCode::Left => app.tab = app.tab.prev(),
                    KeyCode::Char('1') => app.tab = Tab::Dashboard,
                    KeyCode::Char('2') => app.tab = Tab::Board,
                    KeyCode::Char('3') => app.tab = Tab::Miner,
                    KeyCode::Char('4') => app.tab = Tab::Stake,
                    KeyCode::Char('5') => app.tab = Tab::Logs,
                    KeyCode::Char('m') => {
                        app.auto_mine = !app.auto_mine;
                        app.add_log(
                            LogLevel::Info,
                            format!("Auto-mine: {}", if app.auto_mine { "ON" } else { "OFF" }),
                        );
                    }
                    KeyCode::Char('c') => {
                        app.auto_checkpoint = !app.auto_checkpoint;
                        app.add_log(
                            LogLevel::Info,
                            format!(
                                "Auto-checkpoint: {}",
                                if app.auto_checkpoint { "ON" } else { "OFF" }
                            ),
                        );
                    }
                    KeyCode::Char('d') => {
                        // Manual deploy
                        if let Err(e) = deploy_manual(rpc, payer, app).await {
                            app.add_log(LogLevel::Error, format!("Deploy failed: {}", e));
                        }
                    }
                    KeyCode::Char('r') => {
                        // Manual claim rewards
                        if let Err(e) = claim_rewards(rpc, payer, app).await {
                            app.add_log(LogLevel::Error, format!("Claim failed: {}", e));
                        }
                    }
                    KeyCode::Char('+') => {
                        app.deploy_amount = (app.deploy_amount * 2).min(100 * LAMPORTS_PER_SOL);
                        app.add_log(
                            LogLevel::Info,
                            format!("Deploy amount: {} SOL", lamports_to_sol(app.deploy_amount)),
                        );
                    }
                    KeyCode::Char('-') => {
                        app.deploy_amount = (app.deploy_amount / 2).max(LAMPORTS_PER_SOL / 1000);
                        app.add_log(
                            LogLevel::Info,
                            format!("Deploy amount: {} SOL", lamports_to_sol(app.deploy_amount)),
                        );
                    }
                    _ => {}
                }
            }
        }

        // Update data periodically
        if app.last_update.elapsed() >= REFRESH_RATE {
            update_data(rpc, payer, app).await;
            app.stats.uptime = start_time.elapsed();
            app.last_update = Instant::now();

            // Auto-mining logic
            if app.auto_mine {
                if let Err(e) = auto_mine_logic(rpc, payer, app).await {
                    app.add_log(LogLevel::Error, format!("Auto-mine error: {}", e));
                }
            }
        }
    }
}

async fn update_data(rpc: &RpcClient, payer: &Keypair, app: &mut AppState) {
    // Fetch board
    if let Ok(board) = get_board(rpc).await {
        app.board = Some(board);

        // Fetch round
        if let Ok(round) = get_round(rpc, board.round_id).await {
            app.round = Some(round);
        }
    }

    // Fetch clock
    if let Ok(clock) = get_clock(rpc).await {
        app.clock = Some(clock);
    }

    // Fetch miner
    if let Ok(miner) = get_miner(rpc, payer.pubkey()).await {
        app.miner = Some(miner);
    }

    // Fetch stake
    if let Ok(stake) = get_stake(rpc, payer.pubkey()).await {
        app.stake = Some(stake);
    }

    // Fetch treasury
    if let Ok(treasury) = get_treasury(rpc).await {
        app.treasury = Some(treasury);
    }
}

async fn auto_mine_logic(
    rpc: &RpcClient,
    payer: &Keypair,
    app: &mut AppState,
) -> Result<(), anyhow::Error> {
    let Some(ref board) = app.board else {
        return Ok(());
    };
    let Some(ref clock) = app.clock else {
        return Ok(());
    };

    // Check if we need to checkpoint
    if app.auto_checkpoint {
        if let Some(miner) = &app.miner {
            if miner.checkpoint_id < miner.round_id {
                let ix = ore_api::sdk::checkpoint(payer.pubkey(), payer.pubkey(), miner.round_id);
                match submit_transaction(rpc, payer, &[ix]).await {
                    Ok(_) => {
                        app.add_log(
                            LogLevel::Success,
                            format!("Checkpointed round {}", miner.round_id),
                        );
                    }
                    Err(e) => {
                        app.add_log(LogLevel::Error, format!("Checkpoint failed: {}", e));
                    }
                }
            }
        }
    }

    // Check if we're in a mining window
    if clock.slot >= board.start_slot && clock.slot < board.end_slot {
        // Check if we've already deployed this round
        if let Some(miner) = &app.miner {
            if miner.round_id == board.round_id {
                // Already deployed
                return Ok(());
            }
        }

        // Deploy
        deploy_manual(rpc, payer, app).await?;
    }

    // Check if round ended and we can claim
    if clock.slot >= board.end_slot {
        if let Some(miner) = &app.miner {
            if miner.rewards_sol > 0 || miner.rewards_ore > 0 {
                claim_rewards(rpc, payer, app).await?;
            }
        }
    }

    Ok(())
}

async fn deploy_manual(
    rpc: &RpcClient,
    payer: &Keypair,
    app: &mut AppState,
) -> Result<(), anyhow::Error> {
    let Some(board) = app.board else {
        return Err(anyhow::anyhow!("Board not loaded"));
    };

    let ix = ore_api::sdk::deploy(
        payer.pubkey(),
        payer.pubkey(),
        app.deploy_amount,
        board.round_id,
        app.selected_squares,
    );

    match submit_transaction(rpc, payer, &[ix]).await {
        Ok(_) => {
            let square_count = app.selected_squares.iter().filter(|&&x| x).count();
            app.add_log(
                LogLevel::Success,
                format!(
                    "Deployed {} SOL to {} squares (Round #{})",
                    lamports_to_sol(app.deploy_amount),
                    square_count,
                    board.round_id
                ),
            );
            app.stats.rounds_mined += 1;
            Ok(())
        }
        Err(e) => {
            app.add_log(LogLevel::Error, format!("Deploy failed: {}", e));
            Err(e)
        }
    }
}

async fn claim_rewards(
    rpc: &RpcClient,
    payer: &Keypair,
    app: &mut AppState,
) -> Result<(), anyhow::Error> {
    let ix_sol = ore_api::sdk::claim_sol(payer.pubkey());
    let ix_ore = ore_api::sdk::claim_ore(payer.pubkey());

    match submit_transaction(rpc, payer, &[ix_sol, ix_ore]).await {
        Ok(_) => {
            if let Some(miner) = &app.miner {
                let sol = miner.rewards_sol;
                let ore = miner.rewards_ore + miner.refined_ore;
                app.stats.total_sol_earned += sol;
                app.stats.total_ore_earned += ore;
                if sol > 0 || ore > 0 {
                    app.stats.rounds_won += 1;
                }
                app.add_log(
                    LogLevel::Success,
                    format!(
                        "Claimed {} SOL + {} ORE",
                        lamports_to_sol(sol),
                        amount_to_ui_amount(ore, TOKEN_DECIMALS)
                    ),
                );
            }
            Ok(())
        }
        Err(e) => {
            app.add_log(LogLevel::Error, format!("Claim failed: {}", e));
            Err(e)
        }
    }
}

fn ui(f: &mut Frame, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    // Header with tabs
    render_header(f, chunks[0], app);

    // Main content
    match app.tab {
        Tab::Dashboard => render_dashboard(f, chunks[1], app),
        Tab::Board => render_board(f, chunks[1], app),
        Tab::Miner => render_miner(f, chunks[1], app),
        Tab::Stake => render_stake(f, chunks[1], app),
        Tab::Logs => render_logs(f, chunks[1], app),
    }

    // Footer with controls
    render_footer(f, chunks[2], app);
}

fn render_header(f: &mut Frame, area: Rect, app: &AppState) {
    let titles = vec!["Dashboard", "Board", "Miner", "Stake", "Logs"];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("ORE Mining Terminal"))
        .select(match app.tab {
            Tab::Dashboard => 0,
            Tab::Board => 1,
            Tab::Miner => 2,
            Tab::Stake => 3,
            Tab::Logs => 4,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    f.render_widget(tabs, area);
}

fn render_footer(f: &mut Frame, area: Rect, app: &AppState) {
    let controls = vec![
        Span::raw("q: Quit | "),
        Span::raw("←→: Switch tabs | "),
        Span::raw("m: Toggle auto-mine | "),
        Span::raw("c: Toggle auto-checkpoint | "),
        Span::raw("d: Deploy | "),
        Span::raw("r: Claim | "),
        Span::raw("+/-: Amount"),
    ];
    let controls_text = Line::from(controls);
    let footer = Paragraph::new(controls_text)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    f.render_widget(footer, area);
}

fn render_dashboard(f: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(area);

    // Status section
    let mut status_lines = vec![];
    status_lines.push(Line::from(vec![
        Span::styled("Auto-mine: ", Style::default().fg(Color::Gray)),
        Span::styled(
            if app.auto_mine { "ON" } else { "OFF" },
            Style::default().fg(if app.auto_mine {
                Color::Green
            } else {
                Color::Red
            }),
        ),
        Span::raw("  "),
        Span::styled("Auto-checkpoint: ", Style::default().fg(Color::Gray)),
        Span::styled(
            if app.auto_checkpoint { "ON" } else { "OFF" },
            Style::default().fg(if app.auto_checkpoint {
                Color::Green
            } else {
                Color::Red
            }),
        ),
    ]));

    status_lines.push(Line::from(vec![
        Span::styled("Deploy amount: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{} SOL", lamports_to_sol(app.deploy_amount)),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    if let Some(board) = &app.board {
        status_lines.push(Line::from(vec![
            Span::styled("Current round: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("#{}", board.round_id),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        if let Some(clock) = &app.clock {
            let time_remaining = board.end_slot.saturating_sub(clock.slot) as f64 * 0.4;
            status_lines.push(Line::from(vec![
                Span::styled("Time remaining: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{:.1}s", time_remaining),
                    Style::default().fg(if time_remaining < 10.0 {
                        Color::Red
                    } else {
                        Color::Green
                    }),
                ),
            ]));
        }
    }

    let status = Paragraph::new(status_lines)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[0]);

    // Stats section
    let mut stats_lines = vec![];
    stats_lines.push(Line::from(vec![
        Span::styled("Uptime: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.0}s", app.stats.uptime.as_secs()),
            Style::default().fg(Color::White),
        ),
    ]));
    stats_lines.push(Line::from(vec![
        Span::styled("Rounds mined: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.stats.rounds_mined),
            Style::default().fg(Color::White),
        ),
    ]));
    stats_lines.push(Line::from(vec![
        Span::styled("Rounds won: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", app.stats.rounds_won),
            Style::default().fg(Color::Green),
        ),
    ]));
    stats_lines.push(Line::from(vec![
        Span::styled("Total SOL earned: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.4}", lamports_to_sol(app.stats.total_sol_earned)),
            Style::default().fg(Color::Yellow),
        ),
    ]));
    stats_lines.push(Line::from(vec![
        Span::styled("Total ORE earned: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.4}", amount_to_ui_amount(app.stats.total_ore_earned, TOKEN_DECIMALS)),
            Style::default().fg(Color::Magenta),
        ),
    ]));

    let stats = Paragraph::new(stats_lines)
        .block(Block::default().borders(Borders::ALL).title("Mining Stats"))
        .wrap(Wrap { trim: true });
    f.render_widget(stats, chunks[1]);

    // Recent logs
    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(10)
        .map(|log| {
            let color = match log.level {
                LogLevel::Info => Color::White,
                LogLevel::Success => Color::Green,
                LogLevel::Warning => Color::Yellow,
                LogLevel::Error => Color::Red,
            };
            let time = log.timestamp.format("%H:%M:%S");
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", time), Style::default().fg(Color::Gray)),
                Span::styled(&log.message, Style::default().fg(color)),
            ]))
        })
        .collect();

    let logs_widget = List::new(logs).block(Block::default().borders(Borders::ALL).title("Recent Logs"));
    f.render_widget(logs_widget, chunks[2]);
}

fn render_board(f: &mut Frame, area: Rect, app: &AppState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(area);

    let Some(round) = &app.round else {
        let text = Paragraph::new("Loading board data...")
            .block(Block::default().borders(Borders::ALL).title("Board"));
        f.render_widget(text, chunks[0]);
        return;
    };

    // Display 5x5 grid
    let mut rows = vec![];
    for row_idx in 0..5 {
        let mut cells = vec![Cell::from(format!("Row {}", row_idx + 1))];
        for col_idx in 0..5 {
            let square_id = row_idx * 5 + col_idx;
            let deployed = round.deployed[square_id];
            let count = round.count[square_id];
            let cell_text = if deployed > 0 {
                format!(
                    "{:.3} SOL\n{} miners",
                    lamports_to_sol(deployed),
                    count
                )
            } else {
                "Empty".to_string()
            };
            cells.push(Cell::from(cell_text));
        }
        rows.push(Row::new(cells).height(2));
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(8),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
            Constraint::Percentage(18),
        ],
    )
    .header(Row::new(vec!["", "Col 0", "Col 1", "Col 2", "Col 3", "Col 4"]).style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("5x5 Board - Round #{}", round.id)),
    )
    .column_spacing(1);

    f.render_widget(table, chunks[0]);
}

fn render_miner(f: &mut Frame, area: Rect, app: &AppState) {
    let Some(miner) = &app.miner else {
        let text = Paragraph::new("No miner account found. Deploy to create one.")
            .block(Block::default().borders(Borders::ALL).title("Miner"));
        f.render_widget(text, area);
        return;
    };

    let mut lines = vec![];
    lines.push(Line::from(vec![
        Span::styled("Authority: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", miner.authority),
            Style::default().fg(Color::White),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Current round: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", miner.round_id),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Checkpoint ID: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{}", miner.checkpoint_id),
            Style::default().fg(Color::Cyan),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![Span::styled(
        "Pending Rewards:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![
        Span::styled("  SOL: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.6}", lamports_to_sol(miner.rewards_sol)),
            Style::default().fg(Color::Green),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ORE (unrefined): ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.6}", amount_to_ui_amount(miner.rewards_ore, TOKEN_DECIMALS)),
            Style::default().fg(Color::Magenta),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ORE (refined): ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.6}", amount_to_ui_amount(miner.refined_ore, TOKEN_DECIMALS)),
            Style::default().fg(Color::Magenta),
        ),
    ]));
    lines.push(Line::from(""));

    lines.push(Line::from(vec![Span::styled(
        "Lifetime Stats:",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(vec![
        Span::styled("  Total SOL earned: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.6}", lamports_to_sol(miner.lifetime_rewards_sol)),
            Style::default().fg(Color::Green),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Total ORE earned: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.6}", amount_to_ui_amount(miner.lifetime_rewards_ore, TOKEN_DECIMALS)),
            Style::default().fg(Color::Magenta),
        ),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Miner Info"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_stake(f: &mut Frame, area: Rect, app: &AppState) {
    let Some(stake) = &app.stake else {
        let text = Paragraph::new("No stake account found.")
            .block(Block::default().borders(Borders::ALL).title("Stake"));
        f.render_widget(text, area);
        return;
    };

    let mut lines = vec![];
    lines.push(Line::from(vec![
        Span::styled("Staked balance: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.6} ORE", amount_to_ui_amount(stake.balance, TOKEN_DECIMALS)),
            Style::default().fg(Color::Magenta),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Pending rewards: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!("{:.6} ORE", amount_to_ui_amount(stake.rewards, TOKEN_DECIMALS)),
            Style::default().fg(Color::Green),
        ),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Lifetime rewards: ", Style::default().fg(Color::Gray)),
        Span::styled(
            format!(
                "{:.6} ORE",
                amount_to_ui_amount(stake.lifetime_rewards, TOKEN_DECIMALS)
            ),
            Style::default().fg(Color::Yellow),
        ),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title("Stake Info"))
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, area);
}

fn render_logs(f: &mut Frame, area: Rect, app: &AppState) {
    let logs: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .map(|log| {
            let color = match log.level {
                LogLevel::Info => Color::White,
                LogLevel::Success => Color::Green,
                LogLevel::Warning => Color::Yellow,
                LogLevel::Error => Color::Red,
            };
            let time = log.timestamp.format("%H:%M:%S");
            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", time), Style::default().fg(Color::Gray)),
                Span::styled(&log.message, Style::default().fg(color)),
            ]))
        })
        .collect();

    let logs_widget = List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));
    f.render_widget(logs_widget, area);
}
