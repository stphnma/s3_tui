use crate::s3objects::{get_objects, S3Result, S3Type};
use clipboard::{ClipboardContext, ClipboardProvider};
use std::path::Path;
use tui::widgets::TableState;

fn new_list_from_path(
    rt: &tokio::runtime::Runtime, 
    bucket_name: &String, 
    path: &Option<String>
) -> Result<Vec<S3Result>, Box<dyn std::error::Error>>  {
    // Query S3 API for a new list of items
    let path = match path {
        Some(p) => p,
        None => "",
    };
    let objects = rt.block_on(get_objects(bucket_name, path))?;

    let mut items: Vec<S3Result> = Vec::new();
    for obj in objects{
        items.push(obj);
    }

    Ok(items)
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
    pub state: TableState,
    pub items: Vec<S3Result>,
    pub num_items_to_display: usize,
    pub runtime: tokio::runtime::Runtime,
    pub sort_config: SortConfig,
    pub search: String,
    pub bucket: String,
    pub path: String,
    pub mode: AppMode,
}

impl App {
    
    pub fn new(bucket: &String, path: &Option<String>) ->  Result<App, Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        
        let items = new_list_from_path(&rt, bucket, path).unwrap();

        let search = match path {
            Some(p) => p,
            None => "",
        };

        let item_length = items.len();

        Ok(App {
            items: items,
            state: TableState::default(),
            runtime: rt,
            sort_config: SortConfig {
                // default sorting from list_objects_v2
                sort_key: "path".to_string(),
                ascending: true,
            },
            search: search.to_string(),
            bucket: bucket.to_string(),
            path: search.to_string(),
            num_items_to_display: item_length,
            mode: AppMode::RegularMode,
        })
    }
    pub fn refresh(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        // Refresh s3 view given current bucket and path
        self.items = new_list_from_path(&self.runtime, &self.bucket, &Some(self.path.to_string()))?;
        self.num_items_to_display = self.items.len();
        self.unselect();
        Ok(())
    }

    pub fn search(&mut self) {
        self.path = self.search.to_string();
        self.refresh();
    }

    pub fn go_to_selected(&mut self){
        match self.state.selected(){
            Some(i) => {
                let selected = self.items.iter().nth(i);
                match selected {
                    Some(i) => {
                        match i.kind {
                            S3Type::Directory => { 
                                self.search = i.path.to_string();
                                self.path = i.path.to_string();
                            },
                            _ => (),
                        };
                    }
                    None => ()
                };
            }
            None => (),
        }

    }



    // pub fn goback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    pub fn copy(&mut self) {
        // Copy URI of selected item
        match self.state.selected() {
            Some(i) => {
                let selected_item = self
                    .items
                    .iter()
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
        // scroll down
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

    pub fn append_to_search(&mut self, c: char) -> Result<(), Box<dyn std::error::Error>> {
        self.search.push(c);
        Ok(())
    }

    pub fn delete_from_search(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.search.pop();
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
        };
        //config.ascending = ascending.clone();
        //config.sort_key = key.to_string();     
    }
}



