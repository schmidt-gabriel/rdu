# RDU (Rust Disk Usage Analyzer) - Usage Runbook

## Overview

`rdu` is a fast, interactive terminal-based disk usage analyzer. This runbook details how to launch the application and effectively use its features to free up disk space.

## 1. Launching the Application

### Basic Usage

To scan the current directory, simply run:

```bash
rdu
```

### Scan a Specific Path

To scan a specific directory, provide the path as an argument:

```bash
rdu /path/to/scan
```

### Command-Line Options

- `--no-delete` : Disables file deletion capabilities to prevent accidental data loss. This removes the deletion UI and keybindings.
- `-h`, `--help`: Displays help information.
- `-V`, `--version`: Displays version information.

## 2. Navigation & Controls

Use the following keybindings to navigate the application:

| Key | Action |
| --- | --- |
| `j` or `↓` | Move selection down |
| `k` or `↑` | Move selection up |
| `Enter` or `→` | Drill into the selected directory |
| `←`, `Esc`, or `Backspace` | Go up to the parent directory |
| `r` | Rescan the directory from the root |
| `?` | Toggle the help overlay |
| `q` | Quit the application |

*Note: Mouse scrolling and clicking (e.g. clicking a row to drill into it) are also fully supported.*

## 3. Sorting Options

You can cycle through sorting modes by pressing `s`. The active sort mode is displayed in the bottom status bar.

1. **Size Descending** (Largest first - Default)
2. **Size Ascending** (Smallest first)
3. **Name Ascending** (Alphabetical A-Z)
4. **Name Descending** (Alphabetical Z-A)

## 4. Deletion Workflow

If you launched `rdu` without the `--no-delete` flag, you can safely remove files and directories directly from within the UI:

1. **Mark Items:** Navigate to a file or folder and press `Space` to mark it for deletion. You can mark multiple items across different directories. The right sidebar will appear to display all currently marked items.
2. **Review:** Double-check the marked items in the right-hand panel.
3. **Initiate Deletion:** Press `d` or `D`. If no items are marked, this will attempt to delete the currently highlighted item.
4. **Confirm:** A red confirmation overlay will appear displaying the number of items to delete. Press `y` to confirm and permanently delete the items, or `n` (or `Esc`) to cancel.

*Warning: Deletions are permanent and do not go to the system trash bin!*
