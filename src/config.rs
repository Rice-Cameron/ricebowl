use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub favorite_teams: HashSet<String>, // Stores abbreviations (e.g. "TENN", "ALA", "UGA")
    #[serde(default = "default_refresh_secs")]
    pub refresh_interval_secs: u64,
}

fn default_refresh_secs() -> u64 {
    15
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            favorite_teams: HashSet::new(),
            refresh_interval_secs: 15,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("ricebowl");
            p.push("config.json");
            p
        })
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
                        return config;
                    }
                }
            }
        }
        AppConfig::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let data = serde_json::to_string_pretty(self)?;
            fs::write(path, data)?;
        }
        Ok(())
    }

    pub fn is_favorite(&self, abbrev: &str, id: &str) -> bool {
        self.favorite_teams.contains(abbrev) || self.favorite_teams.contains(id)
    }

    pub fn toggle_favorite(&mut self, abbrev: &str) -> bool {
        let is_now_fav = if self.favorite_teams.contains(abbrev) {
            self.favorite_teams.remove(abbrev);
            false
        } else {
            self.favorite_teams.insert(abbrev.to_string());
            true
        };
        let _ = self.save();
        is_now_fav
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_favorites_toggle() {
        let mut cfg = AppConfig::default();
        assert!(!cfg.is_favorite("TENN", "2633"));

        assert!(cfg.toggle_favorite("TENN"));
        assert!(cfg.is_favorite("TENN", "2633"));

        assert!(!cfg.toggle_favorite("TENN"));
        assert!(!cfg.is_favorite("TENN", "2633"));
    }
}
