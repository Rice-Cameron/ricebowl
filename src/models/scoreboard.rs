use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoreboardResponse {
    pub leagues: Option<Vec<League>>,
    pub season: Option<Season>,
    #[serde(default)]
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct League {
    pub id: String,
    pub name: String,
    pub abbreviation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Season {
    pub year: u32,
    #[serde(rename = "type")]
    pub season_type: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Event {
    pub id: String,
    pub date: String,
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    pub status: EventStatus,
    #[serde(default)]
    pub competitions: Vec<Competition>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EventStatus {
    pub period: Option<i32>,
    #[serde(rename = "displayClock")]
    pub display_clock: Option<String>,
    #[serde(rename = "type")]
    pub status_type: StatusType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusType {
    pub id: String,
    pub name: String,
    pub state: String, // "pre", "in", "post"
    pub completed: bool,
    pub description: String,
    pub detail: String,
    #[serde(rename = "shortDetail")]
    pub short_detail: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Competition {
    pub id: String,
    pub date: String,
    #[serde(default)]
    pub competitors: Vec<Competitor>,
    pub situation: Option<Situation>,
    #[serde(default)]
    pub broadcasts: Vec<Broadcast>,
    pub venue: Option<VenueSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VenueSummary {
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    pub address: Option<AddressSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressSummary {
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Competitor {
    pub id: String,
    #[serde(rename = "homeAway")]
    pub home_away: String, // "home" or "away"
    pub winner: Option<bool>,
    pub score: Option<String>,
    #[serde(rename = "curatedRank")]
    pub curated_rank: Option<CuratedRank>,
    pub team: Team,
    #[serde(default)]
    pub records: Vec<RecordSummary>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CuratedRank {
    pub current: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Team {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "shortDisplayName")]
    pub short_display_name: Option<String>,
    pub abbreviation: String,
    pub color: Option<String>,
    #[serde(rename = "alternateColor")]
    pub alternate_color: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecordSummary {
    pub summary: Option<String>,
    #[serde(rename = "type")]
    pub record_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Broadcast {
    #[serde(default)]
    pub names: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Situation {
    pub down: Option<i32>,
    pub distance: Option<i32>,
    #[serde(rename = "yardLine")]
    pub yard_line: Option<i32>,
    #[serde(rename = "isRedZone")]
    pub is_red_zone: Option<bool>,
    pub possession: Option<String>, // team ID
    #[serde(rename = "downDistanceText")]
    pub down_distance_text: Option<String>,
    #[serde(rename = "possessionText")]
    pub possession_text: Option<String>,
    #[serde(rename = "lastPlay")]
    pub last_play: Option<LastPlay>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LastPlay {
    pub id: Option<String>,
    pub text: Option<String>,
    #[serde(rename = "statYardage")]
    pub stat_yardage: Option<i32>,
    #[serde(rename = "type")]
    pub play_type: Option<LastPlayType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LastPlayType {
    pub id: Option<String>,
    pub text: Option<String>,
    pub abbreviation: Option<String>,
}

