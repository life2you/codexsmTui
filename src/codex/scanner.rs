use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::codex::{
    parser::parse_session_preview,
    session::{Project, Session},
};

#[derive(Debug)]
pub struct ScanResult {
    pub sessions: Vec<Session>,
    pub warnings: Vec<String>,
    pub root_exists: bool,
}

pub fn default_session_root() -> Result<PathBuf> {
    let home = dirs::home_dir().context("unable to resolve home directory")?;
    Ok(home.join(".codex").join("sessions"))
}

pub fn default_session_root_label() -> &'static str {
    "~/.codex/sessions"
}

pub fn scan_sessions(root: &std::path::Path) -> ScanResult {
    if !root.exists() {
        return ScanResult {
            sessions: Vec::new(),
            warnings: Vec::new(),
            root_exists: false,
        };
    }

    let mut sessions = Vec::new();
    let mut warnings = Vec::new();

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                warnings.push(format!("Walk error: {error}"));
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("jsonl") {
            continue;
        }

        match parse_session_preview(entry.path()) {
            Ok(session) => sessions.push(session),
            Err(error) => warnings.push(format!("{}: {error}", entry.path().display())),
        }
    }

    sessions.sort_by(|left, right| {
        right
            .updated_sort_key()
            .cmp(&left.updated_sort_key())
            .then_with(|| left.file_path.cmp(&right.file_path))
    });

    ScanResult {
        sessions,
        warnings,
        root_exists: true,
    }
}

pub fn build_projects(sessions: &[Session]) -> Vec<Project> {
    let mut counts = BTreeMap::<String, usize>::new();
    for session in sessions {
        *counts.entry(session.project_path.clone()).or_insert(0) += 1;
    }

    let mut projects = Vec::with_capacity(counts.len() + 1);
    projects.push(Project {
        path: "All Sessions".to_string(),
        session_count: sessions.len(),
    });
    projects.extend(counts.into_iter().map(|(path, session_count)| Project {
        path,
        session_count,
    }));
    projects
}
