use crate::events::Events;
use crate::App;
use crate::AppMode;
use crossterm::event::KeyCode;
use eyre;
use std::time::Duration;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame, Terminal,
};

fn fmt_size(size: i64) -> String {
    if size == 0 {
        return "/".to_string();
    } else {
        // TODO: Make this more human readable
        return size.to_string();
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(3),
                Constraint::Min(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let msg = match app.mode {
        AppMode::RegularMode => {
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled(
                    "or use arrow keys",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to start selecting."),
            ]
        }
        AppMode::FilterMode => {
            vec![
                Span::raw("Filter Mode: Press "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit"),
            ]
        }
        AppMode::SortMode => {
            vec![
                Span::raw("Sort Mode: Press "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to sort by date (last_modified), or "),
                Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to sort by path, "),
                Span::raw("Press "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit"),
            ]
        }
    };

    let text = Text::from(Spans::from(msg));
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let search = Paragraph::new(app.search_input.to_string())
        .style(match app.mode {
            AppMode::FilterMode => Style::default().fg(Color::Yellow),
            AppMode::SortMode => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("Search"));

    f.render_widget(search, chunks[1]);

    let items: Vec<Row> = app
        .items
        .items
        .iter()
        .filter(|res| res.is_matched)
        .map(|res| {
            Row::new(vec![
                res.label.to_string(),
                res.last_modified.to_string(),
                fmt_size(res.size),
            ])
        })
        .collect();

    let path = app.items.current_path.to_string();

    let mut table = Table::new(items)
        .style(Style::default().fg(Color::White))
        .header(
            Row::new(vec!["Path", "Last Modified", "Size"])
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(1),
        )
        .block(Block::default().title(path))
        .widths(&[
            Constraint::Length(50),
            Constraint::Length(15),
            Constraint::Length(5),
        ])
        .column_spacing(10);

    if matches!(app.mode, AppMode::RegularMode) {
        table = table
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
    }

    f.render_stateful_widget(table, chunks[2], &mut app.items.state);
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> eyre::Result<()> {
    let events = Events::new(tick_rate);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        match events.next() {
            Ok(key) => match app.mode {
                AppMode::SortMode => match key.code {
                    KeyCode::Esc => Ok(app.mode = AppMode::RegularMode),
                    KeyCode::Char('p') => Ok(app.items.sort_items("path", &mut app.sort_config)),
                    KeyCode::Char('d') => {
                        Ok(app.items.sort_items("last_modified", &mut app.sort_config))
                    }
                    _ => Ok(()),
                },

                AppMode::FilterMode => match key.code {
                    KeyCode::Backspace => app.delete_from_search(),
                    KeyCode::Char(c) => app.append_to_search(c),
                    KeyCode::Esc => Ok(app.mode = AppMode::RegularMode),
                    KeyCode::Down => Ok(app.mode = AppMode::RegularMode),
                    KeyCode::Enter => app.filter_for_search(),
                    _ => Ok(()),
                },
                AppMode::RegularMode => match key.code {
                    KeyCode::Enter => app.items.refresh(),
                    KeyCode::Left => app.items.goback(),
                    KeyCode::Esc => Ok(app.items.unselect()),
                    KeyCode::Down => Ok(app.items.next()),
                    KeyCode::Up => Ok(app.items.previous()),
                    KeyCode::Char('c') => Ok(app.items.copy()),
                    KeyCode::Char('f') => Ok(app.mode = AppMode::FilterMode),
                    KeyCode::Char('s') => Ok(app.mode = AppMode::SortMode),
                    KeyCode::Char('r') => app.items.reset(),
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => Ok(()),
                },
            },
            Err(err) => Ok(()),
        };
    }
}
