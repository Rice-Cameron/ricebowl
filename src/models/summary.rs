use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameSummary {
    pub boxscore: Option<BoxScore>,
    pub game_info: Option<GameInfo>,
    pub drives: Option<Drives>,
    #[serde(rename = "scoringPlays", default)]
    pub scoring_plays: Vec<ScoringPlay>,
    pub header: Option<Header>,
    #[serde(rename = "winprobability", default)]
    pub win_probability: Vec<WinProbability>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Header {
    pub id: Option<String>,
    #[serde(default)]
    pub competitions: Vec<HeaderCompetition>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderCompetition {
    pub id: Option<String>,
    #[serde(default)]
    pub competitors: Vec<HeaderCompetitor>,
    pub status: Option<HeaderStatus>,
    #[serde(default)]
    pub broadcasts: Vec<HeaderBroadcast>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderBroadcast {
    pub media: Option<HeaderMedia>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderMedia {
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderStatus {
    #[serde(rename = "type")]
    pub status_type: Option<HeaderStatusType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderStatusType {
    pub detail: Option<String>,
    #[serde(rename = "shortDetail")]
    pub short_detail: Option<String>,
    pub completed: Option<bool>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderCompetitor {
    pub id: Option<String>,
    pub score: Option<String>,
    #[serde(rename = "homeAway")]
    pub home_away: Option<String>,
    pub winner: Option<bool>,
    pub team: Option<HeaderTeam>,
    pub possession: Option<bool>,
    pub timeouts: Option<i32>,
    #[serde(rename = "timeoutsUsed")]
    pub timeouts_used: Option<i32>,
    pub rank: Option<i32>,
    #[serde(default)]
    pub record: Vec<RecordSummary>,
    #[serde(default)]
    pub linescores: Vec<LineScore>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LineScore {
    #[serde(rename = "displayValue")]
    pub display_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecordSummary {
    pub summary: Option<String>,
    #[serde(rename = "type")]
    pub record_type: Option<String>,
    #[serde(rename = "displayValue")]
    pub display_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderTeam {
    pub id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub abbreviation: Option<String>,
    pub color: Option<String>,
    #[serde(rename = "alternateColor")]
    pub alternate_color: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameInfo {
    pub venue: Option<Venue>,
    pub weather: Option<Weather>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Venue {
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Address {
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Weather {
    pub temperature: Option<i32>,
    #[serde(rename = "displayValue")]
    pub display_value: Option<String>,
    pub condition_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Drives {
    pub current: Option<Drive>,
    #[serde(default)]
    pub previous: Vec<Drive>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Drive {
    pub id: Option<String>,
    pub description: Option<String>,
    pub team: Option<DriveTeam>,
    pub start: Option<DrivePoint>,
    pub end: Option<DrivePoint>,
    #[serde(rename = "timeElapsed")]
    pub time_elapsed: Option<DisplayValue>,
    pub yards: Option<i32>,
    #[serde(rename = "isScore")]
    pub is_score: Option<bool>,
    #[serde(rename = "offensivePlays")]
    pub offensive_plays: Option<i32>,
    pub result: Option<String>,
    #[serde(rename = "displayResult")]
    pub display_result: Option<String>,
    #[serde(default)]
    pub plays: Vec<Play>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DriveTeam {
    pub id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub abbreviation: Option<String>,
    pub logo: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DrivePoint {
    pub period: Option<Period>,
    pub clock: Option<DisplayValue>,
    #[serde(rename = "yardLine")]
    pub yard_line: Option<i32>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Period {
    pub number: Option<i32>,
    #[serde(rename = "displayValue")]
    pub display_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DisplayValue {
    #[serde(rename = "displayValue")]
    pub display_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Play {
    pub id: Option<String>,
    #[serde(rename = "sequenceNumber")]
    pub sequence_number: Option<String>,
    #[serde(rename = "type")]
    pub play_type: Option<PlayType>,
    pub text: Option<String>,
    pub away_score: Option<i32>,
    pub home_score: Option<i32>,
    pub period: Option<Period>,
    pub clock: Option<DisplayValue>,
    #[serde(rename = "scoringPlay")]
    pub scoring_play: Option<bool>,
    pub priority: Option<bool>,
    #[serde(rename = "statYardage")]
    pub stat_yardage: Option<i32>,
    pub start: Option<PlayPosition>,
    pub end: Option<PlayPosition>,
    #[serde(rename = "isPenalty")]
    pub is_penalty: Option<bool>,
    #[serde(rename = "isTurnover")]
    pub is_turnover: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayType {
    pub id: Option<String>,
    pub text: Option<String>,
    pub abbreviation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayPosition {
    pub down: Option<i32>,
    pub distance: Option<i32>,
    #[serde(rename = "yardLine")]
    pub yard_line: Option<i32>,
    #[serde(rename = "yardsToEndzone")]
    pub yards_to_endzone: Option<i32>,
    #[serde(rename = "downDistanceText")]
    pub down_distance_text: Option<String>,
    #[serde(rename = "shortDownDistanceText")]
    pub short_down_distance_text: Option<String>,
    #[serde(rename = "possessionText")]
    pub possession_text: Option<String>,
    pub team: Option<PlayTeamRef>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayTeamRef {
    pub id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoxScore {
    #[serde(default)]
    pub teams: Vec<BoxScoreTeam>,
    #[serde(default)]
    pub players: Vec<BoxScorePlayers>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoxScoreTeam {
    pub team: BoxScoreTeamInfo,
    #[serde(default)]
    pub statistics: Vec<TeamStatistic>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoxScoreTeamInfo {
    pub id: Option<String>,
    pub uid: Option<String>,
    pub slug: Option<String>,
    pub location: Option<String>,
    pub name: Option<String>,
    pub abbreviation: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TeamStatistic {
    pub name: Option<String>,
    #[serde(rename = "displayValue")]
    pub display_value: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BoxScorePlayers {
    pub team: BoxScoreTeamInfo,
    #[serde(default)]
    pub statistics: Vec<PlayerCategoryStats>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerCategoryStats {
    pub name: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub athletes: Vec<AthleteStats>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AthleteStats {
    pub athlete: AthleteInfo,
    #[serde(default)]
    pub stats: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AthleteInfo {
    pub id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    pub jersey: Option<String>,
    pub position: Option<AthletePosition>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AthletePosition {
    pub abbreviation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoringPlay {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub play_type: Option<PlayType>,
    pub text: Option<String>,
    pub away_score: Option<i32>,
    pub home_score: Option<i32>,
    pub period: Option<Period>,
    pub clock: Option<DisplayValue>,
    pub team: Option<DriveTeam>,
    #[serde(rename = "scoringType")]
    pub scoring_type: Option<ScoringType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScoringType {
    pub name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub abbreviation: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WinProbability {
    pub tie_percentage: Option<f64>,
    pub home_win_percentage: Option<f64>,
    pub play_id: Option<String>,
}
