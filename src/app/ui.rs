
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{ Constraint, Direction, Layout },
    style::{ Color, Modifier, Style },
    widgets::{ Block, Borders, Paragraph, Row, Table },
    Frame,
    Terminal
};
use eyre;
use std::time::Duration;
use crate::App;
use crate::events::Events;
use crossterm::event::KeyCode;

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
        .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
        .split(f.size());

    let search = Paragraph::new(app.search_input.to_string())
        .style(match app.is_in_filter_mode {
            false => Style::default(),
            true => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Search"));

    f.render_widget(search, chunks[0]);

    let items: Vec<Row> = app.items.items
        .iter()
        .filter(|res| res.is_matched)
        .map(|res| {
            Row::new(vec![res.label.to_string(), res.last_modified.to_string(), fmt_size(res.size)])
        })
        .collect();

    let path = app.items.current_path.to_string();

    let mut table = Table::new(items)
        .style(Style::default().fg(Color::White))
        .header(
            Row::new(vec!["Path", "Last Modified", "Size"])
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(1)
        )
        .block(Block::default().title(path))
        .widths(&[Constraint::Length(50), Constraint::Length(15), Constraint::Length(5)])
        .column_spacing(10);

    if !app.is_in_filter_mode {
        table = table
            .highlight_style(Style::default().bg(Color::LightGreen).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
    }

    f.render_stateful_widget(table, chunks[1], &mut app.items.state);
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration
) -> eyre::Result<()> {
    let events = Events::new(tick_rate);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;
        match events.next() {
            Ok(key) => {
                // TODO: Add sort mode
                match app.is_in_filter_mode {
                    false =>
                        match key.code {
                            KeyCode::Enter => app.items.refresh(),
                            KeyCode::Left => app.items.goback(),
                            KeyCode::Esc => Ok(app.items.unselect()),
                            KeyCode::Down => Ok(app.items.next()),
                            KeyCode::Up => Ok(app.items.previous()),
                            KeyCode::Char('c') => Ok(app.items.copy()),
                            KeyCode::Char('e') =>
                                Ok({
                                    app.is_in_filter_mode = true;
                                }),
                            KeyCode::Char('r') => app.items.reset(),
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => Ok(()),
                        }

                    true =>
                        match key.code {
                            KeyCode::Backspace => app.delete_from_search(),
                            KeyCode::Char(c) => app.append_to_search(c),
                            KeyCode::Esc =>
                                Ok({
                                    app.is_in_filter_mode = false;
                                }),
                            KeyCode::Down =>
                                Ok({
                                    app.is_in_filter_mode = false;
                                }),
                            KeyCode::Enter => app.filter_for_search(),
                            _ => Ok(()),
                        }
                };
            }
            Err(err) => (),
        };
    }
}