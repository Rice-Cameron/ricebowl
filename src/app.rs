use crate::config::AppConfig;
use crate::models::{Event, GameSummary, ScoreboardResponse};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Scoreboard,
    GameDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailTab {
    Gamecast,
    Plays,
    BoxScore,
    TeamStats,
}

impl DetailTab {
    pub fn all() -> &'static [DetailTab] {
        &[
            DetailTab::Gamecast,
            DetailTab::Plays,
            DetailTab::BoxScore,
            DetailTab::TeamStats,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            DetailTab::Gamecast => "1. Gamecast & Field",
            DetailTab::Plays => "2. Play-by-Play",
            DetailTab::BoxScore => "3. Box Score",
            DetailTab::TeamStats => "4. Team Stats & Info",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            DetailTab::Gamecast => DetailTab::Plays,
            DetailTab::Plays => DetailTab::BoxScore,
            DetailTab::BoxScore => DetailTab::TeamStats,
            DetailTab::TeamStats => DetailTab::Gamecast,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            DetailTab::Gamecast => DetailTab::TeamStats,
            DetailTab::Plays => DetailTab::Gamecast,
            DetailTab::BoxScore => DetailTab::Plays,
            DetailTab::TeamStats => DetailTab::BoxScore,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterMode {
    All,
    Favorites,
    Live,
    Top25,
}

impl FilterMode {
    pub fn label(&self) -> &'static str {
        match self {
            FilterMode::All => "All FBS",
            FilterMode::Favorites => "Favorites ★",
            FilterMode::Live => "Live Now ●",
            FilterMode::Top25 => "Top 25 Rank",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            FilterMode::All => FilterMode::Favorites,
            FilterMode::Favorites => FilterMode::Live,
            FilterMode::Live => FilterMode::Top25,
            FilterMode::Top25 => FilterMode::All,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxScoreCategory {
    Passing,
    Rushing,
    Receiving,
    Defensive,
    Kicking,
}

impl BoxScoreCategory {
    pub fn all() -> &'static [BoxScoreCategory] {
        &[
            BoxScoreCategory::Passing,
            BoxScoreCategory::Rushing,
            BoxScoreCategory::Receiving,
            BoxScoreCategory::Defensive,
            BoxScoreCategory::Kicking,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BoxScoreCategory::Passing => "passing",
            BoxScoreCategory::Rushing => "rushing",
            BoxScoreCategory::Receiving => "receiving",
            BoxScoreCategory::Defensive => "defensive",
            BoxScoreCategory::Kicking => "kicking",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            BoxScoreCategory::Passing => "Passing",
            BoxScoreCategory::Rushing => "Rushing",
            BoxScoreCategory::Receiving => "Receiving",
            BoxScoreCategory::Defensive => "Defense",
            BoxScoreCategory::Kicking => "Kicking",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            BoxScoreCategory::Passing => BoxScoreCategory::Rushing,
            BoxScoreCategory::Rushing => BoxScoreCategory::Receiving,
            BoxScoreCategory::Receiving => BoxScoreCategory::Defensive,
            BoxScoreCategory::Defensive => BoxScoreCategory::Kicking,
            BoxScoreCategory::Kicking => BoxScoreCategory::Passing,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            BoxScoreCategory::Passing => BoxScoreCategory::Kicking,
            BoxScoreCategory::Rushing => BoxScoreCategory::Passing,
            BoxScoreCategory::Receiving => BoxScoreCategory::Rushing,
            BoxScoreCategory::Defensive => BoxScoreCategory::Receiving,
            BoxScoreCategory::Kicking => BoxScoreCategory::Defensive,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TouchdownCelebration {
    pub team_name: String,
    pub team_abbrev: String,
    pub play_text: String,
    pub score_text: String,
    pub frame: usize,
    pub started_at: Instant,
    pub duration_secs: f32,
    pub is_away: bool,
}

pub struct App {
    pub view_mode: ViewMode,
    pub detail_tab: DetailTab,
    pub filter_mode: FilterMode,
    pub boxscore_category: BoxScoreCategory,
    pub show_help: bool,
    pub show_fav_dialog: bool,

    pub config: AppConfig,
    pub scoreboard: Option<ScoreboardResponse>,
    pub selected_game_index: usize,

    pub active_game_id: Option<String>,
    pub active_summary: Option<GameSummary>,

    pub plays_scroll_offset: usize,
    pub boxscore_scroll_offset: usize,

    pub is_loading: bool,
    pub status_message: Option<(String, Instant)>,
    pub last_updated: Option<Instant>,

    pub touchdown_celebration: Option<TouchdownCelebration>,
    pub last_seen_touchdown_id: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let config = AppConfig::load();
        Self {
            view_mode: ViewMode::Scoreboard,
            detail_tab: DetailTab::Gamecast,
            filter_mode: FilterMode::All,
            boxscore_category: BoxScoreCategory::Passing,
            show_help: false,
            show_fav_dialog: false,
            config,
            scoreboard: None,
            selected_game_index: 0,
            active_game_id: None,
            active_summary: None,
            plays_scroll_offset: 0,
            boxscore_scroll_offset: 0,
            is_loading: true,
            status_message: Some(("Starting up... fetching scores".to_string(), Instant::now())),
            last_updated: None,
            touchdown_celebration: None,
            last_seen_touchdown_id: None,
        }
    }

    pub fn is_animating(&self) -> bool {
        self.touchdown_celebration.is_some()
    }

    pub fn tick_animation(&mut self) {
        if let Some(td) = &mut self.touchdown_celebration {
            td.frame = td.frame.saturating_add(1);
            if td.started_at.elapsed().as_secs_f32() >= td.duration_secs {
                self.touchdown_celebration = None;
            }
        }
    }

    pub fn dismiss_animation(&mut self) {
        self.touchdown_celebration = None;
    }

    pub fn trigger_touchdown_animation(
        &mut self,
        team_name: String,
        team_abbrev: String,
        play_text: String,
        score_text: String,
        is_away: bool,
    ) {
        self.touchdown_celebration = Some(TouchdownCelebration {
            team_name,
            team_abbrev,
            play_text,
            score_text,
            frame: 0,
            started_at: Instant::now(),
            duration_secs: 4.5,
            is_away,
        });
    }

    pub fn trigger_touchdown_for_active_game(&mut self) {
        let Some(summary) = &self.active_summary else {
            return;
        };
        if let Some(td_info) = crate::ui::gamecast::get_recent_touchdown(summary, self.selected_event(), self) {
            self.trigger_touchdown_animation(
                td_info.team_name,
                td_info.team_abbrev,
                td_info.play_text,
                format!("{} {} - {} {}", td_info.away_team, td_info.away_score, td_info.home_score, td_info.home_team),
                td_info.is_away,
            );
        } else {
            let (away, _) = crate::ui::gamecast::get_team_abbrevs(summary, self.selected_event());
            let (away_name, away_score, _, _) = crate::ui::gamecast::get_competitor_info(summary, self.selected_event(), "away", self);
            let (home_name, home_score, _, _) = crate::ui::gamecast::get_competitor_info(summary, self.selected_event(), "home", self);
            self.trigger_touchdown_animation(
                away_name.clone(),
                away,
                "15 Yd Rush TOUCHDOWN".to_string(),
                format!("{} {} - {} {}", away_name, away_score, home_score, home_name),
                true,
            );
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn filtered_events(&self) -> Vec<&Event> {
        let Some(sb) = &self.scoreboard else {
            return Vec::new();
        };

        let mut events: Vec<&Event> = sb
            .events
            .iter()
            .filter(|ev| {
                let comp = ev.competitions.first();
                let away = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "away"));
                let home = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "home"));

                match self.filter_mode {
                    FilterMode::All => true,
                    FilterMode::Favorites => {
                        let away_fav = away
                            .map(|a| self.config.is_favorite(&a.team.abbreviation, &a.team.id))
                            .unwrap_or(false);
                        let home_fav = home
                            .map(|h| self.config.is_favorite(&h.team.abbreviation, &h.team.id))
                            .unwrap_or(false);
                        away_fav || home_fav
                    }
                    FilterMode::Live => ev.status.status_type.state == "in",
                    FilterMode::Top25 => {
                        let away_rank = away
                            .and_then(|a| a.curated_rank.as_ref())
                            .and_then(|r| r.current)
                            .unwrap_or(99);
                        let home_rank = home
                            .and_then(|h| h.curated_rank.as_ref())
                            .and_then(|r| r.current)
                            .unwrap_or(99);
                        (away_rank <= 25 && away_rank > 0) || (home_rank <= 25 && home_rank > 0)
                    }
                }
            })
            .collect();

        // Sort: Favorite games first, then live games, then by start time
        events.sort_by(|a, b| {
            let a_fav = self.event_has_favorite(a);
            let b_fav = self.event_has_favorite(b);
            if a_fav != b_fav {
                return b_fav.cmp(&a_fav);
            }

            let a_live = a.status.status_type.state == "in";
            let b_live = b.status.status_type.state == "in";
            if a_live != b_live {
                return b_live.cmp(&a_live);
            }

            a.date.cmp(&b.date)
        });

        events
    }

    pub fn event_has_favorite(&self, ev: &Event) -> bool {
        let comp = ev.competitions.first();
        let away = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "away"));
        let home = comp.and_then(|c| c.competitors.iter().find(|t| t.home_away == "home"));

        let away_fav = away
            .map(|a| self.config.is_favorite(&a.team.abbreviation, &a.team.id))
            .unwrap_or(false);
        let home_fav = home
            .map(|h| self.config.is_favorite(&h.team.abbreviation, &h.team.id))
            .unwrap_or(false);

        away_fav || home_fav
    }

    pub fn selected_event(&self) -> Option<&Event> {
        let events = self.filtered_events();
        if events.is_empty() {
            None
        } else {
            let idx = self.selected_game_index.min(events.len().saturating_sub(1));
            Some(events[idx])
        }
    }

    pub fn select_next_game(&mut self) {
        let len = self.filtered_events().len();
        if len > 0 {
            if self.selected_game_index + 1 < len {
                self.selected_game_index += 1;
            } else {
                self.selected_game_index = 0;
            }
        }
    }

    pub fn select_prev_game(&mut self) {
        let len = self.filtered_events().len();
        if len > 0 {
            if self.selected_game_index > 0 {
                self.selected_game_index -= 1;
            } else {
                self.selected_game_index = len - 1;
            }
        }
    }

    pub fn open_selected_game(&mut self) -> Option<String> {
        if let Some(ev) = self.selected_event() {
            let id = ev.id.clone();
            self.active_game_id = Some(id.clone());
            self.view_mode = ViewMode::GameDetail;
            self.detail_tab = DetailTab::Gamecast;
            self.plays_scroll_offset = 0;
            self.boxscore_scroll_offset = 0;
            Some(id)
        } else {
            None
        }
    }

    pub fn close_game_detail(&mut self) {
        self.view_mode = ViewMode::Scoreboard;
    }

    pub fn toggle_favorite_for_selected(&mut self, is_away: bool) {
        let team_info = if let Some(ev) = self.selected_event() {
            if let Some(comp) = ev.competitions.first() {
                let competitor = if is_away {
                    comp.competitors.iter().find(|c| c.home_away == "away")
                } else {
                    comp.competitors.iter().find(|c| c.home_away == "home")
                };
                competitor.map(|t| (t.team.abbreviation.clone(), t.team.display_name.clone()))
            } else {
                None
            }
        } else {
            None
        };

        if let Some((abbrev, display_name)) = team_info {
            let now_fav = self.config.toggle_favorite(&abbrev);
            let state_str = if now_fav { "added to" } else { "removed from" };
            self.set_status(format!("{} {} favorites ★", display_name, state_str));
        }
    }
}
