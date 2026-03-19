# RDU 🦀

A terminal disk-usage analyzer written in Rust.

## Screenshots

**Main Window**  
<img width="800" alt="Main Window" src="https://github.com/user-attachments/assets/ab5cbc2a-2955-4859-92d7-57d0e69f6c43" />

**Help Window**  
<img width="800" alt="Help Window" src="https://github.com/user-attachments/assets/d3ffb8a4-2914-411d-9bee-90c00160cbe1" />

**Deletion Confirmation Window**  
<img width="800" alt="Deletion Confirmation Window" src="https://github.com/user-attachments/assets/3c742946-9ee1-464c-b505-6849cf6ce8ab" />


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

Requires Rust 1.75+. Install via [rustup](https://rustup.rs) if needed.

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
- `--no-delete`: Prevent deletions

## Keybindings

| Key | Action |
| ----- | -------- |
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `Enter` / `→` | Drill into directory |
| `←` / `Esc` / `Backspace` | Go up to parent |
| `Space` | Mark item for deletion |
| `d` / `D` | Delete marked item(s) (with prompt) |
| `s` | Cycle sort mode |
| `r` | Rescan from root |
| `?` | Toggle help overlay |
| `q` | Quit |

## Architecture

```text
src/
├── main.rs       — Entry point, terminal setup, event loop
├── app.rs        — Application state (tree, navigation, scan channel)
├── scanner.rs    — Recursive filesystem walker (walkdir + rayon)
└── ui.rs         — All ratatui widget rendering
```

## Possible Extensions

- [ ] Filter by extension (show only `.mp4`, etc.)
