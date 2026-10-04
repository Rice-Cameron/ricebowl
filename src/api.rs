use crate::models::{GameSummary, ScoreboardResponse};
use reqwest::Client;
use std::time::Duration;

#[derive(Clone)]
pub struct EspnClient {
    client: Client,
}

impl Default for EspnClient {
    fn default() -> Self {
        Self::new()
    }
}

impl EspnClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (compatible; RiceBowl/1.0; +https://github.com/Rice-Cameron/ricebowl)")
            .build()
            .expect("Failed to initialize HTTP client");
        Self { client }
    }

    /// Fetches the live college football scoreboard.
    /// `groups=80` filters to FBS (Division 1-A).
    pub async fn fetch_scoreboard(&self, fbs_only: bool) -> Result<ScoreboardResponse, reqwest::Error> {
        let url = if fbs_only {
            "https://site.api.espn.com/apis/site/v2/sports/football/college-football/scoreboard?groups=80&limit=100"
        } else {
            "https://site.api.espn.com/apis/site/v2/sports/football/college-football/scoreboard?limit=100"
        };

        let response = self.client.get(url).send().await?;
        let scoreboard = response.json::<ScoreboardResponse>().await?;
        Ok(scoreboard)
    }

    /// Fetches game summary, drives, play-by-play, and boxscore for a given event ID.
    pub async fn fetch_summary(&self, event_id: &str) -> Result<GameSummary, reqwest::Error> {
        let url = format!(
            "https://site.api.espn.com/apis/site/v2/sports/football/college-football/summary?event={}",
            event_id
        );

        let response = self.client.get(&url).send().await?;
        let summary = response.json::<GameSummary>().await?;
        Ok(summary)
    }
}
