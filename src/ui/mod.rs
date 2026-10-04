pub mod boxscore;
pub mod field;
pub mod gamecast;
pub mod help;
pub mod plays;
pub mod scoreboard;
pub mod team_stats;
pub mod touchdown;

use crate::app::{App, DetailTab, ViewMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

pub fn render(f: &mut Frame, app: &App) {
    let size = f.area();

    match app.view_mode {
        ViewMode::Scoreboard => {
            scoreboard::render_scoreboard(f, size, app);
        }
        ViewMode::GameDetail => {
            render_game_detail_view(f, size, app);
        }
    }

    // Modal overlays
    if let Some(td) = &app.touchdown_celebration {
        touchdown::render_touchdown_overlay(f, size, td);
    } else if app.show_fav_dialog {
        help::render_fav_dialog(f, size, app);
    } else if app.show_help {
        help::render_help_modal(f, size);
    }
}

fn render_game_detail_view(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Detail tabs
            Constraint::Min(5),    // Active tab content
            Constraint::Length(1), // Bottom shortcut help bar
        ])
        .split(area);

    // 1. Detail Tabs
    let tabs_list = DetailTab::all();
    let current_idx = tabs_list
        .iter()
        .position(|t| *t == app.detail_tab)
        .unwrap_or(0);

    let tab_titles: Vec<Line> = tabs_list
        .iter()
        .map(|t| Line::from(t.title()))
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL).title(" Gamecast Navigation "))
        .select(current_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, chunks[0]);

    // 2. Active Tab Content
    let Some(summary) = &app.active_summary else {
        let msg = if app.is_loading {
            "Loading game summary & play-by-play from ESPN..."
        } else {
            "No data available for this game."
        };
        let p = Paragraph::new(msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, chunks[1]);
        render_detail_bottom_bar(f, chunks[2], app);
        return;
    };

    let event = app.scoreboard.as_ref().and_then(|sb| {
        sb.events
            .iter()
            .find(|ev| Some(&ev.id) == app.active_game_id.as_ref())
    });

    match app.detail_tab {
        DetailTab::Gamecast => gamecast::render_gamecast(f, chunks[1], app, summary, event),
        DetailTab::Plays => plays::render_plays(f, chunks[1], app, summary),
        DetailTab::BoxScore => boxscore::render_boxscore(f, chunks[1], app, summary),
        DetailTab::TeamStats => team_stats::render_team_stats(f, chunks[1], summary),
    }

    // 3. Bottom Bar
    render_detail_bottom_bar(f, chunks[2], app);
}

fn render_detail_bottom_bar(f: &mut Frame, area: Rect, app: &App) {
    let status_line = if let Some((msg, _)) = &app.status_message {
        msg.clone()
    } else {
        "".to_string()
    };

    let line = Line::from(vec![
        ratatui::text::Span::styled(" [Esc/q] Back ", Style::default().fg(Color::Yellow)),
        ratatui::text::Span::styled(" [1-4/Tab] Tabs ", Style::default().fg(Color::Yellow)),
        ratatui::text::Span::styled(" [t] TD Celebration ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ratatui::text::Span::styled(" [r] Refresh ", Style::default().fg(Color::Yellow)),
        ratatui::text::Span::styled(" [?] Help ", Style::default().fg(Color::Yellow)),
        ratatui::text::Span::styled(format!("  {}", status_line), Style::default().fg(Color::Green)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
