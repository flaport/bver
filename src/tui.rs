use std::io::{self, stdout};
use std::path::PathBuf;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

/// A proposed change to a file
#[derive(Clone)]
pub struct ProposedChange {
    pub path: PathBuf,
    pub line_idx: usize,
    pub old_line: String,
    pub new_line: String,
    pub context_before: Vec<String>,
    pub context_after: Vec<String>,
    pub selected: bool,
}

/// Run the TUI to select which changes to apply
/// Returns the indices of selected changes
pub fn select_changes(changes: &mut [ProposedChange]) -> io::Result<bool> {
    if changes.is_empty() {
        return Ok(true);
    }

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut state = ListState::default();
    state.select(Some(0));

    let result = run_tui(&mut terminal, changes, &mut state);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_tui(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    changes: &mut [ProposedChange],
    state: &mut ListState,
) -> io::Result<bool> {
    loop {
        terminal.draw(|frame| draw(frame, changes, state))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                KeyCode::Enter => return Ok(true),
                KeyCode::Up | KeyCode::Char('k') => {
                    let i = state.selected().unwrap_or(0);
                    let new_i = if i == 0 { changes.len() - 1 } else { i - 1 };
                    state.select(Some(new_i));
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let i = state.selected().unwrap_or(0);
                    let new_i = if i >= changes.len() - 1 { 0 } else { i + 1 };
                    state.select(Some(new_i));
                }
                KeyCode::Char(' ') => {
                    if let Some(i) = state.selected() {
                        changes[i].selected = !changes[i].selected;
                    }
                }
                KeyCode::Char('a') => {
                    // Select all
                    for change in changes.iter_mut() {
                        change.selected = true;
                    }
                }
                KeyCode::Char('n') => {
                    // Deselect all
                    for change in changes.iter_mut() {
                        change.selected = false;
                    }
                }
                _ => {}
            }
        }
    }
}

fn draw(frame: &mut Frame, changes: &[ProposedChange], state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),
            Constraint::Length(10),
            Constraint::Length(1),
        ])
        .split(frame.area());

    // Changes list
    let items: Vec<ListItem> = changes
        .iter()
        .map(|change| {
            let checkbox = if change.selected { "[x]" } else { "[ ]" };
            let path = change.path.to_string_lossy();
            let line_num = change.line_idx + 1;
            ListItem::new(format!("{} {}:{}", checkbox, path, line_num))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(" Changes (space: toggle, a: all, n: none) "))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[0], state);

    // Preview pane
    if let Some(i) = state.selected() {
        let change = &changes[i];
        let mut preview_lines: Vec<Line> = Vec::new();

        let start_line = change.line_idx.saturating_sub(change.context_before.len());

        // Context before
        for (offset, line) in change.context_before.iter().enumerate() {
            let line_num = start_line + offset + 1;
            preview_lines.push(Line::from(vec![
                Span::styled(format!("  {:4} │ ", line_num), Style::default().fg(Color::DarkGray)),
                Span::raw(line),
            ]));
        }

        // Old line (red)
        let line_num = change.line_idx + 1;
        preview_lines.push(Line::from(vec![
            Span::styled(format!("- {:4} │ ", line_num), Style::default().fg(Color::Red)),
            Span::styled(&change.old_line, Style::default().fg(Color::Red)),
        ]));

        // New line (green)
        preview_lines.push(Line::from(vec![
            Span::styled(format!("+ {:4} │ ", line_num), Style::default().fg(Color::Green)),
            Span::styled(&change.new_line, Style::default().fg(Color::Green)),
        ]));

        // Context after
        for (offset, line) in change.context_after.iter().enumerate() {
            let line_num = change.line_idx + 2 + offset;
            preview_lines.push(Line::from(vec![
                Span::styled(format!("  {:4} │ ", line_num), Style::default().fg(Color::DarkGray)),
                Span::raw(line),
            ]));
        }

        let preview = Paragraph::new(preview_lines)
            .block(Block::default().borders(Borders::ALL).title(" Preview "))
            .wrap(Wrap { trim: false });

        frame.render_widget(preview, chunks[1]);
    }

    // Help line
    let help = Paragraph::new(" ↑↓/jk: navigate │ space: toggle │ a: all │ n: none │ enter: apply │ q/esc: cancel ");
    frame.render_widget(help, chunks[2]);
}
