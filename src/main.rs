use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{backend::CrosstermBackend, Terminal};

use std::{env, io, time::Duration};

mod s3objects;

mod app;
use app::{run_app, App};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bucket = env::var("AWSS3BUCKET").expect("AWSS3BUCKET needs to be defined!");
    let prefix = env::var("AWSS3PREFIX").expect("AWSS3PREFIX needs to be defined!");

    let app = App::new(&bucket, &prefix);
    let tick_rate = Duration::from_millis(250);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    // execute!(stdout, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let _res = run_app(&mut terminal, app, tick_rate);

    disable_raw_mode()?;
    // execute!(terminal.backend_mut(), DisableMouseCapture)?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
