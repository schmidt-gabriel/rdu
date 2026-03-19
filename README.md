# RDU 🦀

A terminal disk-usage analyzer written in Rust.


## Installation

### Manual Install
1. Go to the GitHub [releases](https://github.com/schmidt-gabriel/rdu/releases/latest) page
2. Download the latest non-dSYM build (i.e., `rdu-X.Y.Z.tar.gz` for Linux/macOS or `rdu-X.Y.Z.zip` for Windows)
3. Unzip the archive
4. Run the `rdu` binary from the terminal

### Install via Homebrew :beer:
1. Install [Homebrew](https://brew.sh) if you haven't already
2. Open Terminal and run `brew tap schmidt-gabriel/tap`
3. Run `brew install rdu`


Requires Rust 1.75+. Install via https://rustup.rs if needed.

## Usage

Run `rdu` to scan the current directory, or provide a path to scan a specific directory:

```bash
# Scan current directory
rdu

# Scan a specific directory
rdu /path/to/directory
```

You can also use the following flags:
- `-h`, `--help`: Print help information
- `-V`, `--version`: Print version information

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` / `→` | Drill into directory |
| `←` / `Esc` / `Backspace` | Go up to parent |
| `s` | Cycle sort mode |
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

- [ ] Filter by extension (show only `.mp4`, etc.)
- [ ] Delete selected file/dir with confirmation
