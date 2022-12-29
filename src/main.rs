// TODO: Add some documentation here

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::{CrosstermBackend, Backend}, Terminal};
use eyre;
use std::{io, time::Duration};

mod app;
use app::app::App;
use app::ui::ui;
mod events;
use events::Events;
mod s3objects;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    bucket: String,
    #[arg(short, long)]
    prefix: Option<String>,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let app = App::new(args.bucket, args.prefix);
    let tick_rate = Duration::from_millis(250);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let _res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
