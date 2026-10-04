use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub struct FieldData {
    pub down: Option<i32>,
    pub distance: Option<i32>,
    pub visual_yard_line: Option<i32>, // 0 (Away goal line on left) to 100 (Home goal line on right)
    pub is_red_zone: bool,
    pub down_distance_text: Option<String>,
    pub possession_text: Option<String>,
    pub away_team_abbrev: String,
    pub home_team_abbrev: String,
    pub possession_team_abbrev: Option<String>,
    pub is_away_possession: bool, // true if Away driving left-to-right (▶); false if Home driving right-to-left (◀)
    pub drive_summary: Option<String>,
    pub last_play_text: Option<String>,
}

pub fn render_football_field(f: &mut Frame, area: Rect, data: &FieldData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Down & Distance banner
            Constraint::Length(5), // Football field ASCII graphic
            Constraint::Length(1), // Drive summary
            Constraint::Min(2),    // Last play description
        ])
        .split(area);

    // 1. Down & Distance Header
    let down_dist_str = data
        .down_distance_text
        .as_deref()
        .unwrap_or("Ball in play");

    let is_down_active = data.down.map(|d| (1..=4).contains(&d)).unwrap_or(false);

    let mut header_spans = Vec::new();

    if is_down_active {
        header_spans.push(Span::styled(
            " DOWN: ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
        header_spans.push(Span::styled(
            format!(" 🏈 {} ", down_dist_str),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    } else {
        header_spans.push(Span::styled(
            " SITUATION: ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        header_spans.push(Span::styled(
            format!(" {} ", down_dist_str),
            Style::default()
                .fg(Color::White)
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        ));
    }

    let dir_arrow = if data.is_away_possession { "▶" } else { "◀" };
    if let Some(team) = &data.possession_team_abbrev {
        header_spans.push(Span::raw("   "));
        if is_down_active {
            header_spans.push(Span::styled(
                format!("{} driving {}", team, dir_arrow),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
        } else if down_dist_str == "Kickoff" {
            header_spans.push(Span::styled(
                format!("{} kicking {}", team, dir_arrow),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
        } else {
            header_spans.push(Span::styled(
                team.to_string(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            ));
        }
    }

    if data.is_red_zone {
        header_spans.push(Span::raw("   "));
        header_spans.push(Span::styled(
            " 🚨 RED ZONE ",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if is_down_active {
        if let Some(pos) = &data.possession_text {
            if !down_dist_str.contains(" at ") {
                header_spans.push(Span::styled(
                    format!("  (Spot: {})", pos),
                    Style::default().fg(Color::Gray),
                ));
            }
        }
    }

    let header_line = Line::from(header_spans);
    f.render_widget(Paragraph::new(header_line), chunks[0]);

    // 2. Football Field ASCII Visualization
    render_field_graphic(f, chunks[1], data);

    // 3. Drive summary
    let drive_text = if let Some(ds) = &data.drive_summary {
        ds.clone()
    } else {
        "Drive in progress".to_string()
    };
    let drive_p = Paragraph::new(Line::from(vec![
        Span::styled("Drive: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled(drive_text, Style::default().fg(Color::White)),
    ]));
    f.render_widget(drive_p, chunks[2]);

    // 4. Last Play Paragraph
    let last_play_str = data
        .last_play_text
        .as_deref()
        .unwrap_or("No recent play recorded.");

    let last_play_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Last Play ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    let last_play_p = Paragraph::new(last_play_str)
        .block(last_play_block)
        .wrap(Wrap { trim: true });

    f.render_widget(last_play_p, chunks[3]);
}

fn render_field_graphic(f: &mut Frame, area: Rect, data: &FieldData) {
    if area.width < 30 || area.height < 4 {
        let p = Paragraph::new("Terminal too small for field view")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(p, area);
        return;
    }

    let field_width = (area.width.saturating_sub(4)) as usize;
    if field_width < 24 {
        return;
    }

    // Endzone width accommodates team abbrev: e.g. "[ ORST ]" or "[ CSU ]"
    let max_abbrev_len = data.away_team_abbrev.len().max(data.home_team_abbrev.len());
    let ez_width = (max_abbrev_len + 4).max(field_width / 14).max(6);
    let playable_width = field_width.saturating_sub(ez_width * 2);

    // Ball yardline: 0 = Away goal line (Left), 100 = Home goal line (Right)
    let ball_yard = data.visual_yard_line.unwrap_or(50).clamp(0, 100);
    let ball_pos_in_playable = ((ball_yard as f32 / 100.0) * playable_width as f32).round() as usize;
    let ball_col = ez_width + ball_pos_in_playable.min(playable_width.saturating_sub(1));

    // First down line calculation (only when down is active)
    let is_down_active = data.down.map(|d| (1..=4).contains(&d)).unwrap_or(false);
    let first_down_col = if is_down_active {
        if let Some(dist) = data.distance {
            let fd_yard = if data.is_away_possession {
                // Driving right (towards Home goal line at 100)
                (ball_yard + dist).clamp(0, 100)
            } else {
                // Driving left (towards Away goal line at 0)
                (ball_yard - dist).clamp(0, 100)
            };
            let fd_pos = ((fd_yard as f32 / 100.0) * playable_width as f32).round() as usize;
            Some(ez_width + fd_pos.min(playable_width.saturating_sub(1)))
        } else {
            None
        }
    } else {
        None
    };

    // Header labels with team names on their respective endzones
    let yard_labels = format_yard_header(ez_width, playable_width, &data.away_team_abbrev, &data.home_team_abbrev);

    // Action row with ball and 1st down marker
    let mut action_spans = Vec::new();
    let left_ez_text = format!("[{:^width$}]", data.away_team_abbrev, width = ez_width.saturating_sub(2));
    action_spans.push(Span::styled(left_ez_text, Style::default().fg(Color::Cyan).bg(Color::DarkGray).add_modifier(Modifier::BOLD)));

    let ball_symbol = if data.is_away_possession { "▶" } else { "◀" };

    for col in ez_width..(field_width - ez_width) {
        if col == ball_col {
            action_spans.push(Span::styled(
                ball_symbol,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ));
        } else if Some(col) == first_down_col {
            action_spans.push(Span::styled(
                "┃",
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ));
        } else {
            let rel_col = col - ez_width;
            let is_major_line = rel_col % (playable_width / 10).max(1) == 0;
            if is_major_line {
                action_spans.push(Span::styled("|", Style::default().fg(Color::DarkGray)));
            } else if rel_col % 2 == 0 {
                action_spans.push(Span::styled("·", Style::default().fg(Color::Green)));
            } else {
                action_spans.push(Span::raw(" "));
            }
        }
    }

    let right_ez_text = format!("[{:^width$}]", data.home_team_abbrev, width = ez_width.saturating_sub(2));
    action_spans.push(Span::styled(right_ez_text, Style::default().fg(Color::LightBlue).bg(Color::DarkGray).add_modifier(Modifier::BOLD)));

    // Sideline row
    let mut sideline_str = String::with_capacity(field_width);
    sideline_str.push_str(&"=".repeat(ez_width));
    for col in 0..playable_width {
        if col % (playable_width / 10).max(1) == 0 {
            sideline_str.push('+');
        } else {
            sideline_str.push('-');
        }
    }
    sideline_str.push_str(&"=".repeat(ez_width));

    let field_title = if let Some(dd) = &data.down_distance_text {
        if is_down_active || (dd != "Pregame" && dd != "Final" && dd != "Ball in play") {
            format!(" Field Position — {} ", dd)
        } else {
            " Field Position (Left: Away, Right: Home) ".to_string()
        }
    } else {
        " Field Position (Left: Away, Right: Home) ".to_string()
    };

    let field_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(Span::styled(
            field_title,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ));

    let field_lines = vec![
        Line::from(yard_labels),
        Line::from(Span::styled(sideline_str.clone(), Style::default().fg(Color::DarkGray))),
        Line::from(action_spans),
        Line::from(Span::styled(sideline_str, Style::default().fg(Color::DarkGray))),
    ];

    let field_p = Paragraph::new(field_lines).block(field_block);
    f.render_widget(field_p, area);
}

fn format_yard_header(
    ez_width: usize,
    playable_width: usize,
    away_abbrev: &str,
    home_abbrev: &str,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let left_header = format!("{:^width$}", away_abbrev, width = ez_width);
    spans.push(Span::styled(
        left_header,
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ));

    // Markers: G, 10, 20, 30, 40, 50, 40, 30, 20, 10, G
    let step = playable_width as f32 / 10.0;
    let markers = ["G", "10", "20", "30", "40", "50", "40", "30", "20", "10", "G"];

    let mut header_chars = vec![' '; playable_width];
    for (i, &marker) in markers.iter().enumerate() {
        let center = (i as f32 * step).round() as usize;
        let start = center.saturating_sub(marker.len() / 2);
        for (ch_idx, ch) in marker.chars().enumerate() {
            if start + ch_idx < header_chars.len() {
                header_chars[start + ch_idx] = ch;
            }
        }
    }

    let header_str: String = header_chars.into_iter().collect();
    spans.push(Span::styled(
        header_str,
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
    ));

    let right_header = format!("{:^width$}", home_abbrev, width = ez_width);
    spans.push(Span::styled(
        right_header,
        Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
    ));

    spans
}
