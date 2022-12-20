mod s3objects;
use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use crossterm::event::{ self, Event, KeyCode, KeyEvent };
use eyre;
use s3objects::{ get_objects, S3Result };
use std::{ time::{ Duration } };
use tui::{
    backend::Backend,
    layout::{ Constraint, Direction, Layout },
    style::{ Color, Modifier, Style },
    widgets::{ Block, Borders, Paragraph, Row, Table, TableState },
    Frame,
    Terminal,
};
mod events;
use events::Events;
use std::path::Path;

struct StatefulList<S3Result> {
    state: TableState,
    items: Vec<S3Result>,
    num_items_to_display: usize,
    bucket: String,
    root_path: String,
    current_path: String,
    prev_path: String,
    rt: tokio::runtime::Runtime,
}

fn parse_prev_path(path: &str) -> String {
    return match Path::new(path).parent() {
        Some(p) =>
            match p.to_str().unwrap() {
                "" => "".to_string(),
                other => other.to_string() + "/",
            }
        None => String::from(path),
    };
}

impl StatefulList<S3Result> {
    fn from_path(
        bucket_name: &str,
        path: &Option<String>
    ) -> Result<StatefulList<S3Result>, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;

        let path = match path {
            Some(p) => p,
            None => "",
        };

        let items = rt.block_on(get_objects(bucket_name, path))?;
        let prev_path = parse_prev_path(path);

        Ok(StatefulList {
            state: TableState::default(),
            num_items_to_display: items.len(),
            items: items,
            bucket: String::from(bucket_name),
            root_path: String::from(path),
            current_path: String::from(path),
            prev_path: prev_path,
            rt: rt,
        })
    }

    fn copy(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                let selected_item = self.items
                    .iter()
                    .filter(|res| res.is_matched)
                    .nth(i)
                    .unwrap();
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                let uri = [
                    "s3:/".to_string(),
                    self.bucket.to_string(),
                    selected_item.path.to_string(),
                ].join("/");

                ctx.set_contents(uri.to_owned()).unwrap();
            }
            None => {
                return;
            }
        };
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.num_items_to_display - 1 { 0 } else { i + 1 }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
       let j = match self.state.selected() {
            Some(i) => {
                let pos = match i {
                    0 => self.num_items_to_display - 1,
                    other => other - 1,
                };
                // if i == 0 { self.num_items_to_display - 1 } else { i - 1 }
                self.state.select(Some(pos))
            },
            None => {
                self.state.select(None)
            },
        };
        
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }

    fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Loads the path that the cursor is currently on
        match self.state.selected() {
            Some(i) => {
                let selected_item = self.items
                    .iter()
                    .filter(|res| res.is_matched)
                    .nth(i);
                
                match selected_item {
                    Some(s) => {
                        if s.is_directory {
                            // reset paths
                            self.prev_path = self.current_path.clone();
                            self.current_path = String::from(&s.path);
        
                            // reset items
                            let new_items = self.rt.block_on(
                                get_objects(&self.bucket, &s.path)
                            )?;
                            self.num_items_to_display = new_items.len();
                            self.items = new_items;
                            self.unselect();
                        }
        
                    },
                    None => ()
                };
            },
            None => (),
        };

        Ok(())
    }

    fn goback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // reset paths
        self.current_path = self.prev_path.clone();
        self.prev_path = parse_prev_path(&self.current_path);

        // reset items
        self.items = self.rt.block_on(get_objects(&self.bucket, &self.current_path))?;
        self.num_items_to_display = self.items.len();

        self.unselect();
        Ok(())
    }

    fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_path = self.root_path.clone();
        self.prev_path = parse_prev_path(&self.current_path);
        self.items = self.rt.block_on(get_objects(&self.bucket, &self.current_path))?;
        self.num_items_to_display = self.items.len();
        self.unselect();

        Ok(())
    }
}

pub struct App {
    items: StatefulList<S3Result>,
    is_in_edit_mode: bool,
    search_input: String,
}

impl App {
    pub fn new(bucket: String, path: Option<String>) -> App {
        App {
            items: StatefulList::from_path(&bucket, &path).unwrap(),
            is_in_edit_mode: false,
            search_input: "".to_string(),
        }
    }

    fn append_to_search(&mut self, c: char) -> Result<(), Box<dyn std::error::Error>> {
        self.search_input.push(c);
        Ok(())
    }
    fn delete_from_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.search_input.pop();
        Ok(())
    }

    fn filter_for_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for item in &mut self.items.items {
            if item.label.to_lowercase().contains(&self.search_input.to_lowercase()) {
                item.is_matched = true;
            } else {
                item.is_matched = false;
            }
        }

        Ok(())
    }
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
                match app.is_in_edit_mode {
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
                                    app.is_in_edit_mode = true;
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
                                    app.is_in_edit_mode = false;
                                }),
                            KeyCode::Down =>
                                Ok({
                                    app.is_in_edit_mode = false;
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

fn fmt_size(size: i64) -> String {
    if size == 0 {
        return "/".to_string();
    } else {
        // TODO: Make this more human readable
        return size.to_string();
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(10)].as_ref())
        .split(f.size());

    let search = Paragraph::new(app.search_input.to_string())
        .style(match app.is_in_edit_mode {
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

    if !app.is_in_edit_mode {
        table = table
            .highlight_style(Style::default().bg(Color::LightGreen).add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");
    }

    f.render_stateful_widget(table, chunks[1], &mut app.items.state);
}