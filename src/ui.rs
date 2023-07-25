use crate::app::{App, AppMode};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};
use crate::s3objects::{S3Result, S3Type};

fn fmt_size(size: i64) -> String {
    if size == 0 {
        return "/".to_string();
    } else {
        // TODO: Make this more human readable
        return size.to_string();
    }
}

fn result_style(res: &S3Result) -> Style {
    // TODO: use match here somehow?

    if let S3Type::Directory = res.kind{
        return Style::default();
    } else if res.label.contains(".cloudpickle") || res.label.contains(".pkl"){
        return Style::default().fg(Color::LightMagenta);
    } else if res.label.contains(".parquet"){
        return Style::default().fg(Color::Green);
    } else {
        return Style::default().fg(Color::Yellow);
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Max(1),
                Constraint::Length(3),
                Constraint::Min(10),
            ]
            .as_ref(),
        )
        .split(f.size());

    let msg = match app.mode {
        AppMode::RegularMode => {
            vec![
                Span::raw("Use the arrow keys to navigate. Press "),
                Span::styled("f", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to filter "),
                Span::styled("s", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to sort, "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to copy the obj URI, or "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit."),
            ]
        }
        AppMode::FilterMode => {
            vec![
                Span::raw("Filter Mode: Press "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit filter mode"),
            ]
        }
        AppMode::SortMode => {
            vec![
                Span::raw("Sort Mode: Press "),
                Span::styled("d", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to sort by date (last_modified), or "),
                Span::styled("p", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to sort by path. "),
                Span::raw("Press "),
                Span::styled("ESC", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit sort mode"),
            ]
        }
    };

    let text = Text::from(Spans::from(msg));
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let search = Paragraph::new(app.search.to_string())
        .style(match app.mode {
            AppMode::FilterMode => Style::default().fg(Color::Yellow),
            AppMode::SortMode => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("Search"));

    f.render_widget(search, chunks[1]);

    let items: Vec<Row> = app
        .items
        .iter()
        .map(|res| {
            let style = result_style(&res);
            Row::new(vec![
                Span::styled(res.label.to_string(), style),
                Span::styled(res.last_modified.to_string(), style),
                Span::styled(fmt_size(res.size), style),                
            ])
        })
        .collect();

    let path = app.path.to_string();

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

    f.render_stateful_widget(table, chunks[2], &mut app.state);
}

