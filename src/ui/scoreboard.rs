use crate::app::{App, FilterMode};
use crate::models::{Competitor, Event};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_scoreboard(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top header bar (Title, Filter, Status)
            Constraint::Min(5),    // List of games
            Constraint::Length(1), // Bottom shortcut help bar
        ])
        .split(area);

    // 1. Header Bar
    render_header_bar(f, chunks[0], app);

    // 2. Games List
    render_games_list(f, chunks[1], app);

    // 3. Bottom Bar
    render_bottom_bar(f, chunks[2], app);
}

fn render_header_bar(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let filter_str = match app.filter_mode {
        FilterMode::All => "All FBS",
        FilterMode::Favorites => "Favorites ★",
        FilterMode::Live => "Live Now ●",
        FilterMode::Top25 => "Top 25 Rank",
    };

    let total_games = app.scoreboard.as_ref().map(|s| s.events.len()).unwrap_or(0);
    let filtered_count = app.filtered_events().len();

    let loading_str = if app.is_loading { " ⏳ Fetching..." } else { "" };

    let header_line = Line::from(vec![
        Span::styled(
            " 🏈 COLLEGE FOOTBALL LIVE ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" [Filter: {} (Tab)] ", filter_str),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" Showing {}/{} games ", filtered_count, total_games),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(loading_str, Style::default().fg(Color::LightMagenta)),
    ]);

    f.render_widget(Paragraph::new(header_line), inner);
}

fn render_games_list(f: &mut Frame, area: Rect, app: &App) {
    let events = app.filtered_events();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Games [j/k to select, Enter to open] ");

    if events.is_empty() {
        let msg = if app.is_loading {
            "Loading college football scores from ESPN..."
        } else {
            "No games found matching current filter. Press [Tab] to cycle filters."
        };
        let p = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = events
        .iter()
        .enumerate()
        .map(|(idx, ev)| {
            let is_selected = idx == app.selected_game_index;
            format_game_list_item(ev, is_selected, app)
        })
        .collect();

    // Render list
    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn format_game_list_item(ev: &Event, is_selected: bool, app: &App) -> ListItem<'static> {
    let comp = ev.competitions.first();
    let away = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "away"));
    let home = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "home"));

    let away_name = away.map(|a| a.team.display_name.clone()).unwrap_or_else(|| "Away".to_string());
    let home_name = home.map(|h| h.team.display_name.clone()).unwrap_or_else(|| "Home".to_string());

    let away_abbrev = away.map(|a| a.team.abbreviation.as_str()).unwrap_or("");
    let home_abbrev = home.map(|h| h.team.abbreviation.as_str()).unwrap_or("");

    let away_id = away.map(|a| a.team.id.as_str()).unwrap_or("");
    let home_id = home.map(|h| h.team.id.as_str()).unwrap_or("");

    let away_fav = app.config.is_favorite(away_abbrev, away_id);
    let home_fav = app.config.is_favorite(home_abbrev, home_id);

    let away_score = away.and_then(|a| a.score.as_deref()).unwrap_or("-");
    let home_score = home.and_then(|h| h.score.as_deref()).unwrap_or("-");

    let away_rank = format_rank(away);
    let home_rank = format_rank(home);

    let away_rec = away.and_then(|a| a.records.first()).and_then(|r| r.summary.as_deref()).unwrap_or("");
    let home_rec = home.and_then(|h| h.records.first()).and_then(|r| r.summary.as_deref()).unwrap_or("");

    // Status & Situation
    let is_live = ev.status.status_type.state == "in";
    let is_final = ev.status.status_type.completed;

    let status_color = if is_live {
        Color::LightGreen
    } else if is_final {
        Color::DarkGray
    } else {
        Color::Cyan
    };

    let status_badge = if is_live {
        format!("● {}", ev.status.status_type.short_detail)
    } else {
        ev.status.status_type.short_detail.clone()
    };

    // Live down & distance or broadcast
    let situation_str = if is_live {
        if let Some(sit) = comp.and_then(|c| c.situation.as_ref()) {
            if let Some(dd) = &sit.down_distance_text {
                format!(" | 🏈 {}", dd)
            } else if let Some(d) = sit.down {
                if (1..=4).contains(&d) {
                    let d_str = match d {
                        1 => "1st",
                        2 => "2nd",
                        3 => "3rd",
                        4 => "4th",
                        _ => "Down",
                    };
                    let dist_str = match sit.distance {
                        Some(0) => "Goal".to_string(),
                        Some(dist) if dist > 0 => dist.to_string(),
                        _ => "Goal".to_string(),
                    };
                    if let Some(pos) = &sit.possession_text {
                        format!(" | 🏈 {} & {} at {}", d_str, dist_str, pos)
                    } else {
                        format!(" | 🏈 {} & {}", d_str, dist_str)
                    }
                } else if let Some(last_p) = &sit.last_play {
                    if let Some(txt) = &last_p.text {
                        let truncated: String = txt.chars().take(40).collect();
                        format!(" | {}", truncated)
                    } else {
                        "".to_string()
                    }
                } else {
                    "".to_string()
                }
            } else if let Some(last_p) = &sit.last_play {
                if let Some(txt) = &last_p.text {
                    let truncated: String = txt.chars().take(40).collect();
                    format!(" | {}", truncated)
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        }
    } else if !is_final {
        if let Some(bc) = comp.and_then(|c| c.broadcasts.first()).and_then(|b| b.names.first()) {
            format!(" ({})", bc)
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    let star_str = if away_fav || home_fav { "★ " } else { "  " };
    let star_color = if away_fav || home_fav { Color::Yellow } else { Color::Reset };

    let pointer = if is_selected { "▶ " } else { "  " };

    // Line 1: Away vs Home score + status
    let line1 = Line::from(vec![
        Span::styled(pointer, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::styled(star_str, Style::default().fg(star_color)),
        Span::styled(away_rank, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:<20} ", away_name),
            Style::default().fg(Color::White).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
        ),
        Span::styled(format!("({:<4}) ", away_rec), Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:>3} ", away_score),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled("@ ", Style::default().fg(Color::DarkGray)),
        Span::styled(home_rank, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("{:<20} ", home_name),
            Style::default().fg(Color::White).add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() }),
        ),
        Span::styled(format!("({:<4}) ", home_rec), Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{:>3}   ", home_score),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(status_badge, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        Span::styled(situation_str, Style::default().fg(Color::Yellow)),
    ]);

    let bg_color = if is_selected {
        Color::Rgb(30, 40, 55)
    } else {
        Color::Reset
    };

    ListItem::new(vec![line1]).style(Style::default().bg(bg_color))
}

fn format_rank(competitor: Option<&Competitor>) -> String {
    if let Some(comp) = competitor {
        if let Some(rank_info) = &comp.curated_rank {
            if let Some(rank) = rank_info.current {
                if rank > 0 && rank <= 25 {
                    return format!("#{} ", rank);
                }
            }
        }
    }
    "".to_string()
}

fn render_bottom_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_line = if let Some((msg, _)) = &app.status_message {
        msg.clone()
    } else {
        "".to_string()
    };

    let line = Line::from(vec![
        Span::styled(" [j/k] Navigate ", Style::default().fg(Color::Yellow)),
        Span::styled(" [Enter] Details ", Style::default().fg(Color::Yellow)),
        Span::styled(" [f] Favorite Team ", Style::default().fg(Color::Yellow)),
        Span::styled(" [Tab] Filter ", Style::default().fg(Color::Yellow)),
        Span::styled(" [r] Refresh ", Style::default().fg(Color::Yellow)),
        Span::styled(" [?] Help ", Style::default().fg(Color::Yellow)),
        Span::styled(" [q] Quit ", Style::default().fg(Color::Yellow)),
        Span::styled(format!("  {}", status_line), Style::default().fg(Color::Green)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
