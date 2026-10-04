use crate::app::App;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_help_modal(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 60, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Keyboard Shortcuts & Help ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    let text = vec![
        Line::from(Span::styled("Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  j / k or ↓ / ↑      : Move selection up / down"),
        Line::from("  Enter               : Open gamecast / details for selected game"),
        Line::from("  Esc / Backspace / q : Return to scoreboard (from detail view)"),
        Line::from(""),
        Line::from(Span::styled("Game Details & Tabs", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  1 - 4 or Tab        : Switch between Gamecast, Plays, Boxscore, Team Stats"),
        Line::from("  h / l or ← / →      : Switch stat category in Boxscore view"),
        Line::from("  j / k or PgUp/PgDn  : Scroll plays or stats list"),
        Line::from("  t (on Gamecast)     : Replay / preview Touchdown celebration animation"),
        Line::from(""),
        Line::from(Span::styled("Favorites & Filters", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  f                   : Open Favorite dialog for selected game"),
        Line::from("  Tab (on Scoreboard) : Cycle filter (All FBS → Favorites → Live → Top 25)"),
        Line::from(""),
        Line::from(Span::styled("General", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from("  r                   : Force refresh live scores from ESPN"),
        Line::from("  ?                   : Toggle this help popup"),
        Line::from("  q                   : Quit application (from scoreboard)"),
        Line::from(""),
        Line::from(Span::styled("Press [Esc] or [?] to close", Style::default().fg(Color::DarkGray))),
    ];

    let p = Paragraph::new(text).block(block).alignment(Alignment::Left);
    f.render_widget(p, popup_area);
}

pub fn render_fav_dialog(f: &mut Frame, area: Rect, app: &App) {
    let popup_area = centered_rect(50, 30, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Toggle Favorite Team ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    let Some(ev) = app.selected_event() else {
        let p = Paragraph::new("No game selected").block(block);
        f.render_widget(p, popup_area);
        return;
    };

    let comp = ev.competitions.first();
    let away = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "away"));
    let home = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "home"));

    let away_name = away.map(|a| a.team.display_name.as_str()).unwrap_or("Away");
    let home_name = home.map(|h| h.team.display_name.as_str()).unwrap_or("Home");

    let away_abbrev = away.map(|a| a.team.abbreviation.as_str()).unwrap_or("");
    let home_abbrev = home.map(|h| h.team.abbreviation.as_str()).unwrap_or("");

    let away_id = away.map(|a| a.team.id.as_str()).unwrap_or("");
    let home_id = home.map(|h| h.team.id.as_str()).unwrap_or("");

    let away_fav = app.config.is_favorite(away_abbrev, away_id);
    let home_fav = app.config.is_favorite(home_abbrev, home_id);

    let away_star = if away_fav { "★ Favorited" } else { "☆ Not Favorited" };
    let home_star = if home_fav { "★ Favorited" } else { "☆ Not Favorited" };

    let text = vec![
        Line::from("Press a number key to toggle favorite:"),
        Line::from(""),
        Line::from(vec![
            Span::styled("  [1] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<25} ", away_name), Style::default().fg(Color::White)),
            Span::styled(away_star, Style::default().fg(if away_fav { Color::Yellow } else { Color::DarkGray })),
        ]),
        Line::from(vec![
            Span::styled("  [2] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{:<25} ", home_name), Style::default().fg(Color::White)),
            Span::styled(home_star, Style::default().fg(if home_fav { Color::Yellow } else { Color::DarkGray })),
        ]),
        Line::from(""),
        Line::from(Span::styled("  [Esc] or [f] to cancel", Style::default().fg(Color::DarkGray))),
    ];

    let p = Paragraph::new(text).block(block).alignment(Alignment::Left);
    f.render_widget(p, popup_area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
