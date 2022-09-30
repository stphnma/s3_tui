use crossterm::{
    event::{ self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use tui::{
    backend::{ Backend, CrosstermBackend },
    layout::{ Constraint, Corner, Direction, Layout },
    style::{ Color, Modifier, Style },
    text::{ Span, Spans, Text },
    widgets::{ Block, Borders, List, ListItem, ListState },
    Frame,
    Terminal,
};

use std::{ io, time::{ Duration, Instant }, env, error::Error };

use crate::s3objects::{ S3Object, get_objects };

struct StatefulList<S3Object> {
    state: ListState,
    items: Vec<S3Object>,
    bucket: String,
    root_path: String,
    current_path: String,
    prev_path: String,
}

fn parse_prev_path(path: &str) -> String {
    let path_parts = path.split("/").collect::<Vec<&str>>();

    if path_parts.len() >= 2 {
        return path_parts[0..path_parts.len() - 2].join("/").to_string() + "/";
    } else {
        return env::var("AWSS3PREFIX").expect("AWSS3PREFIX needs to be defined!");
        //return path.to_string()
    }

    // let prev_path = path_parts[0 .. path_parts.len()-2].join("/").to_string() + "/";

    // prev_path
}

impl StatefulList<S3Object> {
    fn from_path(
        bucket_name: &str,
        path: &str
    ) -> Result<StatefulList<S3Object>, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        let items = rt.block_on(get_objects(bucket_name, path))?;
        let prev_path = parse_prev_path(path);

        // println!("path is {}", path);
        // println!("prev_path is {}", prev_path);

        Ok(StatefulList {
            state: ListState::default(),
            items: items,
            bucket: String::from(bucket_name),
            root_path: String::from(path),
            current_path: String::from(path),
            prev_path: prev_path,
        })
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 { self.items.len() - 1 } else { i - 1 }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }

    fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let i = self.state.selected().unwrap();

        if self.items[i].is_directory {
            let path = self.items[i].path.to_string();
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

            self.items = rt.block_on(get_objects(&self.bucket, &path))?;
            // reset paths
            self.prev_path = self.current_path.clone();
            self.current_path = String::from(path);
        }
        Ok(())
    }

    fn back(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        self.items = rt.block_on(get_objects(&self.bucket, &self.prev_path))?;

        // reset paths
        self.current_path = self.prev_path.clone();
        self.prev_path = parse_prev_path(&self.current_path);
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        self.current_path = self.root_path.clone();
        self.prev_path = parse_prev_path(&self.current_path);
        self.items = rt.block_on(get_objects(&self.bucket, &self.current_path))?;
        Ok(())
    }
}

pub struct App {
    items: StatefulList<S3Object>,
}

impl App {
    pub fn new(bucket: &str, path: &str) -> App {
        App {
            items: StatefulList::from_path(bucket, path).unwrap(),
        }
    }
}

pub fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout).expect("Some error message") {
            if let Event::Key(key) = event::read().expect("Some error message") {
                match key.code {
                    KeyCode::Enter => app.items.refresh(),
                    KeyCode::Left => app.items.back(),
                    KeyCode::Esc => Ok(app.items.unselect()),
                    KeyCode::Down => Ok(app.items.next()),
                    KeyCode::Up => Ok(app.items.previous()),
                    KeyCode::Char('r') => app.items.reset(),
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => Ok(()),
                };
            };
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(90)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app.items.items
        .iter()
        .map(|res| { ListItem::new(Text::from(res.label.as_str())).style(Style::default()) })
        .collect();

    let title = app.items.current_path.to_string();
    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().bg(Color::LightGreen).add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);
}