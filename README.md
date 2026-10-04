# ricebowl

A terminal user interface (TUI) for tracking live NCAA College Football scores, play-by-play, field position, drive charts, and player statistics using ESPN's unofficial API. Built with Rust and Ratatui.

> [!WARNING]
> **Disclaimer**: This application relies on ESPN's unofficial, undocumented API. Endpoints and data structures are subject to change and may be prone to randomly breaking at any time.

---

## Features

- **Live Scoreboard**: Real-time scores, quarter, game clock, timeouts, TV network info, AP Top 25 rankings, records, and live down & distance.
- **Visual Gamecast & Field View**: Dynamic 100-yard ASCII football field showing line of scrimmage, first down marker, red zone status, and drive summaries.
- **Play-by-Play**: Full chronological play feed tagged by play type (scoring plays, turnovers, penalties, and first downs).
- **Box Scores & Player Stats**: Detailed player statistics categorized across Passing, Rushing, Receiving, Defense, and Kicking.
- **Team Stats & Game Info**: Head-to-head comparison for first downs, 3rd/4th down efficiency, total yards, turnovers, penalties, and time of possession, plus venue and weather conditions.
- **Favorite Teams**: Save favorite teams to `~/.config/ricebowl/config.json` with quick filter cycling (`Tab`).
- **Touchdown Alerts**: In-app celebration banner and overlay animation when a touchdown occurs.
- **Async & Responsive**: Automatic 15-second background polling via Tokio channels without UI stutter or input lag.

---

## Installation

### From Source

Requires Rust and Cargo:

```bash
git clone git@github.com:Rice-Cameron/ricebowl.git
cd ricebowl
cargo install --path .
```

Or build the release binary directly:

```bash
cargo build --release
# Executable will be located at target/release/ricebowl
```

---

## Keybindings

### Scoreboard View

| Key | Action |
| :--- | :--- |
| `j` / `Down` | Move down |
| `k` / `Up` | Move up |
| `Enter` | Open game details |
| `Tab` | Cycle filters (All FBS -> Favorites -> Live -> Top 25) |
| `f` | Toggle favorite status for selected game's teams |
| `r` | Force refresh scores |
| `?` | Show help modal |
| `q` | Quit |

### Game Details View

| Key | Action |
| :--- | :--- |
| `Esc` / `Backspace` / `q` | Return to scoreboard |
| `1` - `4` | Select tab (1: Gamecast, 2: Plays, 3: Box Score, 4: Team Stats) |
| `Tab` / `Shift+Tab` | Cycle tabs forward / backward |
| `h` / `l` or `Left` / `Right` | Switch stat category in Box Score |
| `j` / `k` or `PgUp` / `PgDn` | Scroll plays or player lists |
| `t` | Trigger touchdown celebration animation |
| `r` | Force refresh active game data |
