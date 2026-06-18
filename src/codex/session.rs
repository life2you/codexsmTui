use std::path::PathBuf;

use chrono::{DateTime, Local};

pub const UNKNOWN_PROJECT: &str = "Unknown Project";
pub const UNTITLED_SESSION: &str = "Untitled Session";

#[derive(Clone, Debug)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub project_path: String,
    pub file_path: PathBuf,
    pub created_at: Option<DateTime<Local>>,
    pub updated_at: Option<DateTime<Local>>,
    pub size: u64,
    pub selected: bool,
}

impl Session {
    pub fn updated_sort_key(&self) -> i64 {
        self.updated_at
            .map(|dt| dt.timestamp())
            .or_else(|| self.created_at.map(|dt| dt.timestamp()))
            .unwrap_or_default()
    }

    pub fn search_blob(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.title,
            self.project_path,
            self.id,
            self.file_path.display()
        )
        .to_lowercase()
    }
}

#[derive(Clone, Debug)]
pub struct Project {
    pub path: String,
    pub session_count: usize,
}

#[derive(Clone, Debug)]
pub struct MessageSnippet {
    pub role: String,
    pub timestamp: Option<DateTime<Local>>,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct SessionDetail {
    pub session: Session,
    pub recent_messages: Vec<MessageSnippet>,
}
