use crate::s3objects::{get_objects, S3Result};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::path::Path;
use tui::widgets::TableState;

pub struct StatefulList<S3Result> {
    pub state: TableState,
    pub items: Vec<S3Result>,
    pub num_items_to_display: usize,
    pub bucket: String,
    pub root_path: String,
    pub current_path: String,
    pub prev_path: String,
    pub rt: tokio::runtime::Runtime,
}

fn parse_prev_path(path: &str) -> String {
    return match Path::new(path).parent() {
        Some(p) => match p.to_str().unwrap() {
            "" => "".to_string(),
            other => other.to_string() + "/",
        },
        None => String::from(path),
    };
}

impl StatefulList<S3Result> {
    fn from_path(
        bucket_name: &str,
        path: &Option<String>,
    ) -> Result<StatefulList<S3Result>, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

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

    pub fn copy(&mut self) {
        match self.state.selected() {
            Some(i) => {
                let selected_item = self
                    .items
                    .iter()
                    .filter(|res| res.is_matched)
                    .nth(i)
                    .unwrap();
                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                let uri = [
                    "s3:/".to_string(),
                    self.bucket.to_string(),
                    selected_item.path.to_string(),
                ]
                .join("/");

                ctx.set_contents(uri.to_owned()).unwrap();
            }
            None => {
                return;
            }
        };
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.num_items_to_display - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        match self.state.selected() {
            Some(i) => {
                let pos = match i {
                    0 => self.num_items_to_display - 1,
                    other => other - 1,
                };
                self.state.select(Some(pos))
            }
            None => self.state.select(None),
        };
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }

    pub fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Loads the path that the cursor is currently on
        match self.state.selected() {
            Some(i) => {
                let selected_item = self.items.iter().filter(|res| res.is_matched).nth(i);

                match selected_item {
                    Some(s) => {
                        if s.is_directory {
                            // reset paths
                            self.prev_path = self.current_path.clone();
                            self.current_path = String::from(&s.path);

                            // reset items
                            let new_items = self.rt.block_on(get_objects(&self.bucket, &s.path))?;
                            self.num_items_to_display = new_items.len();
                            self.items = new_items;
                            self.unselect();
                        }
                    }
                    None => (),
                };
            }
            None => (),
        };

        Ok(())
    }

    pub fn goback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // reset paths
        self.current_path = self.prev_path.clone();
        self.prev_path = parse_prev_path(&self.current_path);

        // reset items
        self.items = self
            .rt
            .block_on(get_objects(&self.bucket, &self.current_path))?;
        self.num_items_to_display = self.items.len();

        self.unselect();
        Ok(())
    }

    pub fn reset(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.current_path = self.root_path.clone();
        self.prev_path = parse_prev_path(&self.current_path);
        self.items = self
            .rt
            .block_on(get_objects(&self.bucket, &self.current_path))?;
        self.num_items_to_display = self.items.len();
        self.unselect();

        Ok(())
    }

    pub fn sort_items(&mut self, key: &str, config: &mut SortConfig) {
        let ascending: bool;

        if key == config.sort_key {
            ascending = !config.ascending;
        } else {
            ascending = true;
        }

        match key {
            "path" => match ascending {
                true => self.items.sort_by(|d1, d2| d1.path.cmp(&d2.path)),
                false => self.items.sort_by(|d1, d2| d2.path.cmp(&d1.path)),
            },
            "last_modified" => match ascending {
                true => self
                    .items
                    .sort_by(|d1, d2| d1.last_modified.cmp(&d2.last_modified)),
                false => self
                    .items
                    .sort_by(|d1, d2| d2.last_modified.cmp(&d1.last_modified)),
            },
            _ => (),
        }
        config.ascending = ascending.clone();
        config.sort_key = key.to_string();
    }
}

pub struct SortConfig {
    pub sort_key: String,
    pub ascending: bool,
}

pub enum AppMode {
    FilterMode,
    SortMode,
    RegularMode,
}

pub struct App {
    pub items: StatefulList<S3Result>,
    pub mode: AppMode,
    pub search_input: String,
    pub sort_config: SortConfig,
}

impl App {
    pub fn new(bucket: String, path: Option<String>) -> App {
        App {
            items: StatefulList::from_path(&bucket, &path).unwrap(),
            mode: AppMode::RegularMode,
            search_input: "".to_string(),
            sort_config: SortConfig {
                // default sorting from list_objects_v2
                sort_key: "path".to_string(),
                ascending: true,
            },
        }
    }

    pub fn append_to_search(&mut self, c: char) -> Result<(), Box<dyn std::error::Error>> {
        self.search_input.push(c);
        Ok(())
    }
    pub fn delete_from_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.search_input.pop();
        Ok(())
    }

    pub fn filter_for_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for item in &mut self.items.items {
            if item
                .label
                .to_lowercase()
                .contains(&self.search_input.to_lowercase())
            {
                item.is_matched = true;
            } else {
                item.is_matched = false;
            }
        }

        Ok(())
    }
}
