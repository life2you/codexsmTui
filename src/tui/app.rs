use std::path::PathBuf;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::ListState;

use crate::codex::{
    ScanResult, Session, SessionDetail, build_projects, default_session_root_label, scan_sessions,
    trash::move_session_to_trash,
};
use crate::tui::detail::load_detail;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Focus {
    Projects,
    Sessions,
}

#[derive(Clone, Debug)]
pub struct DeleteRequest {
    pub session_indices: Vec<usize>,
    pub prompt: String,
}

pub struct App {
    pub scan_root: PathBuf,
    pub scan_root_label: &'static str,
    pub root_exists: bool,
    pub sessions: Vec<Session>,
    pub projects: Vec<crate::codex::Project>,
    pub filtered_session_indices: Vec<usize>,
    pub focus: Focus,
    pub project_state: ListState,
    pub session_state: ListState,
    pub search_mode: bool,
    pub search_query: String,
    pub show_help: bool,
    pub detail: Option<SessionDetail>,
    pub confirm_delete: Option<DeleteRequest>,
    pub status: String,
    pub warnings: Vec<String>,
    pub should_quit: bool,
}

impl App {
    pub fn new(scan_root: PathBuf, scan: ScanResult) -> Self {
        let mut app = Self {
            scan_root,
            scan_root_label: default_session_root_label(),
            root_exists: scan.root_exists,
            sessions: scan.sessions,
            projects: Vec::new(),
            filtered_session_indices: Vec::new(),
            focus: Focus::Sessions,
            project_state: ListState::default(),
            session_state: ListState::default(),
            search_mode: false,
            search_query: String::new(),
            show_help: false,
            detail: None,
            confirm_delete: None,
            status: String::new(),
            warnings: scan.warnings,
            should_quit: false,
        };
        app.project_state.select(Some(0));
        app.rebuild_views();
        app.status = if app.root_exists {
            format!("Loaded {} sessions", app.sessions.len())
        } else {
            format!("No Codex sessions found at {}", app.scan_root_label)
        };
        app
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if let Some(confirm) = self.confirm_delete.clone() {
            self.handle_confirm_key(key, confirm);
            return;
        }

        if self.show_help {
            match key.code {
                KeyCode::Char('q') => self.should_quit = true,
                KeyCode::Esc | KeyCode::Char('?') => self.show_help = false,
                _ => {}
            }
            return;
        }

        if self.detail.is_some() {
            self.handle_detail_key(key);
            return;
        }

        if self.search_mode {
            self.handle_search_key(key);
            return;
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('?') => self.show_help = !self.show_help,
            KeyCode::Tab => self.toggle_focus(),
            KeyCode::Up => self.move_up(),
            KeyCode::Down => self.move_down(),
            KeyCode::Enter => self.open_detail(),
            KeyCode::Char('/') => self.search_mode = true,
            KeyCode::Char(' ') => self.toggle_current_selection(),
            KeyCode::Char('d') => self.queue_current_delete(),
            KeyCode::Char('D') => self.queue_selected_delete(),
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('g') => self.jump_to_top(),
            KeyCode::Char('G') => self.jump_to_bottom(),
            _ => {}
        }
    }

    pub fn selected_count(&self) -> usize {
        self.sessions
            .iter()
            .filter(|session| session.selected)
            .count()
    }

    pub fn current_project(&self) -> Option<&crate::codex::Project> {
        self.project_state
            .selected()
            .and_then(|index| self.projects.get(index))
    }

    pub fn current_session(&self) -> Option<&Session> {
        let session_index = self
            .session_state
            .selected()
            .and_then(|list_index| self.filtered_session_indices.get(list_index).copied())?;
        self.sessions.get(session_index)
    }

