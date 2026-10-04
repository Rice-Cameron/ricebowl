# 🍚 ricebowl: College Football Live Terminal UI

A fast, async terminal user interface (TUI) written in Rust to track live College Football scores, play-by-play, field position, ball possession, drive charts, and player statistics using ESPN's free public REST API.

---

## ✨ Features

- **🔴 Live Scoreboard:**
  - Real-time scores, quarter, game clock, timeouts, and TV network info.
  - Live down & distance with ball possession indicators (`🏈`).
  - Top 25 AP rankings and team records.
- **⭐ Favorite Teams:**
  - Mark any team as a favorite with `f`.
  - Favorited teams are pinned with gold stars (`★`) to the top of the scoreboard.
  - Filter games to show only your favorites.
  - Automatically saved to `~/.config/ricebowl/config.json`.
- **🏟️ Visual Football Field & Gamecast:**
  - Dynamic 100-yard ASCII football field with end zones, yard lines, and hash marks.
  - Real-time line-of-scrimmage marker (`🏈`) and 1st down target line (`┃`).
  - Red zone indicator (`[🚨 RED ZONE]`).
  - Current drive stats (plays, yards, elapsed time).
  - Scoring summary with quarter-by-quarter breakdown.
- **📜 Play-by-Play:**
  - Chronological play feed with color-coded tags: `[SCORE]`, `[TURNOVER]`, `[PENALTY]`, `[1ST DOWN]`.
  - Scrollable with `j`/`k` or `PgUp`/`PgDn`.
- **📊 Box Score & Player Stats:**
  - Categorized player statistics: **Passing**, **Rushing**, **Receiving**, **Defense**, and **Kicking**.
  - Side-by-side or stacked comparative tables for Away and Home teams.
  - Switch categories instantly with `h`/`l` or Left/Right arrows.
- **📈 Team Stats & Venue Info:**
  - Head-to-head comparison for 1st downs, 3rd/4th down efficiency, total yards, passing/rushing yards, turnovers, penalties, and time of possession.
  - Stadium name, city, state, temperature, and weather conditions.
- **⚡ Async & Non-Blocking:**
  - Automatic background refresh every 15 seconds.
  - Zero terminal stutter or input lag thanks to Tokio async background channels.

---

## ⌨️ Keybindings

### Global & Scoreboard View
| Key | Action |
| :--- | :--- |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` | Open game details / gamecast |
| `Tab` | Cycle filters (`All FBS` → `Favorites ★` → `Live Now ●` → `Top 25`) |
| `f` | Open favorite team toggle dialog |
| `r` | Manually force refresh scores |
| `?` | Toggle keyboard shortcuts help popup |
| `q` | Quit application |

### Game Details View
| Key | Action |
| :--- | :--- |
| `Esc` / `Backspace` / `q` | Return to scoreboard list |
| `1` - `4` | Directly switch tabs (`1: Gamecast`, `2: Plays`, `3: Box Score`, `4: Team Stats`) |
| `Tab` / `Shift-Tab` | Cycle tabs forward / backward |
| `h` / `l` or `←` / `→` | Switch stat category in Box Score (Passing, Rushing, Receiving, Defense, Kicking) |
| `j` / `k` or `PgUp` / `PgDn` | Scroll through plays or player lists |
| `r` | Refresh the active game's play-by-play and stats |

---

## 🚀 Running the App

```bash
cd ~/Projects/ricebowl
cargo run --release
```

Or install to `~/.cargo/bin`:
```bash
cargo install --path .
```
