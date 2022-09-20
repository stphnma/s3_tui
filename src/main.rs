use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3 as s3;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use std::{
    io,
    time::{Duration, Instant},
    env,
};
use std::error::Error;

struct S3Object{
    path: String,
    label: String,
    is_directory: bool,
}

impl S3Object{
    fn new(path: String) -> S3Object {
        let path_parts = path.split("/").collect::<Vec<&str>>();
        let is_directory = match path.chars().last().unwrap() {
            '/' => true,
            _ => false
        };

        let label = match is_directory{
            true => path_parts[path_parts.len() - 2 .. path_parts.len()].join("/").to_string(),
            false => path_parts.last().unwrap().to_string()
        };

        S3Object{
            path: path.clone(), 
            label: label,
            is_directory: is_directory
        }
    }
}


async fn get_results(
    client: &s3::Client,
    bucket_name: String,
    prefix: String
) -> Result<Vec<S3Object>, s3::Error> {
    let objects = client
        .list_objects_v2()
        .bucket(bucket_name)
        .prefix(prefix)
        .max_keys(200)
        .delimiter("/")
        .send().await?;

    let mut result: Vec<S3Object> = Vec::new();

    for obj in objects.common_prefixes().unwrap_or_default() {
        result.push(S3Object::new(String::from(obj.prefix().unwrap())));
    }
    for obj in objects.contents().unwrap_or_default() {
        result.push(S3Object::new(String::from(obj.key().unwrap())));
    }
    return Ok(result);
}

struct StatefulList<S3Object> {
    state: ListState,
    items: Vec<S3Object>,
}


impl StatefulList<S3Object> {
    fn with_items(items: Vec<S3Object>) -> StatefulList<S3Object> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
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
            let path = &self.items[i].path;
            let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()?;
            
    
            self.items = rt.block_on(get_objects(path))?;
    
        }


        Ok(())

    }

}

struct App {
    items: StatefulList<S3Object>
}

impl App {
    fn new(s3_list: Vec<S3Object>) -> App {
        App {
            items: StatefulList::with_items(s3_list)
        }
    }
}


fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()>  {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout).expect("Some error message")  {
            if let Event::Key(key) = event::read().expect("Some error message") {
                match key.code {
                    KeyCode::Enter => app.items.refresh(),
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => Ok(app.items.unselect()),
                    KeyCode::Down => Ok(app.items.next()),
                    KeyCode::Up => Ok(app.items.previous()),
                    _ => Ok(())
                };
            }
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
    let items: Vec<ListItem> = app
        .items
        .items
        .iter()
        .map(|res| {
            ListItem::new(Text::from(res.path.as_str())).style(Style::default())
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("List"))
        .highlight_style(
            Style::default()
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.items.state);
}

async fn get_objects(bucket_name: &str, path : &str) -> Result<Vec<S3Object>, s3::Error>{
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = s3::Client::new(&config);

    let bucket_name = String::from(bucket);
    let prefix = String::from(path);
    let results = get_results(&client, bucket_name, prefix).await?;

    Ok(results)

}


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    // Call the asynchronous connect method using the runtime.

    let bucket = env::var("AWSS3BUCKET").expect("AWSS3BUCKET needs to be defined!")
    let prefix = env::var("AWSS3PREFIX").expect("AWSS3PREFIX needs to be defined!")

    let results = rt.block_on(get_objects(bucket, prefix))?;

    // for res in &results {
    //     println!("Object {}", res.label);
    // }


    let tick_rate = Duration::from_millis(250);

    let app = App::new(results);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, app, tick_rate);


    // for res in results {
    //     match res {
    //         S3File::S3OFileObject { name } => println!("Object {}", name),
    //         S3File::S3FileDirectory { name } => println!("Folder {}", name),
    //         // _ => println!("Something else"),
    //     }
    // }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // if let Err(err) = res {
    //     println!("{:?}", err)
    // }

    Ok(())
}