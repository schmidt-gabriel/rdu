# RDU 🦀

A terminal disk-usage analyzer written in Rust.


## Build & Run

```bash
# Clone / navigate
cd rdu

# Debug build (faster compile)
cargo run

# Release build (faster runtime)
cargo build --release

```

Requires Rust 1.75+. Install via https://rustup.rs if needed.

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` / `→` | Drill into directory |
| `←` / `Esc` / `Backspace` | Go up to parent |
| `r` | Rescan from root |
| `?` | Toggle help overlay |
| `q` | Quit |

## Architecture

```
src/
├── main.rs       — Entry point, terminal setup, event loop
├── app.rs        — Application state (tree, navigation, scan channel)
├── scanner.rs    — Recursive filesystem walker (walkdir + rayon)
└── ui.rs         — All ratatui widget rendering
```


## Possible Extensions

- [ ] Mouse support (click arc to drill down)
- [ ] Export to SVG
- [ ] Filter by extension (show only `.mp4`, etc.)
- [ ] Delete selected file/dir with confirmation
- [ ] Sort modes (by name, by count, by modified date)
- [ ] Config file (`~/.config/rdu/config.toml`)
