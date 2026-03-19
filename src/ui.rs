use crate::{app::App, scanner::fmt_size};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

const ACCENT: Color = Color::Rgb(88, 166, 255);
const DIM: Color = Color::Rgb(110, 118, 129);
const FG: Color = Color::Rgb(201, 209, 217);
const BG: Color = Color::Rgb(13, 17, 23);
const SELECTED_BG: Color = Color::Rgb(31, 51, 88);

pub fn draw(frame: &mut Frame, app: &mut App) {
    let area = frame.size();

    // Background
    frame.render_widget(Block::default().style(Style::default().bg(BG)), area);

    draw_file_panel(frame, app, area);
    draw_statusbar(frame, app, area);

    if app.show_help {
        draw_help_overlay(frame, area);
    }
}

// ── Left panel: file list ────────────────────────────────────────────────────

fn draw_file_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    // Reserve last 1 row for status bar
    let inner = Rect {
        height: area.height.saturating_sub(1),
        ..area
    };

    let max_title_len = (area.width as usize).saturating_sub(8).max(34);
    let title = if app.scanning {
        " 󰑓 Scanning… ".to_string()
    } else {
        format!(" 󰉋 {} ", short_path(&app.current_path_display(), max_title_len))
    };

    let block = Block::default()
        .title(Line::from(vec![Span::styled(
            title,
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(42, 47, 58)))
        .style(Style::default().bg(BG));

    if app.scanning || app.tree.is_none() {
        let msg = if app.scanning {
            "Scanning filesystem…"
        } else {
            "No data"
        };
        let p = Paragraph::new(msg)
            .block(block)
            .style(Style::default().fg(DIM));
        frame.render_widget(p, inner);
        return;
    }

    let children = app.current_children();
    let max_size = children.first().map(|c| c.size).filter(|&s| s > 0).unwrap_or(1);

    // Build list items
    let items: Vec<ListItem> = children
        .iter()
        .enumerate()
        .map(|(i, child)| {
            let icon = if child.is_dir { "󰉋 " } else { "󰈙 " };
            let bar_width = 10usize;
            let ratio = child.size as f64 / max_size as f64;
            
            // Calculate total eighths of a block (10 blocks = 80 eighths max)
            let eighths = (ratio * bar_width as f64 * 8.0).round() as usize;
            let eighths = eighths.min(bar_width * 8);

            let full_blocks = eighths / 8;
            let fraction = eighths % 8;

            let mut bar = "█".repeat(full_blocks);
            if full_blocks < bar_width {
                let fract_char = ["", "▏", "▎", "▍", "▌", "▋", "▊", "▉"][fraction];
                bar.push_str(fract_char);
                let spaces = bar_width.saturating_sub(full_blocks + if fraction > 0 { 1 } else { 0 });
                bar.push_str(&" ".repeat(spaces));
            }
            bar.push('▏'); // Add trailing delimiter

            let size_str = fmt_size(child.size);
            // Dynamic width: borders(2) + size(10) + space(1) + bar(11) + space(1) + icon(2) + highlight(2) = 29
            let name_width = (area.width as usize).saturating_sub(29).max(10);
            let name = truncate(&child.name, name_width);

            let is_selected = i == app.selected;
            let prefix = if is_selected {
                if child.is_dir { "▶ " } else { "● " }
            } else {
                "  "
            };

            let line = Line::from(vec![
                Span::styled(
                    prefix,
                    if is_selected {
                        Style::default().fg(Color::Rgb(121, 192, 255)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    }
                ),
                Span::styled(format!("{:>10} ", size_str), Style::default().fg(DIM)),
                Span::styled(format!("{} ", bar), Style::default().fg(Color::Rgb(56, 139, 253))),
                Span::styled(
                    icon,
                    Style::default().fg(if child.is_dir { ACCENT } else { DIM }),
                ),
                Span::styled(
                    format!("{:<width$}", name, width = name_width),
                    Style::default().fg(FG),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let mut list_state = ListState::default();
    list_state.select(Some(app.selected));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(SELECTED_BG)
                .fg(Color::Rgb(121, 192, 255))
                .add_modifier(Modifier::BOLD),
        );

    frame.render_stateful_widget(list, inner, &mut list_state);
}

// ── Status bar ───────────────────────────────────────────────────────────────

fn draw_statusbar(frame: &mut Frame, app: &App, area: Rect) {
    let bar_area = Rect {
        y: area.y + area.height.saturating_sub(1),
        height: 1,
        ..area
    };

    let left = if app.scanning {
        Line::from(vec![Span::styled(" ● SCANNING ", Style::default().fg(Color::Yellow))])
    } else {
        let (total_size, total_items) = app.current_node()
            .map(|t| (fmt_size(t.size), t.items))
            .unwrap_or_else(|| (String::from("0 B"), 0));

        Line::from(vec![
            Span::styled(" Total disk usage: ", Style::default().fg(DIM)),
            Span::styled(total_size, Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("  Items: ", Style::default().fg(DIM)),
            Span::styled(total_items.to_string(), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)),
            Span::styled("  Sorting by: ", Style::default().fg(DIM)),
            Span::styled("size desc", Style::default().fg(ACCENT)),
        ])
    };

    let right = " ↑ / ↓ move  Enter drill  r rescan  ? help  q quit ";

    let bar = Paragraph::new(left).style(Style::default().bg(Color::Rgb(22, 27, 34)).fg(DIM));

    frame.render_widget(bar, bar_area);

    // Right-aligned keybinds
    let right_area = Rect {
        x: bar_area.x + bar_area.width.saturating_sub(right.len() as u16),
        width: right.len() as u16,
        ..bar_area
    };
    frame.render_widget(
        Paragraph::new(right).style(Style::default().fg(DIM).bg(Color::Rgb(22, 27, 34))),
        right_area,
    );
}

// ── Help overlay ─────────────────────────────────────────────────────────────

fn draw_help_overlay(frame: &mut Frame, area: Rect) {
    let popup = centered_rect(50, 60, area);
    frame.render_widget(Clear, popup);

    let text = vec![
        Line::from(Span::styled(
            " RDU — keyboard shortcuts ",
            Style::default().fg(ACCENT).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        key_line("j / ↓", "Move selection down"),
        key_line("k / ↑", "Move selection up"),
        key_line("Enter / →", "Drill into directory"),
        key_line("← / Esc / Backspace", " Go up to parent"),
        key_line("r", "Rescan from root"),
        key_line("?", "Toggle this help"),
        key_line("q", "Quit"),
        Line::from(""),
        Line::from(Span::styled(
            " Press any key to close ",
            Style::default().fg(DIM),
        )),
    ];

    let popup_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(Color::Rgb(22, 27, 34)));

    let p = Paragraph::new(text)
        .block(popup_block)
        .wrap(Wrap { trim: false });

    frame.render_widget(p, popup);
}

fn key_line(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<12}", key), Style::default().fg(ACCENT)),
        Span::styled(desc.to_string(), Style::default().fg(FG)),
    ])
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    format!("{}…", &s[..max.saturating_sub(1)])
}

fn short_path(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    format!("…{}", &s[s.len().saturating_sub(max - 1)..])
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
