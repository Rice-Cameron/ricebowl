mod api;
mod app;
mod config;
mod models;
mod ui;

use api::EspnClient;
use app::{App, DetailTab, ViewMode};
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use models::{GameSummary, ScoreboardResponse};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::stdout;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

enum Action {
    FetchScoreboard,
    FetchSummary(String),
}

enum AppEvent {
    Input(CEvent),
    ScoreboardResult(Result<ScoreboardResponse, String>),
    SummaryResult(String, Result<GameSummary, String>),
    Tick,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Setup terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Set panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));

    // 3. Channels & Background Workers
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(100);
    let (action_tx, mut action_rx) = mpsc::channel::<Action>(20);

    // Input listener thread
    let input_tx = event_tx.clone();
    tokio::task::spawn_blocking(move || {
        loop {
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(evt) = event::read() {
                    if input_tx.blocking_send(AppEvent::Input(evt)).is_err() {
                        break;
                    }
                }
            }
        }
    });

    // Network worker task
    let net_tx = event_tx.clone();
    let client = EspnClient::new();
    tokio::spawn(async move {
        while let Some(action) = action_rx.recv().await {
            let client = client.clone();
            let net_tx = net_tx.clone();
            match action {
                Action::FetchScoreboard => {
                    tokio::spawn(async move {
                        let res = client.fetch_scoreboard(true).await.map_err(|e| e.to_string());
                        let _ = net_tx.send(AppEvent::ScoreboardResult(res)).await;
                    });
                }
                Action::FetchSummary(event_id) => {
                    tokio::spawn(async move {
                        let res = client.fetch_summary(&event_id).await.map_err(|e| e.to_string());
                        let _ = net_tx.send(AppEvent::SummaryResult(event_id, res)).await;
                    });
                }
            }
        }
    });

    // Periodic tick task (every 15s for auto-refresh)
    let tick_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            if tick_tx.send(AppEvent::Tick).await.is_err() {
                break;
            }
        }
    });

    // 4. Initialize App
    let mut app = App::new();

    // Trigger initial fetch
    let _ = action_tx.send(Action::FetchScoreboard).await;

    // 5. Main Event Loop
    let mut should_quit = false;

    while !should_quit {
        // Draw UI
        terminal.draw(|f| ui::render(f, &app))?;

        // Wait for next event
        if let Some(event) = event_rx.recv().await {
            match event {
                AppEvent::Input(CEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                    // Check modal dialogs first
                    if app.show_help {
                        match key.code {
                            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
                                app.show_help = false;
                            }
                            _ => {}
                        }
                    } else if app.show_fav_dialog {
                        match key.code {
                            KeyCode::Char('1') => {
                                app.toggle_favorite_for_selected(true);
                                app.show_fav_dialog = false;
                            }
                            KeyCode::Char('2') => {
                                app.toggle_favorite_for_selected(false);
                                app.show_fav_dialog = false;
                            }
                            KeyCode::Esc | KeyCode::Char('f') | KeyCode::Char('q') => {
                                app.show_fav_dialog = false;
                            }
                            _ => {}
                        }
                    } else {
                        // General navigation
                        match app.view_mode {
                            ViewMode::Scoreboard => match key.code {
                                KeyCode::Char('q') => {
                                    should_quit = true;
                                }
                                KeyCode::Char('j') | KeyCode::Down => {
                                    app.select_next_game();
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    app.select_prev_game();
                                }
                                KeyCode::Enter => {
                                    if let Some(id) = app.open_selected_game() {
                                        app.is_loading = true;
                                        app.set_status("Loading game details...");
                                        let _ = action_tx.send(Action::FetchSummary(id)).await;
                                    }
                                }
                                KeyCode::Tab => {
                                    app.filter_mode = app.filter_mode.next();
                                    app.selected_game_index = 0;
                                    app.set_status(format!("Filter: {}", app.filter_mode.label()));
                                }
                                KeyCode::Char('f') => {
                                    if app.selected_event().is_some() {
                                        app.show_fav_dialog = true;
                                    }
                                }
                                KeyCode::Char('r') => {
                                    app.is_loading = true;
                                    app.set_status("Refreshing scores...");
                                    let _ = action_tx.send(Action::FetchScoreboard).await;
                                }
                                KeyCode::Char('?') => {
                                    app.show_help = true;
                                }
                                _ => {}
                            },
                            ViewMode::GameDetail => match key.code {
                                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                                    app.close_game_detail();
                                }
                                KeyCode::Char('1') => app.detail_tab = DetailTab::Gamecast,
                                KeyCode::Char('2') => app.detail_tab = DetailTab::Plays,
                                KeyCode::Char('3') => app.detail_tab = DetailTab::BoxScore,
                                KeyCode::Char('4') => app.detail_tab = DetailTab::TeamStats,
                                KeyCode::Tab => {
                                    app.detail_tab = app.detail_tab.next();
                                }
                                KeyCode::BackTab => {
                                    app.detail_tab = app.detail_tab.prev();
                                }
                                KeyCode::Char('h') | KeyCode::Left => {
                                    if app.detail_tab == DetailTab::BoxScore {
                                        app.boxscore_category = app.boxscore_category.prev();
                                    }
                                }
                                KeyCode::Char('l') | KeyCode::Right => {
                                    if app.detail_tab == DetailTab::BoxScore {
                                        app.boxscore_category = app.boxscore_category.next();
                                    }
                                }
                                KeyCode::Char('j') | KeyCode::Down => {
                                    if app.detail_tab == DetailTab::Plays {
                                        app.plays_scroll_offset = app.plays_scroll_offset.saturating_add(1);
                                    }
                                }
                                KeyCode::Char('k') | KeyCode::Up => {
                                    if app.detail_tab == DetailTab::Plays {
                                        app.plays_scroll_offset = app.plays_scroll_offset.saturating_sub(1);
                                    }
                                }
                                KeyCode::PageDown => {
                                    if app.detail_tab == DetailTab::Plays {
                                        app.plays_scroll_offset = app.plays_scroll_offset.saturating_add(10);
                                    }
                                }
                                KeyCode::PageUp => {
                                    if app.detail_tab == DetailTab::Plays {
                                        app.plays_scroll_offset = app.plays_scroll_offset.saturating_sub(10);
                                    }
                                }
                                KeyCode::Char('r') => {
                                    if let Some(id) = app.active_game_id.clone() {
                                        app.is_loading = true;
                                        app.set_status("Refreshing game summary...");
                                        let _ = action_tx.send(Action::FetchSummary(id)).await;
                                    }
                                }
                                KeyCode::Char('?') => {
                                    app.show_help = true;
                                }
                                _ => {}
                            },
                        }
                    }
                }
                AppEvent::ScoreboardResult(res) => {
                    app.is_loading = false;
                    match res {
                        Ok(sb) => {
                            let count = sb.events.len();
                            app.scoreboard = Some(sb);
                            app.last_updated = Some(Instant::now());
                            app.set_status(format!("Updated {} games at {}", count, chrono::Local::now().format("%H:%M:%S")));
                        }
                        Err(e) => {
                            app.set_status(format!("Failed to update scoreboard: {}", e));
                        }
                    }
                }
                AppEvent::SummaryResult(event_id, res) => {
                    app.is_loading = false;
                    if app.active_game_id.as_deref() == Some(&event_id) {
                        match res {
                            Ok(summary) => {
                                app.active_summary = Some(summary);
                                app.set_status("Gamecast updated");
                            }
                            Err(e) => {
                                app.set_status(format!("Failed to load game: {}", e));
                            }
                        }
                    }
                }
                AppEvent::Tick => {
                    // Periodic auto-refresh
                    let _ = action_tx.send(Action::FetchScoreboard).await;
                    if let Some(id) = &app.active_game_id {
                        let _ = action_tx.send(Action::FetchSummary(id.clone())).await;
                    }
                }
                _ => {}
            }
        }
    }

    // 6. Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
