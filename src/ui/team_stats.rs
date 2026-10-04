use crate::models::GameSummary;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

pub fn render_team_stats(f: &mut Frame, area: Rect, summary: &GameSummary) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),   // Team comparison stats table
            Constraint::Length(5), // Game info (venue, weather, TV)
        ])
        .split(area);

    // 1. Team Stats Table
    render_stats_table(f, chunks[0], summary);

    // 2. Game Info Block
    render_game_info(f, chunks[1], summary);
}

fn render_stats_table(f: &mut Frame, area: Rect, summary: &GameSummary) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Team Comparison ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    let Some(boxscore) = &summary.boxscore else {
        let p = Paragraph::new("No team statistics available yet.").block(block);
        f.render_widget(p, area);
        return;
    };

    if boxscore.teams.len() < 2 {
        let p = Paragraph::new("Incomplete team data.").block(block);
        f.render_widget(p, area);
        return;
    }

    let (away_id, home_id) = if let Some(hdr) = &summary.header {
        let away = hdr.competitions.first().and_then(|c| {
            c.competitors.iter().find(|comp| comp.home_away.as_deref() == Some("away")).and_then(|comp| comp.id.as_deref())
        });
        let home = hdr.competitions.first().and_then(|c| {
            c.competitors.iter().find(|comp| comp.home_away.as_deref() == Some("home")).and_then(|comp| comp.id.as_deref())
        });
        (away, home)
    } else {
        (None, None)
    };

    let away_team = boxscore.teams.iter().find(|t| {
        t.team.id.as_deref() == away_id
    }).unwrap_or(&boxscore.teams[0]);

    let home_team = boxscore.teams.iter().find(|t| {
        t.team.id.as_deref() == home_id
    }).unwrap_or_else(|| {
        boxscore.teams.get(1).unwrap_or(&boxscore.teams[0])
    });

    let away_name = away_team
        .team
        .display_name
        .as_deref()
        .or(away_team.team.abbreviation.as_deref())
        .unwrap_or("Away");

    let home_name = home_team
        .team
        .display_name
        .as_deref()
        .or(home_team.team.abbreviation.as_deref())
        .unwrap_or("Home");

    let header = Row::new(vec![
        Cell::from(away_name).style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("STATISTIC").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from(home_name).style(Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
    ])
    .height(1)
    .bottom_margin(1);

    // Collect matched statistics
    let mut rows = Vec::new();
    for stat in &away_team.statistics {
        let label = stat.label.as_deref().or(stat.name.as_deref()).unwrap_or("");
        let away_val = stat.display_value.as_deref().unwrap_or("-");

        // Find matching home stat
        let home_val = home_team
            .statistics
            .iter()
            .find(|s| s.name == stat.name)
            .and_then(|s| s.display_value.as_deref())
            .unwrap_or("-");

        rows.push(Row::new(vec![
            Cell::from(away_val).style(Style::default().fg(Color::White)),
            Cell::from(label).style(Style::default().fg(Color::DarkGray)),
            Cell::from(home_val).style(Style::default().fg(Color::White)),
        ]));
    }

    let widths = [
        Constraint::Percentage(35),
        Constraint::Percentage(30),
        Constraint::Percentage(35),
    ];

    let table = Table::new(rows, widths).header(header).block(block);
    f.render_widget(table, area);
}

fn render_game_info(f: &mut Frame, area: Rect, summary: &GameSummary) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Venue & Weather ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));

    let mut lines = Vec::new();

    if let Some(info) = &summary.game_info {
        if let Some(v) = &info.venue {
            let name = v.full_name.as_deref().unwrap_or("Unknown Stadium");
            let loc = match (&v.address.as_ref().and_then(|a| a.city.as_deref()), &v.address.as_ref().and_then(|a| a.state.as_deref())) {
                (Some(c), Some(s)) => format!(" ({}, {})", c, s),
                (Some(c), None) => format!(" ({})", c),
                _ => "".to_string(),
            };
            lines.push(Line::from(vec![
                Span::styled("Venue: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{}{}", name, loc), Style::default().fg(Color::White)),
            ]));
        }

        if let Some(w) = &info.weather {
            let temp = w
                .temperature
                .map(|t| format!("{}°F", t))
                .or_else(|| w.display_value.clone())
                .unwrap_or_else(|| "--".to_string());
            let cond = w.condition_id.as_deref().unwrap_or("");
            lines.push(Line::from(vec![
                Span::styled("Weather: ", Style::default().fg(Color::Yellow)),
                Span::styled(format!("{} {}", temp, cond), Style::default().fg(Color::White)),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from("No additional game info available."));
    }

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
