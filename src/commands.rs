use crate::app::{App, AppMode};
use std::time::Duration;
use eyre;
use tui::{Terminal, backend::Backend};
use crate::events::Events;
use crossterm::event::KeyCode;

use crate::ui::ui;

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
                    KeyCode::Right => app.items.refresh(),
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
            Err(_err) => {
                println!("{:?}", _err);
                return Ok(());
            }
        };
    }
}
