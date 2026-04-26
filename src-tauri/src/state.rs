use anki::collection::Collection;
use std::path::PathBuf;
use tokio::sync::Mutex;

pub struct AppState {
    pub col: Mutex<Option<Collection>>,
    pub col_path: Mutex<Option<PathBuf>>,
    pub http: reqwest::Client,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            col: Mutex::new(None),
            col_path: Mutex::new(None),
            http: reqwest::Client::new(),
        }
    }
}
