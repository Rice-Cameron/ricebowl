use crate::app::{App, BoxScoreCategory};
use crate::models::{BoxScorePlayers, GameSummary};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs},
    Frame,
};

pub fn render_boxscore(f: &mut Frame, area: Rect, app: &App, summary: &GameSummary) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Category tabs
            Constraint::Min(5),    // Player stats table(s)
        ])
        .split(area);

    // 1. Category Tabs
    let categories = BoxScoreCategory::all();
    let current_cat_idx = categories
        .iter()
        .position(|c| *c == app.boxscore_category)
        .unwrap_or(0);

    let tab_titles: Vec<Line> = categories
        .iter()
        .map(|c| Line::from(c.title()))
        .collect();

    let tabs = Tabs::new(tab_titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Stat Categories [h/l to switch] "),
        )
        .select(current_cat_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, chunks[0]);

    // 2. Player Stats Tables (Away & Home side-by-side or stacked)
    let Some(boxscore) = &summary.boxscore else {
        let p = Paragraph::new("No box score data available yet.")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, chunks[1]);
        return;
    };

    if boxscore.players.is_empty() {
        let p = Paragraph::new("Box score is empty.")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, chunks[1]);
        return;
    }

    // Split horizontally for two teams if terminal is wide enough, else vertically
    let is_wide = chunks[1].width >= 100;
    let team_chunks = if is_wide {
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1])
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1])
    };

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

    let away_players = boxscore.players.iter().find(|p| {
        p.team.id.as_deref() == away_id
    }).unwrap_or(&boxscore.players[0]);

    let home_players = boxscore.players.iter().find(|p| {
        p.team.id.as_deref() == home_id
    }).unwrap_or_else(|| {
        boxscore.players.get(1).unwrap_or(&boxscore.players[0])
    });

    render_team_category_table(f, team_chunks[0], away_players, app.boxscore_category);
    if team_chunks.len() > 1 && boxscore.players.len() > 1 {
        render_team_category_table(f, team_chunks[1], home_players, app.boxscore_category);
    }
}

fn render_team_category_table(
    f: &mut Frame,
    area: Rect,
    team_players: &BoxScorePlayers,
    category: BoxScoreCategory,
) {
    let team_name = team_players
        .team
        .display_name
        .as_deref()
        .unwrap_or("Team");

    let cat_stats = team_players
        .statistics
        .iter()
        .find(|s| s.name.as_deref() == Some(category.name()));

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" {} - {} ", team_name, category.title()),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));

    let Some(stats) = cat_stats else {
        let p = Paragraph::new(format!("No {} stats recorded", category.title())).block(block);
        f.render_widget(p, area);
        return;
    };

    if stats.athletes.is_empty() {
        let p = Paragraph::new("No players listed").block(block);
        f.render_widget(p, area);
        return;
    }

    // Header row
    let mut header_cells = vec![Cell::from("Player").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))];
    for label in &stats.labels {
        header_cells.push(
            Cell::from(label.as_str())
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        );
    }
    let header = Row::new(header_cells).height(1).bottom_margin(1);

    // Player rows
    let rows: Vec<Row> = stats
        .athletes
        .iter()
        .map(|ath| {
            let name = ath
                .athlete
                .short_name
                .as_deref()
                .or(ath.athlete.display_name.as_deref())
                .unwrap_or("Unknown");

            let jersey_pos = match (&ath.athlete.jersey, &ath.athlete.position) {
                (Some(j), Some(p)) if p.abbreviation.is_some() => {
                    format!(" #{} ({})", j, p.abbreviation.as_ref().unwrap())
                }
                (Some(j), _) => format!(" #{}", j),
                _ => "".to_string(),
            };

            let mut cells = vec![Cell::from(format!("{}{}", name, jersey_pos))];
            for stat in &ath.stats {
                cells.push(Cell::from(stat.as_str()));
            }
            Row::new(cells)
        })
        .collect();

    // Calculate column constraints
    let mut widths = vec![Constraint::Min(16)]; // Player name column
    for _ in &stats.labels {
        widths.push(Constraint::Length(8));
    }

    let table = Table::new(rows, widths).header(header).block(block);
    f.render_widget(table, area);
}
