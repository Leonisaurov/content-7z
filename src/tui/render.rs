//! Drawing of the interface.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::prelude::{Frame, Line, Span, Style};
use ratatui::widgets::Row as TableRow;
use ratatui::widgets::{
    Block, Borders, Cell, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
    Wrap,
};

use crate::cli::PROGRAM;
use crate::tui::app::{App, Dialog, Level};
use crate::util;

/// Renders the whole interface.
pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();
    if area.width < 12 || area.height < 6 {
        frame.render_widget(
            Paragraph::new("The terminal is too small.").style(app.theme.base()),
            area,
        );
        return;
    }

    let chunks = areas(area);

    render_header(frame, app, chunks[0]);
    render_list(frame, app, chunks[1]);
    render_footer(frame, app, chunks[2]);

    if let Some(dialog) = app.dialog.clone() {
        render_dialog(frame, app, &dialog, area);
    }
}

/// Splits the screen into the header, the list and the status bar.
pub fn areas(area: Rect) -> [Rect; 3] {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(area);

    [chunks[0], chunks[1], chunks[2]]
}

/// Draws the header with the archive name and the current path.
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let archive = util::file_name_of(&app.archive.path);

    let mut title = Line::from(vec![
        Span::styled(
            format!(" {PROGRAM} "),
            Style::default()
                .fg(theme.accent)
                .bg(theme.background)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        Span::styled(
            archive.clone(),
            Style::default().fg(theme.text).bg(theme.background),
        ),
    ]);
    title = title.left_aligned();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .style(theme.base())
        .title(title)
        .title_bottom(
            Line::from(format!(" {} ", app.breadcrumb()))
                .left_aligned()
                .style(Style::default().fg(theme.dim).bg(theme.background)),
        );

    let path = Paragraph::new(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            app.breadcrumb(),
            Style::default()
                .fg(theme.text)
                .bg(theme.background)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
    ]))
    .style(theme.base());

    frame.render_widget(path.block(block), area);
}

/// Draws the list of entries.
fn render_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let theme = &app.theme;
    let title = match app.path().last() {
        Some(name) => format!(" {name} "),
        None => format!(" {} ", util::file_name_of(&app.archive.path)),
    };

    let total = app.rows().len();
    let selected = app.state.selected().map(|index| index + 1).unwrap_or(0);
    let counter = if total == 0 {
        String::from("empty")
    } else {
        format!(" {selected}/{total} ")
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border())
        .style(theme.base())
        .title(title)
        .title_bottom(
            Line::from(counter)
                .right_aligned()
                .style(Style::default().fg(theme.dim).bg(theme.background)),
        );

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if app.rows().is_empty() {
        let message = if app.filter.query.is_empty() {
            "This directory is empty."
        } else {
            "No entry matches the filter."
        };
        frame.render_widget(
            Paragraph::new(message).style(Style::default().fg(theme.dim).bg(theme.background)),
            inner,
        );
        return;
    }

    let mut widths = vec![Constraint::Min(10)];
    if app.config.show_size {
        widths.push(Constraint::Length(12));
    }

    let rows: Vec<TableRow> = app
        .rows()
        .iter()
        .map(|row| {
            let (bullet, bullet_color) = if row.is_dir {
                (theme.folder_bullet.clone(), theme.folder_bullet_color)
            } else {
                (theme.file_bullet.clone(), theme.file_bullet_color)
            };

            let mut spans = vec![
                Span::styled(bullet, Style::default().fg(bullet_color)),
                Span::styled(row.name.clone(), Style::default().fg(theme.text)),
            ];
            if row.encrypted {
                spans.push(Span::styled(
                    " (encrypted)",
                    Style::default().fg(theme.warning),
                ));
            }

            let mut cells = vec![Cell::from(Line::from(spans))];
            if app.config.show_size {
                let size = if row.is_dir {
                    String::from("-")
                } else {
                    util::human_size(row.size)
                };
                cells.push(
                    Cell::from(Line::from(size).right_aligned())
                        .style(Style::default().fg(theme.dim)),
                );
            }

            TableRow::new(cells)
        })
        .collect();

    let table = Table::new(rows, widths)
        .row_highlight_style(theme.selection())
        .highlight_symbol("▌ ");

    frame.render_stateful_widget(table, inner, &mut app.state);

    let visible = inner.height as usize;
    if total > visible {
        let mut scrollbar_state =
            ScrollbarState::new(total.saturating_sub(1)).position(app.state.offset());
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(Some("│"))
            .thumb_symbol("█")
            .style(Style::default().fg(theme.accent).bg(theme.background));

        frame.render_stateful_widget(scrollbar, inner, &mut scrollbar_state);
    }
}

