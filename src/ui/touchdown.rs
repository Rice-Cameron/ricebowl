use crate::app::TouchdownCelebration;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_touchdown_overlay(f: &mut Frame, area: Rect, anim: &TouchdownCelebration) {
    let popup_width = 76.min(area.width.saturating_sub(4)).max(36);
    let popup_height = 15.min(area.height.saturating_sub(2)).max(8);

    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    // Clear background so modal is crisp
    f.render_widget(Clear, popup_area);

    let frame = anim.frame;
    let border_colors = [
        Color::Yellow,
        Color::Rgb(255, 140, 0), // Gold/Orange
        Color::Cyan,
        Color::White,
        Color::LightRed,
    ];
    let current_border_color = border_colors[frame % border_colors.len()];

    let title_prefix = if frame % 2 == 0 { "🏈 ★★★ " } else { "✦ 🏈 ★ " };
    let title_suffix = if frame % 2 == 0 { " ★★★ 🏈" } else { " ★ 🏈 ✦" };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(
            Style::default()
                .fg(current_border_color)
                .add_modifier(Modifier::BOLD),
        )
        .title(Span::styled(
            format!("{}TOUCHDOWN!{}", title_prefix, title_suffix),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    let mut lines = Vec::new();

    // 1. Animated top sparkle / celebratory banner
    let count = (inner.width as usize / 3).max(6);
    let top_spans: Vec<Span> = (0..count)
        .map(|i| {
            let sym = if (i + frame) % 2 == 0 { "★ " } else { "✦ " };
            let col = border_colors[(i + frame) % border_colors.len()];
            Span::styled(sym, Style::default().fg(col))
        })
        .collect();
    lines.push(Line::from(top_spans).alignment(Alignment::Center));

    // 2. ASCII Big Art for TOUCHDOWN (if height allows)
    if inner.height >= 12 {
        lines.push(Line::from(""));
        lines.push(
            Line::from(Span::styled(
                "▀█▀ █▀█ █ █ █▀▀ █ █ █▀▄ █▀█ █ █ █ █ █",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        );
        lines.push(
            Line::from(Span::styled(
                " █  █ █ █ █ █   █▀█ █ █ █ █ █▄█ █ █ █",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        );
        lines.push(
            Line::from(Span::styled(
                " ▀  ▀▀▀ ▀▀▀ ▀▀▀ ▀ ▀ ▀▀  ▀▀▀ ▀ ▀ ▀ ▀ ▀",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        );
    } else {
        lines.push(
            Line::from(vec![
                Span::styled(
                    "💥 ⚡ ",
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled(
                    "T O U C H D O W N ! ! !",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " ⚡ 💥",
                    Style::default().fg(Color::Yellow),
                ),
            ])
            .alignment(Alignment::Center),
        );
    }

    lines.push(Line::from(""));

    // 3. Scoring Team Callout
    let team_color = if anim.is_away {
        Color::Cyan
    } else {
        Color::LightGreen
    };
    let team_display = if anim.team_abbrev.is_empty() || anim.team_name.contains(&anim.team_abbrev) {
        anim.team_name.to_uppercase()
    } else {
        format!("{} ({})", anim.team_name.to_uppercase(), anim.team_abbrev)
    };
    lines.push(
        Line::from(vec![
            Span::styled(
                "🎉 ",
                Style::default().fg(Color::Yellow),
            ),
            Span::styled(
                format!("{} SCORES!", team_display),
                Style::default()
                    .fg(team_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " 🎉",
                Style::default().fg(Color::Yellow),
            ),
        ])
        .alignment(Alignment::Center),
    );

    // 4. Play Description
    if !anim.play_text.is_empty() {
        let max_text_len = inner.width.saturating_sub(6) as usize;
        let truncated_text: String = if anim.play_text.chars().count() > max_text_len {
            anim.play_text.chars().take(max_text_len.saturating_sub(3)).collect::<String>() + "..."
        } else {
            anim.play_text.clone()
        };
        lines.push(
            Line::from(Span::styled(
                format!("\"{}\"", truncated_text),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::ITALIC),
            ))
            .alignment(Alignment::Center),
        );
    }

    // 5. Score Pill
    if !anim.score_text.is_empty() {
        lines.push(
            Line::from(Span::styled(
                format!(" [ {} ] ", anim.score_text),
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
        );
    }

    // 6. Bottom Sparkles
    let bot_spans: Vec<Span> = (0..count)
        .map(|i| {
            let sym = if (i + frame + 1) % 2 == 0 { "★ " } else { "✦ " };
            let col = border_colors[(i + frame + 1) % border_colors.len()];
            Span::styled(sym, Style::default().fg(col))
        })
        .collect();
    lines.push(Line::from(bot_spans).alignment(Alignment::Center));

    // 7. Auto-dismiss indicator
    let elapsed = anim.started_at.elapsed().as_secs_f32();
    let remaining = (anim.duration_secs - elapsed).max(0.0);
    lines.push(
        Line::from(Span::styled(
            format!("[ Press Esc or Space to close • Auto-closing in {:.0}s ]", remaining.ceil()),
            Style::default().fg(Color::DarkGray),
        ))
        .alignment(Alignment::Center),
    );

    let p = Paragraph::new(lines);
    f.render_widget(p, inner);
}
