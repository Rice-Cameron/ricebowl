use crate::app::App;
use crate::models::{Event, GameSummary, Play};
use crate::ui::field::{render_football_field, FieldData};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn render_gamecast(f: &mut Frame, area: Rect, app: &App, summary: &GameSummary, event: Option<&Event>) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Big Score Banner
            Constraint::Length(10), // Football Field & Down/Distance
            Constraint::Min(6),     // Scoring Summary & Drive Chart
        ])
        .split(area);

    // Extract field data first so score banner can also show live down/distance!
    let field_data = extract_field_data(summary, event);

    // 1. Big Score Banner
    render_score_banner(f, chunks[0], app, summary, event, &field_data);

    // 2. Football Field & Down/Distance
    render_football_field(f, chunks[1], &field_data);

    // 3. Lower Section: Scoring Plays & Drive Details
    let lower_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Scoring Plays
            Constraint::Percentage(50), // Drive History / Leaders
        ])
        .split(chunks[2]);

    render_scoring_plays(f, lower_chunks[0], summary);
    render_recent_drives(f, lower_chunks[1], summary);
}

fn render_score_banner(
    f: &mut Frame,
    area: Rect,
    app: &App,
    summary: &GameSummary,
    event: Option<&Event>,
    field_data: &FieldData,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Scoreboard ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    // Extract Away and Home teams from Header or Event
    let (away_team, away_score, away_fav, away_rank) = get_competitor_info(summary, event, "away", app);
    let (home_team, home_score, home_fav, home_rank) = get_competitor_info(summary, event, "home", app);

    // Status text (clock, quarter, final)
    let status_text = if let Some(hdr) = &summary.header {
        hdr.competitions
            .first()
            .and_then(|c| c.status.as_ref())
            .and_then(|s| s.status_type.as_ref())
            .and_then(|st| st.detail.as_deref())
            .unwrap_or("In Progress")
    } else if let Some(ev) = event {
        ev.status.status_type.detail.as_str()
    } else {
        "Live"
    };

    // Possession indicator
    let (away_has_ball, home_has_ball) = get_possession(summary, event);

    let away_ball_str = if away_has_ball { " 🏈" } else { "" };
    let home_ball_str = if home_has_ball { " 🏈" } else { "" };

    let away_fav_str = if away_fav { " ★" } else { "" };
    let home_fav_str = if home_fav { " ★" } else { "" };

    let away_rank_str = if away_rank > 0 && away_rank <= 25 {
        format!("#{} ", away_rank)
    } else {
        "".to_string()
    };
    let home_rank_str = if home_rank > 0 && home_rank <= 25 {
        format!("#{} ", home_rank)
    } else {
        "".to_string()
    };

    let line1 = Line::from(vec![
        Span::styled(away_rank_str, Style::default().fg(Color::Yellow)),
        Span::styled(
            away_team.clone(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
        Span::styled(away_fav_str, Style::default().fg(Color::Yellow)),
        Span::styled(away_ball_str, Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("  {}", away_score),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::raw("    vs    "),
        Span::styled(
            format!("{}  ", home_score),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        ),
        Span::styled(home_ball_str, Style::default().fg(Color::Yellow)),
        Span::styled(home_fav_str, Style::default().fg(Color::Yellow)),
        Span::styled(home_rank_str, Style::default().fg(Color::Yellow)),
        Span::styled(
            home_team,
            Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD),
        ),
    ]);

    let mut line2_spans = vec![
        Span::styled("Status: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            status_text,
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        ),
    ];

    let is_down_active = field_data.down.map(|d| (1..=4).contains(&d)).unwrap_or(false);

    if let Some(dd) = &field_data.down_distance_text {
        if dd != "Pregame" && dd != "Final" && dd != "Ball in play" {
            line2_spans.push(Span::raw("    "));
            let badge = if is_down_active {
                format!("🏈 {} ", dd)
            } else {
                format!("{} ", dd)
            };
            line2_spans.push(Span::styled(
                badge,
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ));
        }
    }

    if field_data.is_red_zone {
        line2_spans.push(Span::raw(" "));
        line2_spans.push(Span::styled(
            "[🚨 RED ZONE]",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
    }

    let line2 = Line::from(line2_spans);

    let dir_arrow = if field_data.is_away_possession { "▶" } else { "◀" };
    let mut line3_spans = Vec::new();
    if let Some(team) = &field_data.possession_team_abbrev {
        line3_spans.push(Span::styled("Possession: ", Style::default().fg(Color::DarkGray)));
        let poss_str = if is_down_active {
            format!("{} (driving {})", team, dir_arrow)
        } else {
            team.clone()
        };
        line3_spans.push(Span::styled(
            poss_str,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(ds) = &field_data.drive_summary {
        if !line3_spans.is_empty() {
            line3_spans.push(Span::styled("  •  ", Style::default().fg(Color::DarkGray)));
        }
        line3_spans.push(Span::styled("Drive: ", Style::default().fg(Color::DarkGray)));
        line3_spans.push(Span::styled(ds.clone(), Style::default().fg(Color::Gray)));
    }
    let line3 = Line::from(line3_spans);

    let p = Paragraph::new(vec![line1, line2, line3]);
    f.render_widget(p, inner);
}

fn get_competitor_info(
    summary: &GameSummary,
    event: Option<&Event>,
    side: &str,
    app: &App,
) -> (String, String, bool, i32) {
    if let Some(hdr) = &summary.header {
        if let Some(comp) = hdr.competitions.first() {
            if let Some(team) = comp.competitors.iter().find(|c| c.home_away.as_deref() == Some(side)) {
                let name = team
                    .team
                    .as_ref()
                    .and_then(|t| t.display_name.clone())
                    .unwrap_or_else(|| side.to_uppercase());
                let abbrev = team
                    .team
                    .as_ref()
                    .and_then(|t| t.abbreviation.clone())
                    .unwrap_or_default();
                let score = team.score.clone().unwrap_or_else(|| "0".to_string());
                let is_fav = app.config.is_favorite(&abbrev, team.id.as_deref().unwrap_or(""));
                let rank = team.rank.unwrap_or(99);
                return (name, score, is_fav, rank);
            }
        }
    }

    if let Some(ev) = event {
        if let Some(comp) = ev.competitions.first() {
            if let Some(team) = comp.competitors.iter().find(|c| c.home_away == side) {
                let name = team.team.display_name.clone();
                let abbrev = &team.team.abbreviation;
                let score = team.score.clone().unwrap_or_else(|| "0".to_string());
                let is_fav = app.config.is_favorite(abbrev, &team.team.id);
                let rank = team.curated_rank.as_ref().and_then(|r| r.current).unwrap_or(99);
                return (name, score, is_fav, rank);
            }
        }
    }

    (side.to_uppercase(), "0".to_string(), false, 99)
}

fn get_possession(summary: &GameSummary, event: Option<&Event>) -> (bool, bool) {
    let (away_id, home_id) = get_competitor_ids(summary, event);

    // 1. Try Header competitors possession flag if either is true
    if let Some(hdr) = &summary.header {
        if let Some(comp) = hdr.competitions.first() {
            let away_poss = comp
                .competitors
                .iter()
                .find(|c| c.home_away.as_deref() == Some("away"))
                .and_then(|c| c.possession)
                .unwrap_or(false);
            let home_poss = comp
                .competitors
                .iter()
                .find(|c| c.home_away.as_deref() == Some("home"))
                .and_then(|c| c.possession)
                .unwrap_or(false);
            if away_poss || home_poss {
                return (away_poss, home_poss);
            }
        }
    }

    // 2. Try Event situation possession ID from live scoreboard
    if let Some(ev) = event {
        if let Some(comp) = ev.competitions.first() {
            if let Some(sit) = &comp.situation {
                if let Some(poss_id) = &sit.possession {
                    if Some(poss_id.as_str()) == away_id.as_deref() {
                        return (true, false);
                    } else if Some(poss_id.as_str()) == home_id.as_deref() {
                        return (false, true);
                    }
                }
            }
        }
    }

    // 3. Try Current drive team ID (only if drive is still active)
    if let Some(curr) = summary.drives.as_ref().and_then(|d| d.current.as_ref()) {
        if curr.result.is_none() && curr.is_score != Some(true) {
            if let Some(drive_team_id) = curr.team.as_ref().and_then(|t| t.id.as_deref()) {
                if Some(drive_team_id) == away_id.as_deref() {
                    return (true, false);
                } else if Some(drive_team_id) == home_id.as_deref() {
                    return (false, true);
                }
            }
        }
    }

    // 4. Try current drive team ID even if completed (e.g. kicking team after score)
    if let Some(curr) = summary.drives.as_ref().and_then(|d| d.current.as_ref()) {
        if let Some(drive_team_id) = curr.team.as_ref().and_then(|t| t.id.as_deref()) {
            if Some(drive_team_id) == away_id.as_deref() {
                return (true, false);
            } else if Some(drive_team_id) == home_id.as_deref() {
                return (false, true);
            }
        }
    }

    (false, false)
}

fn get_competitor_ids(summary: &GameSummary, event: Option<&Event>) -> (Option<String>, Option<String>) {
    if let Some(hdr) = &summary.header {
        if let Some(comp) = hdr.competitions.first() {
            let away = comp
                .competitors
                .iter()
                .find(|c| c.home_away.as_deref() == Some("away"))
                .and_then(|c| c.id.clone());
            let home = comp
                .competitors
                .iter()
                .find(|c| c.home_away.as_deref() == Some("home"))
                .and_then(|c| c.id.clone());
            if away.is_some() || home.is_some() {
                return (away, home);
            }
        }
    }

    if let Some(ev) = event {
        if let Some(comp) = ev.competitions.first() {
            let away = comp
                .competitors
                .iter()
                .find(|c| c.home_away == "away")
                .map(|c| c.id.clone());
            let home = comp
                .competitors
                .iter()
                .find(|c| c.home_away == "home")
                .map(|c| c.id.clone());
            return (away, home);
        }
    }

    (None, None)
}

fn get_team_abbrevs(summary: &GameSummary, event: Option<&Event>) -> (String, String) {
    if let Some(hdr) = &summary.header {
        if let Some(comp) = hdr.competitions.first() {
            let away = comp
                .competitors
                .iter()
                .find(|c| c.home_away.as_deref() == Some("away"))
                .and_then(|c| c.team.as_ref())
                .and_then(|t| t.abbreviation.clone());
            let home = comp
                .competitors
                .iter()
                .find(|c| c.home_away.as_deref() == Some("home"))
                .and_then(|c| c.team.as_ref())
                .and_then(|t| t.abbreviation.clone());
            if let (Some(a), Some(h)) = (away, home) {
                return (a, h);
            }
        }
    }

    if let Some(ev) = event {
        if let Some(comp) = ev.competitions.first() {
            let away = comp
                .competitors
                .iter()
                .find(|c| c.home_away == "away")
                .map(|c| c.team.abbreviation.clone());
            let home = comp
                .competitors
                .iter()
                .find(|c| c.home_away == "home")
                .map(|c| c.team.abbreviation.clone());
            if let (Some(a), Some(h)) = (away, home) {
                return (a, h);
            }
        }
    }

    ("AWAY".to_string(), "HOME".to_string())
}

fn extract_field_data(summary: &GameSummary, event: Option<&Event>) -> FieldData {
    let (away_abbrev, home_abbrev) = get_team_abbrevs(summary, event);
    let (away_has_ball, home_has_ball) = get_possession(summary, event);
    let is_away_possession = if away_has_ball {
        true
    } else if home_has_ball {
        false
    } else {
        true
    };

    let possession_team_abbrev = if away_has_ball {
        Some(away_abbrev.clone())
    } else if home_has_ball {
        Some(home_abbrev.clone())
    } else if let Some(curr) = summary.drives.as_ref().and_then(|d| d.current.as_ref()) {
        curr.team.as_ref().and_then(|t| t.abbreviation.clone())
    } else {
        None
    };

    let poss_team_for_formatting = possession_team_abbrev.as_deref().or(if is_away_possession {
        Some(away_abbrev.as_str())
    } else {
        Some(home_abbrev.as_str())
    });

    let current_drive = summary.drives.as_ref().and_then(|d| d.current.as_ref());
    let last_play = current_drive
        .and_then(|d| d.plays.last())
        .or_else(|| {
            summary
                .drives
                .as_ref()
                .and_then(|d| d.previous.last())
                .and_then(|d| d.plays.last())
        });

    let (down, distance, raw_espn_yard, down_dist_text, poss_text, is_red_zone_sit) =
        resolve_situation(summary, event, poss_team_for_formatting, last_play);

    // CONVERT ESPN YARDLINE TO VISUAL FIELD POSITION:
    // In ESPN: 0 = Home goal line, 100 = Away goal line
    // On our visual field: 0 = Away (Left), 100 = Home (Right)
    // visual_yard_line = 100 - raw_espn_yard
    let visual_yard_line = raw_espn_yard.map(|y| 100 - y.clamp(0, 100));

    // Red zone check:
    // Only active when an offensive down is in play!
    let is_down_active = down.map(|d| (1..=4).contains(&d)).unwrap_or(false);
    let is_red_zone = is_down_active && (is_red_zone_sit || match (is_away_possession, visual_yard_line) {
        (true, Some(y)) => y >= 80,
        (false, Some(y)) => y <= 20,
        _ => false,
    });

    let drive_summary = current_drive.and_then(|d| d.description.clone()).or_else(|| {
        summary
            .drives
            .as_ref()
            .and_then(|d| d.previous.last())
            .and_then(|d| d.description.clone())
    });

    let last_play_text = last_play
        .and_then(|p| p.text.clone())
        .or_else(|| {
            event
                .and_then(|ev| ev.competitions.first())
                .and_then(|c| c.situation.as_ref())
                .and_then(|s| s.last_play.as_ref())
                .and_then(|lp| lp.text.clone())
        });

    FieldData {
        down,
        distance,
        visual_yard_line,
        is_red_zone,
        down_distance_text: down_dist_text,
        possession_text: poss_text,
        away_team_abbrev: away_abbrev,
        home_team_abbrev: home_abbrev,
        possession_team_abbrev,
        is_away_possession,
        drive_summary,
        last_play_text,
    }
}

fn is_timeout_or_stoppage(play: &Play) -> bool {
    if let Some(pt) = &play.play_type {
        if let Some(txt) = &pt.text {
            let t = txt.to_lowercase();
            if t.contains("timeout") {
                return true;
            }
        }
    }
    if let Some(txt) = &play.text {
        let t = txt.to_lowercase();
        if t.starts_with("timeout") || t.contains("timeout ") {
            return true;
        }
    }
    false
}

fn resolve_situation(
    summary: &GameSummary,
    event: Option<&Event>,
    possession_team: Option<&str>,
    last_play: Option<&Play>,
) -> (Option<i32>, Option<i32>, Option<i32>, Option<String>, Option<String>, bool) {
    let ev_sit = event
        .and_then(|ev| ev.competitions.first())
        .and_then(|c| c.situation.as_ref());

    // 1. Check game status (completed, pregame, halftime, end of period)
    if let Some(ev) = event {
        if ev.status.status_type.completed || ev.status.status_type.state == "post" {
            return (None, None, None, Some("Final".to_string()), None, false);
        }
        if ev.status.status_type.state == "pre" {
            return (None, None, None, Some("Pregame".to_string()), None, false);
        }
        let clock = ev.status.display_clock.as_deref().unwrap_or("");
        let period = ev.status.period.unwrap_or(0);
        let detail = ev.status.status_type.detail.to_lowercase();
        if detail.contains("halftime") || (period == 2 && clock == "0:00") {
            return (None, None, None, Some("Halftime".to_string()), None, false);
        } else if period == 1 && clock == "0:00" {
            return (None, None, None, Some("End of 1st".to_string()), None, false);
        } else if period == 3 && clock == "0:00" {
            return (None, None, None, Some("End of 3rd".to_string()), None, false);
        }
    }

    let mut down = None;
    let mut distance = None;
    let mut raw_espn_yard = None;
    let mut down_dist_text = None;
    let mut poss_text = None;
    let mut is_red_zone = false;

    // 2. Check event.situation from live scoreboard
    if let Some(sit) = ev_sit {
        if let Some(rz) = sit.is_red_zone {
            is_red_zone = rz;
        }
        if let Some(d) = sit.down {
            if (1..=4).contains(&d) {
                down = Some(d);
                distance = sit.distance;
                raw_espn_yard = sit.yard_line;
                down_dist_text = sit.down_distance_text.clone();
                poss_text = sit.possession_text.clone();
            } else {
                // Non-down situation from scoreboard (e.g. -1 after a score or kickoff)
                if sit.yard_line.is_some() {
                    raw_espn_yard = sit.yard_line;
                }
                if sit.possession_text.is_some() {
                    poss_text = sit.possession_text.clone();
                }
            }
        } else {
            if let Some(dd) = &sit.down_distance_text {
                down_dist_text = Some(dd.clone());
            }
            if sit.yard_line.is_some() {
                raw_espn_yard = sit.yard_line;
            }
            if sit.possession_text.is_some() {
                poss_text = sit.possession_text.clone();
            }
        }
    }

    let curr_drive = summary.drives.as_ref().and_then(|d| d.current.as_ref());

    // 3. Detect if a score occurred, or drive finished, or non-down
    let is_espn_non_down = ev_sit.and_then(|s| s.down).map(|d| d < 0).unwrap_or(false);

    let is_scoring_drive = curr_drive.map(|d| {
        d.is_score == Some(true)
            || d.result.as_deref() == Some("TD")
            || d.result.as_deref() == Some("FG")
            || d.display_result.as_deref() == Some("Touchdown")
            || d.display_result.as_deref() == Some("Field Goal")
    }).unwrap_or(false);

    let is_scoring_play = last_play.map(|p| p.scoring_play == Some(true)).unwrap_or(false)
        || ev_sit.and_then(|s| s.last_play.as_ref()).map(|lp| {
            lp.play_type.as_ref().and_then(|t| t.text.as_deref()).map(|txt| {
                txt.contains("Touchdown")
                    || txt.contains("Field Goal")
                    || txt.contains("Extra Point")
                    || txt.contains("Safety")
            }).unwrap_or(false)
            || lp.text.as_deref().map(|txt| {
                txt.contains("TOUCHDOWN")
                    || txt.contains("KICK")
                    || txt.contains("kick attempt")
                    || txt.contains("field goal")
            }).unwrap_or(false)
        }).unwrap_or(false);

    let drive_completed_result = curr_drive.and_then(|d| d.result.as_deref());

    if is_espn_non_down || is_scoring_drive || is_scoring_play || drive_completed_result.is_some() {
        // Clear down, distance, and red zone because play is not an active down
        down = None;
        distance = None;
        is_red_zone = false;

        if is_scoring_drive || is_scoring_play {
            let is_td = curr_drive.and_then(|d| d.result.as_deref()) == Some("TD")
                || curr_drive.and_then(|d| d.display_result.as_deref()) == Some("Touchdown")
                || last_play.map(|p| {
                    p.play_type.as_ref().and_then(|t| t.text.as_deref()).map(|txt| txt.contains("Touchdown")).unwrap_or(false)
                        || p.text.as_deref().map(|txt| txt.contains("TOUCHDOWN")).unwrap_or(false)
                }).unwrap_or(false)
                || ev_sit.and_then(|s| s.last_play.as_ref()).map(|lp| {
                    lp.play_type.as_ref().and_then(|t| t.text.as_deref()).map(|txt| txt.contains("Touchdown")).unwrap_or(false)
                        || lp.text.as_deref().map(|txt| txt.contains("TOUCHDOWN")).unwrap_or(false)
                }).unwrap_or(false);

            let is_fg = curr_drive.and_then(|d| d.result.as_deref()) == Some("FG")
                || curr_drive.and_then(|d| d.display_result.as_deref()) == Some("Field Goal")
                || last_play.map(|p| {
                    p.play_type.as_ref().and_then(|t| t.text.as_deref()).map(|txt| txt.contains("Field Goal")).unwrap_or(false)
                        || p.text.as_deref().map(|txt| txt.contains("field goal")).unwrap_or(false)
                }).unwrap_or(false)
                || ev_sit.and_then(|s| s.last_play.as_ref()).map(|lp| {
                    lp.play_type.as_ref().and_then(|t| t.text.as_deref()).map(|txt| txt.contains("Field Goal")).unwrap_or(false)
                        || lp.text.as_deref().map(|txt| txt.contains("field goal")).unwrap_or(false)
                }).unwrap_or(false);

            let is_safety = curr_drive.and_then(|d| d.result.as_deref()) == Some("SAFETY")
                || last_play.map(|p| {
                    p.play_type.as_ref().and_then(|t| t.text.as_deref()).map(|txt| txt.contains("Safety")).unwrap_or(false)
                        || p.text.as_deref().map(|txt| txt.contains("Safety")).unwrap_or(false)
                }).unwrap_or(false);

            if is_td {
                down_dist_text = Some("Touchdown".to_string());
            } else if is_fg {
                down_dist_text = Some("Field Goal".to_string());
            } else if is_safety {
                down_dist_text = Some("Safety".to_string());
            } else {
                down_dist_text = Some("Touchdown".to_string());
            }
        } else if let Some(res) = drive_completed_result {
            let res_upper = res.to_uppercase();
            let label = match res_upper.as_str() {
                "PUNT" => "Punt",
                "DOWNS" => "Turnover on Downs",
                "FUMBLE" | "INT" | "INTERCEPTION" => "Turnover",
                "MISSED FG" => "Missed FG",
                "END OF HALF" => "End of Half",
                "END OF GAME" => "Final",
                _ => res,
            };
            down_dist_text = Some(label.to_string());
        } else if is_espn_non_down {
            let lp_type = ev_sit.and_then(|s| s.last_play.as_ref())
                .and_then(|lp| lp.play_type.as_ref())
                .and_then(|t| t.text.as_deref())
                .unwrap_or("");
            let lp_text = ev_sit.and_then(|s| s.last_play.as_ref())
                .and_then(|lp| lp.text.as_deref())
                .unwrap_or("");
            if lp_type.contains("Kickoff") || lp_text.contains("kickoff") {
                down_dist_text = Some("Kickoff".to_string());
            } else if lp_type.contains("Punt") || lp_text.contains("punt") {
                down_dist_text = Some("Punt".to_string());
            } else if lp_type.contains("Extra Point") || lp_type.contains("XP") || lp_text.contains("KICK") || lp_text.contains("kick attempt") {
                down_dist_text = Some("Touchdown".to_string());
            }
        }
    } else if down.unwrap_or(0) <= 0 {
        // 4. Drive is active, but down is not yet set (e.g. during a Timeout or stoppage).
        if let Some(curr) = curr_drive {
            if curr.result.is_none() && curr.is_score != Some(true) {
                for play in curr.plays.iter().rev() {
                    let is_timeout = is_timeout_or_stoppage(play);
                    if is_timeout {
                        if let Some(start) = &play.start {
                            if let Some(d) = start.down {
                                if (1..=4).contains(&d) {
                                    down = Some(d);
                                    distance = start.distance;
                                    if raw_espn_yard.is_none() { raw_espn_yard = start.yard_line; }
                                    if down_dist_text.is_none() { down_dist_text = start.down_distance_text.clone(); }
                                    if poss_text.is_none() { poss_text = start.possession_text.clone(); }
                                    break;
                                }
                            }
                        }
                    } else {
                        if let Some(end) = &play.end {
                            if let Some(d) = end.down {
                                if (1..=4).contains(&d) {
                                    down = Some(d);
                                    distance = end.distance;
                                    if raw_espn_yard.is_none() { raw_espn_yard = end.yard_line; }
                                    if down_dist_text.is_none() { down_dist_text = end.down_distance_text.clone(); }
                                    if poss_text.is_none() { poss_text = end.possession_text.clone(); }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 5. Fill missing yard line / possession text from last play only if still missing
    if raw_espn_yard.is_none() {
        if let Some(play) = last_play {
            raw_espn_yard = play.end.as_ref().and_then(|e| e.yard_line)
                .or_else(|| play.start.as_ref().and_then(|s| s.yard_line));
        }
    }
    if poss_text.is_none() && down.is_some() {
        if let Some(play) = last_play {
            poss_text = play.end.as_ref().and_then(|e| e.possession_text.clone())
                .or_else(|| play.start.as_ref().and_then(|s| s.possession_text.clone()));
        }
    }

    // 6. Format the final down & distance string
    let final_down_dist_text = format_down_distance_spot(
        down,
        distance,
        poss_text.as_deref(),
        possession_team,
        raw_espn_yard,
        down_dist_text.as_deref(),
        last_play,
        event,
    );

    (down, distance, raw_espn_yard, Some(final_down_dist_text), poss_text, is_red_zone)
}

fn format_down_distance_spot(
    down: Option<i32>,
    distance: Option<i32>,
    poss_text: Option<&str>,
    possession_team: Option<&str>,
    yard_line: Option<i32>,
    down_dist_text: Option<&str>,
    last_play: Option<&Play>,
    event: Option<&Event>,
) -> String {
    // 1. If down is between 1 and 4 (active down)
    if let Some(d) = down {
        if (1..=4).contains(&d) {
            if let Some(dd_text) = down_dist_text {
                let trimmed = dd_text.trim();
                if !trimmed.is_empty() {
                    if trimmed.contains(" at ") {
                        return trimmed.to_string();
                    } else if let Some(spot) = poss_text {
                        return format!("{} at {}", trimmed, spot);
                    } else if let (Some(team), Some(yd)) = (possession_team, yard_line) {
                        return format!("{} at {} {}", trimmed, team, yd);
                    } else {
                        return trimmed.to_string();
                    }
                }
            }

            let d_str = match d {
                1 => "1st",
                2 => "2nd",
                3 => "3rd",
                4 => "4th",
                _ => "Down",
            };
            let dist_str = match distance {
                Some(0) => "Goal".to_string(),
                Some(dist) if dist > 0 => dist.to_string(),
                _ => "Goal".to_string(),
            };
            let base = format!("{} & {}", d_str, dist_str);
            if let Some(spot) = poss_text {
                return format!("{} at {}", base, spot);
            } else if let (Some(team), Some(yd)) = (possession_team, yard_line) {
                return format!("{} at {} {}", base, team, yd);
            } else {
                return base;
            }
        }
    }

    // 2. Non-down situations: If we already have a situation string (e.g. "Touchdown", "Field Goal", "Final", "Pregame", "Halftime", "Punt")
    if let Some(dd_text) = down_dist_text {
        let trimmed = dd_text.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // 3. Status checks for non-down situations
    if let Some(ev) = event {
        if ev.status.status_type.completed || ev.status.status_type.state == "post" {
            return "Final".to_string();
        }
        if ev.status.status_type.state == "pre" {
            return "Pregame".to_string();
        }
    }

    if let Some(play) = last_play {
        if let Some(pt) = &play.play_type {
            if let Some(txt) = &pt.text {
                let t = txt.to_lowercase();
                if t.contains("touchdown") {
                    return "Touchdown".to_string();
                } else if t.contains("field goal") {
                    return "Field Goal".to_string();
                } else if t.contains("extra point") || t.contains("xp") {
                    return "Extra Point".to_string();
                } else if t.contains("safety") {
                    return "Safety".to_string();
                } else if t.contains("kickoff") {
                    return "Kickoff".to_string();
                } else if t.contains("punt") {
                    return "Punt".to_string();
                }
            }
        }
        if play.scoring_play == Some(true) {
            return "Touchdown".to_string();
        }
    }

    if let Some(spot) = poss_text {
        return format!("Ball at {}", spot);
    }

    "Ball in play".to_string()
}

fn render_scoring_plays(f: &mut Frame, area: Rect, summary: &GameSummary) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Scoring Summary ",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    if summary.scoring_plays.is_empty() {
        let p = Paragraph::new("No scoring plays yet.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = summary
        .scoring_plays
        .iter()
        .map(|play| {
            let qtr = play
                .period
                .as_ref()
                .and_then(|p| p.number)
                .map(|n| format!("Q{}", n))
                .unwrap_or_else(|| "OT".to_string());
            let clock = play
                .clock
                .as_ref()
                .and_then(|c| c.display_value.clone())
                .unwrap_or_default();
            let team_abbrev = play
                .team
                .as_ref()
                .and_then(|t| t.abbreviation.clone())
                .unwrap_or_else(|| "---".to_string());
            let away_s = play.away_score.unwrap_or(0);
            let home_s = play.home_score.unwrap_or(0);
            let desc = play.text.as_deref().unwrap_or("");

            let line = Line::from(vec![
                Span::styled(format!("[{} {}] ", qtr, clock), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{:<4} ", team_abbrev), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(format!("({}-{}) ", away_s, home_s), Style::default().fg(Color::Gray)),
                Span::styled(desc, Style::default().fg(Color::White)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

fn render_recent_drives(f: &mut Frame, area: Rect, summary: &GameSummary) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Drive Chart ",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));

    let drives = summary.drives.as_ref();
    if drives.is_none() || drives.unwrap().previous.is_empty() {
        let p = Paragraph::new("No drive history yet.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block);
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = drives
        .unwrap()
        .previous
        .iter()
        .rev()
        .take(15)
        .map(|drive| {
            let team = drive
                .team
                .as_ref()
                .and_then(|t| t.abbreviation.clone())
                .unwrap_or_else(|| "---".to_string());
            let res = drive.display_result.as_deref().or(drive.result.as_deref()).unwrap_or("---");
            let desc = drive.description.as_deref().unwrap_or("");

            let res_color = match res.to_uppercase().as_str() {
                "TOUCHDOWN" | "TD" => Color::Yellow,
                "FIELD GOAL" | "FG" => Color::Green,
                "PUNT" => Color::DarkGray,
                "FUMBLE" | "INTERCEPTION" | "DOWNS" => Color::Red,
                _ => Color::White,
            };

            let line = Line::from(vec![
                Span::styled(format!("{:<4} ", team), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:<12} ", res), Style::default().fg(res_color).add_modifier(Modifier::BOLD)),
                Span::styled(desc, Style::default().fg(Color::Gray)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Drives, Drive, PlayPosition};

    #[test]
    fn test_format_down_distance_complete_text() {
        let res = format_down_distance_spot(
            Some(4),
            Some(4),
            Some("CSU 48"),
            Some("CSU"),
            Some(48),
            Some("4th & 4 at CSU 48"),
            None,
            None,
        );
        assert_eq!(res, "4th & 4 at CSU 48");
    }

    #[test]
    fn test_format_down_distance_short_text_with_possession() {
        let res = format_down_distance_spot(
            Some(4),
            Some(4),
            Some("CSU 48"),
            Some("CSU"),
            Some(48),
            Some("4th & 4"),
            None,
            None,
        );
        assert_eq!(res, "4th & 4 at CSU 48");
    }

    #[test]
    fn test_format_down_distance_synthesized() {
        let res = format_down_distance_spot(
            Some(4),
            Some(4),
            Some("CSU 48"),
            Some("CSU"),
            Some(48),
            None,
            None,
            None,
        );
        assert_eq!(res, "4th & 4 at CSU 48");
    }

    #[test]
    fn test_format_down_distance_goal_situation() {
        let res = format_down_distance_spot(
            Some(2),
            Some(1),
            Some("GASO 1"),
            Some("GASO"),
            Some(99),
            Some("2nd & Goal at GASO 1"),
            None,
            None,
        );
        assert_eq!(res, "2nd & Goal at GASO 1");
    }

    #[test]
    fn test_resolve_situation_timeout_play_fallback_to_start() {
        // Simulates the Colorado State situation where the last play was a timeout:
        // end.down = 0, but start.down = 4, start.distance = 4, start.down_distance_text = "4th & 4 at CSU 48"
        let timeout_play = Play {
            id: Some("123".to_string()),
            sequence_number: None,
            play_type: None,
            text: Some("Timeout Colorado State, clock 01:52".to_string()),
            away_score: None,
            home_score: None,
            period: None,
            clock: None,
            scoring_play: None,
            priority: None,
            stat_yardage: None,
            start: Some(PlayPosition {
                down: Some(4),
                distance: Some(4),
                yard_line: Some(48),
                yards_to_endzone: Some(52),
                down_distance_text: Some("4th & 4 at CSU 48".to_string()),
                short_down_distance_text: Some("4th & 4".to_string()),
                possession_text: Some("CSU 48".to_string()),
                team: None,
            }),
            end: Some(PlayPosition {
                down: Some(0),
                distance: Some(4),
                yard_line: Some(48),
                yards_to_endzone: Some(52),
                down_distance_text: None,
                short_down_distance_text: None,
                possession_text: None,
                team: None,
            }),
            is_penalty: None,
            is_turnover: None,
        };

        let summary = GameSummary {
            boxscore: None,
            game_info: None,
            drives: Some(Drives {
                current: Some(Drive {
                    id: Some("drive1".to_string()),
                    description: Some("4 plays, 23 yards, 1:19".to_string()),
                    team: None,
                    start: None,
                    end: None,
                    time_elapsed: None,
                    yards: None,
                    is_score: None,
                    offensive_plays: None,
                    result: None,
                    display_result: None,
                    plays: vec![timeout_play],
                }),
                previous: vec![],
            }),
            scoring_plays: vec![],
            header: None,
            win_probability: vec![],
        };

        let (down, distance, yard_line, down_dist_text, poss_text, _) =
            resolve_situation(&summary, None, Some("CSU"), None);

        assert_eq!(down, Some(4));
        assert_eq!(distance, Some(4));
        assert_eq!(yard_line, Some(48));
        assert_eq!(down_dist_text, Some("4th & 4 at CSU 48".to_string()));
        assert_eq!(poss_text, Some("CSU 48".to_string()));
    }

    #[test]
    fn test_resolve_situation_touchdown_scoring_drive_no_stale_down() {
        // Simulates the Oregon State game situation:
        // Drive ended with a touchdown pass that started on 2nd & 4 at CSU 8.
        // Event situation down is -1, yard_line is 65 (kickoff).
        // It must NOT show "2nd & 4 at CSU 8"!
        let td_play = Play {
            id: Some("play7".to_string()),
            sequence_number: None,
            play_type: Some(crate::models::PlayType {
                id: Some("67".to_string()),
                text: Some("Passing Touchdown".to_string()),
                abbreviation: Some("TD".to_string()),
            }),
            text: Some("(07:31) B.Atkinson pass complete to E.Olsen for 8 yards to CSU00 TOUCHDOWN".to_string()),
            away_score: Some(21),
            home_score: Some(14),
            period: None,
            clock: None,
            scoring_play: Some(true),
            priority: None,
            stat_yardage: None,
            start: Some(PlayPosition {
                down: Some(2),
                distance: Some(4),
                yard_line: Some(8),
                yards_to_endzone: Some(8),
                down_distance_text: Some("2nd & 4 at CSU 8".to_string()),
                short_down_distance_text: Some("2nd & 4".to_string()),
                possession_text: Some("CSU 8".to_string()),
                team: None,
            }),
            end: Some(PlayPosition {
                down: Some(-1),
                distance: Some(0),
                yard_line: Some(0),
                yards_to_endzone: Some(0),
                down_distance_text: None,
                short_down_distance_text: None,
                possession_text: None,
                team: None,
            }),
            is_penalty: None,
            is_turnover: None,
        };

        let summary = GameSummary {
            boxscore: None,
            game_info: None,
            drives: Some(Drives {
                current: Some(Drive {
                    id: Some("drive1".to_string()),
                    description: Some("8 plays, 48 yards, 3:29".to_string()),
                    team: None,
                    start: None,
                    end: None,
                    time_elapsed: None,
                    yards: None,
                    is_score: Some(true),
                    offensive_plays: None,
                    result: Some("TD".to_string()),
                    display_result: Some("Touchdown".to_string()),
                    plays: vec![td_play.clone()],
                }),
                previous: vec![],
            }),
            scoring_plays: vec![],
            header: None,
            win_probability: vec![],
        };

        let event = Event {
            id: "401860899".to_string(),
            date: "2026-10-03T17:00:00Z".to_string(),
            name: "Oregon State Beavers at Colorado State Rams".to_string(),
            short_name: "ORST @ CSU".to_string(),
            status: crate::models::scoreboard::EventStatus {
                display_clock: Some("7:26".to_string()),
                period: Some(3),
                status_type: crate::models::scoreboard::StatusType {
                    id: "2".to_string(),
                    name: "STATUS_IN_PROGRESS".to_string(),
                    state: "in".to_string(),
                    completed: false,
                    description: "In Progress".to_string(),
                    detail: "7:26 - 3rd Quarter".to_string(),
                    short_detail: "7:26 - 3rd".to_string(),
                },
            },
            competitions: vec![crate::models::Competition {
                id: "401860899".to_string(),
                date: "2026-10-03T17:00:00Z".to_string(),
                competitors: vec![],
                situation: Some(crate::models::Situation {
                    down: Some(-1),
                    distance: Some(-1),
                    yard_line: Some(65),
                    is_red_zone: Some(false),
                    possession: None,
                    down_distance_text: None,
                    possession_text: None,
                    last_play: Some(crate::models::LastPlay {
                        id: Some("-89216018".to_string()),
                        text: Some("(C. Ojeda KICK)".to_string()),
                        stat_yardage: Some(0),
                        play_type: Some(crate::models::scoreboard::LastPlayType {
                            id: Some("61".to_string()),
                            text: Some("Extra Point Good".to_string()),
                            abbreviation: Some("XP".to_string()),
                        }),
                    }),
                }),
                broadcasts: vec![],
                venue: None,
            }],
        };

        let (down, distance, yard_line, down_dist_text, _, is_red_zone) =
            resolve_situation(&summary, Some(&event), Some("ORST"), Some(&td_play));

        assert_eq!(down, None, "Down should be None after a touchdown!");
        assert_eq!(distance, None, "Distance should be None after a touchdown!");
        assert_eq!(yard_line, Some(65), "Yardline should reflect kickoff spot!");
        assert_eq!(down_dist_text, Some("Touchdown".to_string()), "Text should be 'Touchdown'!");
        assert_eq!(is_red_zone, false, "Red zone should be false after touchdown!");
    }

    #[test]
    fn test_resolve_situation_field_goal_drive() {
        let summary = GameSummary {
            boxscore: None,
            game_info: None,
            drives: Some(Drives {
                current: Some(Drive {
                    id: Some("drive1".to_string()),
                    description: Some("10 plays, 55 yards, 4:12".to_string()),
                    team: None,
                    start: None,
                    end: None,
                    time_elapsed: None,
                    yards: None,
                    is_score: Some(true),
                    offensive_plays: None,
                    result: Some("FG".to_string()),
                    display_result: Some("Field Goal".to_string()),
                    plays: vec![],
                }),
                previous: vec![],
            }),
            scoring_plays: vec![],
            header: None,
            win_probability: vec![],
        };

        let (down, distance, _, down_dist_text, _, is_red_zone) =
            resolve_situation(&summary, None, Some("ORST"), None);

        assert_eq!(down, None);
        assert_eq!(distance, None);
        assert_eq!(down_dist_text, Some("Field Goal".to_string()));
        assert_eq!(is_red_zone, false);
    }

    #[test]
    fn test_resolve_situation_normal_play_in_active_drive_uses_end_down() {
        // In an active drive, Play 1 ended on 2nd & 6 at CSU 20.
        // It must use end.down (2), NOT start.down (1).
        let normal_play = Play {
            id: Some("play1".to_string()),
            sequence_number: None,
            play_type: Some(crate::models::PlayType {
                id: Some("5".to_string()),
                text: Some("Rush".to_string()),
                abbreviation: Some("RUSH".to_string()),
            }),
            text: Some("Rush for 4 yards".to_string()),
            away_score: None,
            home_score: None,
            period: None,
            clock: None,
            scoring_play: Some(false),
            priority: None,
            stat_yardage: None,
            start: Some(PlayPosition {
                down: Some(1),
                distance: Some(10),
                yard_line: Some(24),
                yards_to_endzone: Some(24),
                down_distance_text: Some("1st & 10 at CSU 24".to_string()),
                short_down_distance_text: Some("1st & 10".to_string()),
                possession_text: Some("CSU 24".to_string()),
                team: None,
            }),
            end: Some(PlayPosition {
                down: Some(2),
                distance: Some(6),
                yard_line: Some(20),
                yards_to_endzone: Some(20),
                down_distance_text: Some("2nd & 6 at CSU 20".to_string()),
                short_down_distance_text: Some("2nd & 6".to_string()),
                possession_text: Some("CSU 20".to_string()),
                team: None,
            }),
            is_penalty: None,
            is_turnover: None,
        };

        let summary = GameSummary {
            boxscore: None,
            game_info: None,
            drives: Some(Drives {
                current: Some(Drive {
                    id: Some("drive1".to_string()),
                    description: Some("1 play, 4 yards".to_string()),
                    team: None,
                    start: None,
                    end: None,
                    time_elapsed: None,
                    yards: None,
                    is_score: None,
                    offensive_plays: None,
                    result: None,
                    display_result: None,
                    plays: vec![normal_play],
                }),
                previous: vec![],
            }),
            scoring_plays: vec![],
            header: None,
            win_probability: vec![],
        };

        let (down, distance, yard_line, down_dist_text, poss_text, _) =
            resolve_situation(&summary, None, Some("ORST"), None);

        assert_eq!(down, Some(2));
        assert_eq!(distance, Some(6));
        assert_eq!(yard_line, Some(20));
        assert_eq!(down_dist_text, Some("2nd & 6 at CSU 20".to_string()));
        assert_eq!(poss_text, Some("CSU 20".to_string()));
    }

    #[test]
    fn test_extract_field_data_touchdown_end_to_end() {
        let td_play = Play {
            id: Some("play7".to_string()),
            sequence_number: None,
            play_type: Some(crate::models::PlayType {
                id: Some("67".to_string()),
                text: Some("Passing Touchdown".to_string()),
                abbreviation: Some("TD".to_string()),
            }),
            text: Some("(07:31) B.Atkinson pass complete to E.Olsen for 8 yards to CSU00 TOUCHDOWN".to_string()),
            away_score: Some(21),
            home_score: Some(14),
            period: None,
            clock: None,
            scoring_play: Some(true),
            priority: None,
            stat_yardage: None,
            start: Some(PlayPosition {
                down: Some(2),
                distance: Some(4),
                yard_line: Some(8),
                yards_to_endzone: Some(8),
                down_distance_text: Some("2nd & 4 at CSU 8".to_string()),
                short_down_distance_text: Some("2nd & 4".to_string()),
                possession_text: Some("CSU 8".to_string()),
                team: None,
            }),
            end: Some(PlayPosition {
                down: Some(-1),
                distance: Some(0),
                yard_line: Some(0),
                yards_to_endzone: Some(0),
                down_distance_text: None,
                short_down_distance_text: None,
                possession_text: None,
                team: None,
            }),
            is_penalty: None,
            is_turnover: None,
        };

        let summary = GameSummary {
            boxscore: None,
            game_info: None,
            drives: Some(Drives {
                current: Some(Drive {
                    id: Some("drive1".to_string()),
                    description: Some("8 plays, 48 yards, 3:29".to_string()),
                    team: Some(crate::models::summary::DriveTeam {
                        id: Some("204".to_string()),
                        display_name: Some("Oregon State Beavers".to_string()),
                        abbreviation: Some("ORST".to_string()),
                        logo: None,
                    }),
                    start: None,
                    end: None,
                    time_elapsed: None,
                    yards: None,
                    is_score: Some(true),
                    offensive_plays: None,
                    result: Some("TD".to_string()),
                    display_result: Some("Touchdown".to_string()),
                    plays: vec![td_play],
                }),
                previous: vec![],
            }),
            scoring_plays: vec![],
            header: None,
            win_probability: vec![],
        };

        let event = Event {
            id: "401860899".to_string(),
            date: "2026-10-03T17:00:00Z".to_string(),
            name: "Oregon State Beavers at Colorado State Rams".to_string(),
            short_name: "ORST @ CSU".to_string(),
            status: crate::models::scoreboard::EventStatus {
                display_clock: Some("7:26".to_string()),
                period: Some(3),
                status_type: crate::models::scoreboard::StatusType {
                    id: "2".to_string(),
                    name: "STATUS_IN_PROGRESS".to_string(),
                    state: "in".to_string(),
                    completed: false,
                    description: "In Progress".to_string(),
                    detail: "7:26 - 3rd Quarter".to_string(),
                    short_detail: "7:26 - 3rd".to_string(),
                },
            },
            competitions: vec![crate::models::Competition {
                id: "401860899".to_string(),
                date: "2026-10-03T17:00:00Z".to_string(),
                competitors: vec![
                    crate::models::Competitor {
                        id: "204".to_string(),
                        home_away: "away".to_string(),
                        winner: None,
                        team: crate::models::scoreboard::Team {
                            id: "204".to_string(),
                            name: Some("Beavers".to_string()),
                            display_name: "Oregon State Beavers".to_string(),
                            abbreviation: "ORST".to_string(),
                            short_display_name: Some("Oregon St".to_string()),
                            color: None,
                            alternate_color: None,
                            logo: None,
                        },
                        score: Some("21".to_string()),
                        curated_rank: None,
                        records: vec![],
                    },
                    crate::models::Competitor {
                        id: "36".to_string(),
                        home_away: "home".to_string(),
                        winner: None,
                        team: crate::models::scoreboard::Team {
                            id: "36".to_string(),
                            name: Some("Rams".to_string()),
                            display_name: "Colorado State Rams".to_string(),
                            abbreviation: "CSU".to_string(),
                            short_display_name: Some("Colorado St".to_string()),
                            color: None,
                            alternate_color: None,
                            logo: None,
                        },
                        score: Some("14".to_string()),
                        curated_rank: None,
                        records: vec![],
                    },
                ],
                situation: Some(crate::models::Situation {
                    down: Some(-1),
                    distance: Some(-1),
                    yard_line: Some(65),
                    is_red_zone: Some(false),
                    possession: None,
                    down_distance_text: None,
                    possession_text: None,
                    last_play: Some(crate::models::LastPlay {
                        id: Some("-89216018".to_string()),
                        text: Some("(C. Ojeda KICK)".to_string()),
                        stat_yardage: Some(0),
                        play_type: Some(crate::models::scoreboard::LastPlayType {
                            id: Some("61".to_string()),
                            text: Some("Extra Point Good".to_string()),
                            abbreviation: Some("XP".to_string()),
                        }),
                    }),
                }),
                broadcasts: vec![],
                venue: None,
            }],
        };

        let field_data = extract_field_data(&summary, Some(&event));

        assert_eq!(field_data.down, None, "Down must be None");
        assert_eq!(field_data.distance, None, "Distance must be None");
        assert_eq!(field_data.down_distance_text, Some("Touchdown".to_string()));
        assert_eq!(field_data.is_red_zone, false, "Red zone must be false");
        assert_eq!(field_data.visual_yard_line, Some(35), "Kickoff visual yard must be 35");
        assert_eq!(field_data.away_team_abbrev, "ORST");
        assert_eq!(field_data.home_team_abbrev, "CSU");
    }
}