/// Draws the status bar.
fn render_footer(frame: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    if let Some(status) = &app.status {
        let color = match status.level {
            Level::Info => theme.text,
            Level::Warning => theme.warning,
            Level::Error => theme.error,
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                format!(" {} ", status.text),
                Style::default().fg(color).bg(theme.background),
            ))),
            area,
        );
        return;
    }

    let mut spans: Vec<Span> = Vec::new();
    if app.filter.editing {
        spans.push(Span::styled(
            " filter: ",
            Style::default().fg(theme.accent).bg(theme.background),
        ));
        spans.push(Span::styled(
            app.filter.query.clone(),
            Style::default()
                .fg(theme.text)
                .bg(theme.background)
                .add_modifier(ratatui::style::Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "_",
            Style::default().fg(theme.accent).bg(theme.background),
        ));
    } else {
        for (key, description) in [
            ("↑↓", "move"),
            ("⏎", "open"),
            ("o", "edit"),
            ("x", "extract"),
            ("/", "filter"),
            ("?", "help"),
            ("q", "quit"),
        ] {
            spans.push(Span::styled(
                format!(" {key} "),
                Style::default()
                    .fg(theme.accent)
                    .bg(theme.background)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!("{description}  "),
                Style::default().fg(theme.dim).bg(theme.background),
            ));
        }
    }

    frame.render_widget(Paragraph::new(Line::from(spans)).style(theme.base()), area);
}

/// Draws a modal dialog on top of the interface.
fn render_dialog(frame: &mut Frame, app: &App, dialog: &Dialog, area: Rect) {
    let theme = &app.theme;

    let (title, body, accent) = match dialog {
        Dialog::Message { title, text, level } => {
            let color = match level {
                Level::Info => theme.accent,
                Level::Warning => theme.warning,
                Level::Error => theme.error,
            };
            (title.clone(), text.clone(), color)
        }
        Dialog::Confirm { title, text, .. } => (
            title.clone(),
            format!(
                "{text}\n\n{}",
                app.config
                    .multi_choice_dialog_helper
                    .as_deref()
                    .unwrap_or("y(es) / n(o)")
            ),
            theme.accent,
        ),
        Dialog::Input {
            title,
            prompt,
            value,
            masked,
            ..
        } => {
            let shown = if *masked {
                "*".repeat(value.chars().count())
            } else {
                value.clone()
            };
            (title.clone(), format!("{prompt}\n\n{shown}█"), theme.accent)
        }
        Dialog::Help => (String::from("Help"), help_text(), theme.accent),
    };

    let lines = body.lines().count().max(1) as u16;
    let width = body
        .lines()
        .map(|line| line.chars().count() as u16)
        .max()
        .unwrap_or(20)
        .max(title.chars().count() as u16 + 4)
        .min(area.width.saturating_sub(4))
        .max(20);
    let height = (lines + 4).min(area.height.saturating_sub(2)).max(5);

    let popup = centered(width, height, area);
    frame.render_widget(Clear, popup);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(accent).bg(theme.background))
        .style(theme.base())
        .title(format!(" {title} "));

    let paragraph = Paragraph::new(body)
        .block(block)
        .wrap(Wrap { trim: false })
        .style(theme.base());

    frame.render_widget(paragraph, popup);
}

/// The content of the help dialog.
fn help_text() -> String {
    let entries = [
        ("↑ / ↓", "move the selection"),
        ("page up / page down", "move ten positions"),
        ("home / end", "first or last entry"),
        ("enter / →", "open the selected folder"),
        ("← / backspace / -", "go back to the parent folder"),
        ("o", "open the file with the editor"),
        ("x", "extract the whole archive"),
        ("/", "filter the entries by name"),
        ("r", "reload the archive"),
        ("?", "show this help"),
        ("q / esc", "quit"),
    ];

    let mut text = String::new();
    for (key, description) in entries {
        text.push_str(&format!("{key:<20}{description}\n"));
    }
    text
}

/// Centers a box of the given size inside an area.
fn centered(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    let x = area.x + (area.width.saturating_sub(width)) / 2;
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(x, y, width, height)
}
