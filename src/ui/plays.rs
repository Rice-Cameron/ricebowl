use crate::app::App;
use crate::models::{GameSummary, Play};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

pub fn render_plays(f: &mut Frame, area: Rect, app: &App, summary: &GameSummary) {
    let mut all_plays: Vec<&Play> = Vec::new();

    // Collect all plays: current drive first (or last), then previous drives
    if let Some(drives) = &summary.drives {
        // Collect in chronological order
        for drive in &drives.previous {
            for play in &drive.plays {
                all_plays.push(play);
            }
        }
        if let Some(curr) = &drives.current {
            for play in &curr.plays {
                all_plays.push(play);
            }
        }
    }

    let total_plays = all_plays.len();
    let title = format!(" Play-by-Play ({} plays) [j/k to scroll] ", total_plays);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));

    if all_plays.is_empty() {
        let empty_list = List::new(vec![ListItem::new("No plays recorded yet.")]);
        f.render_widget(empty_list.block(block), area);
        return;
    }

    // Reverse to show most recent plays at top
    all_plays.reverse();

    let visible_height = area.height.saturating_sub(2) as usize;
    let offset = app.plays_scroll_offset.min(total_plays.saturating_sub(1));
    let end_idx = (offset + visible_height).min(total_plays);

    let items: Vec<ListItem> = all_plays[offset..end_idx]
        .iter()
        .map(|play| {
            let period_str = play
                .period
                .as_ref()
                .and_then(|p| p.number)
                .map(|n| format!("Q{}", n))
                .unwrap_or_else(|| "OT".to_string());

            let clock_str = play
                .clock
                .as_ref()
                .and_then(|c| c.display_value.clone())
                .unwrap_or_else(|| "--:--".to_string());

            let down_dist_str = play
                .end
                .as_ref()
                .and_then(|e| e.down_distance_text.clone())
                .unwrap_or_default();

            let text = play.text.as_deref().unwrap_or("");

            // Badges
            let mut spans = vec![
                Span::styled(
                    format!("[{} {:>5}] ", period_str, clock_str),
                    Style::default().fg(Color::Cyan),
                ),
            ];

            if !down_dist_str.is_empty() {
                spans.push(Span::styled(
                    format!("{:<18} ", down_dist_str),
                    Style::default().fg(Color::DarkGray),
                ));
            }

            // Highlight special play outcomes
            if play.scoring_play.unwrap_or(false) {
                spans.push(Span::styled(
                    "[SCORE] ",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                ));
            }

            if play.is_turnover.unwrap_or(false) {
                spans.push(Span::styled(
                    "[TURNOVER] ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ));
            }

            if play.is_penalty.unwrap_or(false) {
                spans.push(Span::styled(
                    "[PENALTY] ",
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                ));
            }

            let text_color = if play.scoring_play.unwrap_or(false) {
                Color::Yellow
            } else if play.is_turnover.unwrap_or(false) {
                Color::LightRed
            } else {
                Color::White
            };

            spans.push(Span::styled(text, Style::default().fg(text_color)));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