    pub fn no_sessions_message(&self) -> String {
        if !self.root_exists {
            format!("No Codex sessions found at {}", self.scan_root_label)
        } else if self.sessions.is_empty() {
            "No session files found.".to_string()
        } else if !self.search_query.is_empty() {
            format!("No sessions match search: {}", self.search_query)
        } else {
            "No sessions in this project group.".to_string()
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.search_query.clear();
                self.search_mode = false;
                self.rebuild_filtered_sessions();
                self.status = "Search cleared".to_string();
            }
            KeyCode::Enter => {
                self.search_mode = false;
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.rebuild_filtered_sessions();
            }
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.search_query.push(c);
                self.rebuild_filtered_sessions();
            }
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.detail = None,
            KeyCode::Char('d') => self.queue_current_delete(),
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_confirm_key(&mut self, key: KeyEvent, confirm: DeleteRequest) {
        match key.code {
            KeyCode::Char('y') => self.execute_delete(confirm.session_indices),
            KeyCode::Esc | KeyCode::Char('n') | KeyCode::Enter => {
                self.confirm_delete = None;
                self.status = "Delete cancelled".to_string();
            }
            _ => {}
        }
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Projects => Focus::Sessions,
            Focus::Sessions => Focus::Projects,
        };
    }

    fn move_up(&mut self) {
        match self.focus {
            Focus::Projects => {
                let changed = move_state_up(&mut self.project_state, self.projects.len());
                if changed {
                    self.rebuild_filtered_sessions();
                }
            }
            Focus::Sessions => {
                move_state_up(&mut self.session_state, self.filtered_session_indices.len());
            }
        }
    }

    fn move_down(&mut self) {
        match self.focus {
            Focus::Projects => {
                let changed = move_state_down(&mut self.project_state, self.projects.len());
                if changed {
                    self.rebuild_filtered_sessions();
                }
            }
            Focus::Sessions => {
                move_state_down(&mut self.session_state, self.filtered_session_indices.len());
            }
        }
    }

    fn jump_to_top(&mut self) {
        match self.focus {
            Focus::Projects => {
                if !self.projects.is_empty() {
                    self.project_state.select(Some(0));
                    self.rebuild_filtered_sessions();
                }
            }
            Focus::Sessions => {
                if !self.filtered_session_indices.is_empty() {
                    self.session_state.select(Some(0));
                }
            }
        }
    }

    fn jump_to_bottom(&mut self) {
        match self.focus {
            Focus::Projects => {
                if !self.projects.is_empty() {
                    self.project_state.select(Some(self.projects.len() - 1));
                    self.rebuild_filtered_sessions();
                }
            }
            Focus::Sessions => {
                if !self.filtered_session_indices.is_empty() {
                    self.session_state
                        .select(Some(self.filtered_session_indices.len() - 1));
                }
            }
        }
    }

    fn toggle_current_selection(&mut self) {
        let Some(session_index) = self.current_session_index() else {
            return;
        };

        if let Some(session) = self.sessions.get_mut(session_index) {
            session.selected = !session.selected;
            self.status = if session.selected {
                format!("Selected {}", session.id)
            } else {
                format!("Unselected {}", session.id)
            };
        }
    }

    fn current_session_index(&self) -> Option<usize> {
        let selected_row = self.session_state.selected()?;
        self.filtered_session_indices.get(selected_row).copied()
    }

    fn queue_current_delete(&mut self) {
        let Some(session_index) = self.current_session_index() else {
            return;
        };

        let Some(session) = self.sessions.get(session_index) else {
            return;
        };

        self.confirm_delete = Some(DeleteRequest {
            session_indices: vec![session_index],
            prompt: format!("Delete this session? y/N [{}]", session.id),
        });
    }

    fn queue_selected_delete(&mut self) {
        let session_indices = self
            .sessions
            .iter()
            .enumerate()
            .filter_map(|(index, session)| session.selected.then_some(index))
            .collect::<Vec<_>>();

        if session_indices.is_empty() {
            self.status = "No selected sessions".to_string();
            return;
        }

        self.confirm_delete = Some(DeleteRequest {
            prompt: format!("Delete selected {} sessions? y/N", session_indices.len()),
            session_indices,
        });
    }

    fn execute_delete(&mut self, mut session_indices: Vec<usize>) {
        self.confirm_delete = None;
        session_indices.sort_unstable();
        session_indices.dedup();

        let mut removed = Vec::new();
        let mut failures = Vec::new();

        for index in session_indices {
            let Some(session) = self.sessions.get(index).cloned() else {
                continue;
            };

            match move_session_to_trash(&session.file_path) {
                Ok(_) => removed.push(index),
                Err(error) => failures.push(format!("{}: {error}", session.file_path.display())),
            }
        }

        removed.sort_unstable_by(|left, right| right.cmp(left));
        let deleted_current = self
            .detail
            .as_ref()
            .map(|detail| detail.session.file_path.clone());

        for index in removed.iter().copied() {
            self.sessions.remove(index);
        }

        if let Some(path) = deleted_current {
            if !self
                .sessions
                .iter()
                .any(|session| session.file_path == path)
            {
                self.detail = None;
            }
        }

        self.rebuild_views();

        self.status = match (removed.is_empty(), failures.is_empty()) {
            (false, true) => format!("Moved {} session(s) to trash", removed.len()),
            (false, false) => format!(
                "Moved {} session(s); {} failure(s)",
                removed.len(),
                failures.len()
            ),
            (true, false) => failures
                .first()
                .cloned()
                .unwrap_or_else(|| "Delete failed".to_string()),
            (true, true) => "No sessions deleted".to_string(),
        };

        if !failures.is_empty() {
            self.warnings.extend(failures);
        }
    }

    fn open_detail(&mut self) {
        let Some(session) = self.current_session().cloned() else {
            return;
        };

        match load_detail(&session) {
            Ok(detail) => self.detail = Some(detail),
            Err(error) => self.status = format!("Detail load failed: {error}"),
        }
    }

    fn refresh(&mut self) {
        let selected_project = self.current_project().map(|project| project.path.clone());
        let selected_session_path = self
            .current_session()
            .map(|session| session.file_path.clone());
        let scan = scan_sessions(&self.scan_root);
        self.root_exists = scan.root_exists;
        self.sessions = scan.sessions;
        self.warnings = scan.warnings;
        self.projects = build_projects(&self.sessions);

        let target_project = selected_project.unwrap_or_else(|| "All Sessions".to_string());
        let project_index = self
            .projects
            .iter()
            .position(|project| project.path == target_project)
            .unwrap_or(0);
        self.project_state.select(Some(project_index));

        self.rebuild_filtered_sessions_with_target(selected_session_path.as_ref());
        self.detail = None;
        self.confirm_delete = None;
        self.status = if self.root_exists {
            format!("Refreshed {} sessions", self.sessions.len())
        } else {
            format!("No Codex sessions found at {}", self.scan_root_label)
        };
    }

    fn rebuild_views(&mut self) {
        self.projects = build_projects(&self.sessions);
        if self.project_state.selected().is_none() {
            self.project_state.select(Some(0));
        }
        self.rebuild_filtered_sessions();
    }

    fn rebuild_filtered_sessions(&mut self) {
        let current_path = self
            .current_session()
            .map(|session| session.file_path.clone());
        self.rebuild_filtered_sessions_with_target(current_path.as_ref());
    }

    fn rebuild_filtered_sessions_with_target(&mut self, target_path: Option<&PathBuf>) {
        let selected_project_path = self.current_project().map(|project| project.path.clone());
        let query = self.search_query.to_lowercase();

        self.filtered_session_indices = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, session)| match selected_project_path.as_deref() {
                Some("All Sessions") | None => true,
                Some(project) => session.project_path == project,
            })
            .filter(|(_, session)| query.is_empty() || session.search_blob().contains(&query))
            .map(|(index, _)| index)
            .collect();

        if self.filtered_session_indices.is_empty() {
            self.session_state.select(None);
            return;
        }

        if let Some(path) = target_path {
            if let Some(index) = self
                .filtered_session_indices
                .iter()
                .position(|session_index| self.sessions[*session_index].file_path == *path)
            {
                self.session_state.select(Some(index));
                return;
            }
        }

        let selected = self
            .session_state
            .selected()
            .unwrap_or(0)
            .min(self.filtered_session_indices.len().saturating_sub(1));
        self.session_state.select(Some(selected));
    }
}

fn move_state_up(state: &mut ListState, len: usize) -> bool {
    if len == 0 {
        state.select(None);
        return false;
    }
    let next = state.selected().unwrap_or(0).saturating_sub(1);
    let changed = state.selected() != Some(next);
    state.select(Some(next));
    changed
}

fn move_state_down(state: &mut ListState, len: usize) -> bool {
    if len == 0 {
        state.select(None);
        return false;
    }
    let next = match state.selected() {
        Some(current) if current + 1 < len => current + 1,
        Some(current) => current,
        None => 0,
    };
    let changed = state.selected() != Some(next);
    state.select(Some(next));
    changed
}
