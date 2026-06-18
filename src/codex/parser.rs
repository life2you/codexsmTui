use std::{
    collections::VecDeque,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::Path,
};

use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};
use serde_json::Value;
use unicode_width::UnicodeWidthStr;

use crate::codex::session::{
    MessageSnippet, Session, SessionDetail, UNKNOWN_PROJECT, UNTITLED_SESSION,
};

const PREVIEW_LINE_LIMIT: usize = 50;
const DETAIL_HEAD_LINE_LIMIT: usize = 200;
const DETAIL_TAIL_BYTES: u64 = 128 * 1024;
const DETAIL_RECENT_LIMIT: usize = 8;

pub fn parse_session_preview(path: &Path) -> Result<Session> {
    let metadata = fs::metadata(path)
        .with_context(|| format!("unable to read metadata for {}", path.display()))?;
    let size = metadata.len();
    let file_updated_at = metadata.modified().ok().and_then(system_time_to_local);

    let file = File::open(path).with_context(|| format!("unable to open {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut session_id: Option<String> = None;
    let mut project_path: Option<String> = None;
    let mut title: Option<String> = None;
    let mut first_user_text: Option<String> = None;
    let mut created_at: Option<DateTime<Local>> = None;
    let mut last_seen_timestamp: Option<DateTime<Local>> = None;

    for line in reader.lines().take(PREVIEW_LINE_LIMIT) {
        let Ok(line) = line else { continue };
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        if created_at.is_none() {
            created_at = extract_created_time(&value);
        }

        last_seen_timestamp = extract_timestamp(&value).or(last_seen_timestamp);

        if session_id.is_none() {
            session_id = extract_session_id(&value);
        }

        if project_path.is_none() {
            project_path = extract_project_path(&value);
        }

        if title.is_none() {
            title = extract_title(&value);
        }

        if first_user_text.is_none() {
            first_user_text = extract_message_snippet(&value, "user");
        }
    }

    let title = title
        .or(first_user_text)
        .unwrap_or_else(|| UNTITLED_SESSION.to_string());
    let title = truncate_display_width(&title, 80);

    Ok(Session {
        id: session_id.unwrap_or_else(|| fallback_session_id(path)),
        title,
        project_path: project_path.unwrap_or_else(|| UNKNOWN_PROJECT.to_string()),
        file_path: path.to_path_buf(),
        created_at: created_at.or(file_updated_at),
        updated_at: file_updated_at.or(last_seen_timestamp).or(created_at),
        size,
        selected: false,
    })
}

pub fn load_session_detail(session: &Session) -> Result<SessionDetail> {
    let recent_messages = read_recent_messages(&session.file_path)?;

    Ok(SessionDetail {
        session: session.clone(),
        recent_messages,
    })
}

fn read_recent_messages(path: &Path) -> Result<Vec<MessageSnippet>> {
    let mut recent = parse_recent_messages_from_tail(path)?;
    if recent.is_empty() {
        recent = parse_recent_messages_from_head(path)?;
    }
    Ok(recent)
}

fn parse_recent_messages_from_head(path: &Path) -> Result<Vec<MessageSnippet>> {
    let file = File::open(path).with_context(|| format!("unable to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut messages = VecDeque::with_capacity(DETAIL_RECENT_LIMIT);

    for line in reader.lines().take(DETAIL_HEAD_LINE_LIMIT) {
        let Ok(line) = line else { continue };
        let Ok(value) = serde_json::from_str::<Value>(&line) else {
            continue;
        };

        if let Some(message) = extract_recent_message(&value) {
            if messages.len() == DETAIL_RECENT_LIMIT {
                messages.pop_front();
            }
            messages.push_back(message);
        }
    }

    Ok(messages.into_iter().collect())
}

fn parse_recent_messages_from_tail(path: &Path) -> Result<Vec<MessageSnippet>> {
    let mut file =
        File::open(path).with_context(|| format!("unable to open {}", path.display()))?;
    let file_len = file
        .metadata()
        .with_context(|| format!("unable to read metadata for {}", path.display()))?
        .len();
    let start = file_len.saturating_sub(DETAIL_TAIL_BYTES);
    file.seek(SeekFrom::Start(start))
        .with_context(|| format!("unable to seek {}", path.display()))?;

    let mut chunk = String::new();
    file.read_to_string(&mut chunk)
        .with_context(|| format!("unable to read tail of {}", path.display()))?;

    let content = if start > 0 {
        chunk.split_once('\n').map(|(_, rest)| rest).unwrap_or("")
    } else {
        chunk.as_str()
    };

    let mut messages = VecDeque::with_capacity(DETAIL_RECENT_LIMIT);
    for line in content.lines() {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if let Some(message) = extract_recent_message(&value) {
            if messages.len() == DETAIL_RECENT_LIMIT {
                messages.pop_front();
            }
            messages.push_back(message);
        }
    }

    Ok(messages.into_iter().collect())
}

fn extract_recent_message(value: &Value) -> Option<MessageSnippet> {
    let message = extract_role_and_text(value)?;
    if message.1.is_empty() {
        return None;
    }

    Some(MessageSnippet {
        role: message.0.to_string(),
        timestamp: extract_timestamp(value),
        text: truncate_display_width(&message.1, 160),
    })
}

fn extract_title(value: &Value) -> Option<String> {
    for path in [
        &["payload", "title"][..],
        &["title"][..],
        &["payload", "session_title"][..],
    ] {
        if let Some(text) = get_nested_str(value, path)
            .and_then(clean_text)
            .filter(|text| !is_noise_text(text))
        {
            return Some(text);
        }
    }
    None
}

fn extract_session_id(value: &Value) -> Option<String> {
    for path in [
        &["payload", "id"][..],
        &["id"][..],
        &["payload", "session_id"][..],
    ] {
        if let Some(id) = get_nested_str(value, path)
            .map(str::trim)
            .filter(|id| !id.is_empty())
        {
            return Some(id.to_string());
        }
    }
    None
}

fn extract_project_path(value: &Value) -> Option<String> {
    for path in [
        &["payload", "cwd"][..],
        &["cwd"][..],
        &["payload", "current_working_directory"][..],
        &["payload", "project_path"][..],
    ] {
        if let Some(text) = get_nested_str(value, path).and_then(clean_text) {
            return Some(text);
        }
    }
    None
}

fn extract_created_time(value: &Value) -> Option<DateTime<Local>> {
    for path in [
        &["payload", "timestamp"][..],
        &["timestamp"][..],
        &["created_at"][..],
        &["payload", "created_at"][..],
    ] {
        if let Some(dt) = get_nested_str(value, path).and_then(parse_datetime) {
            return Some(dt);
        }
    }
    None
}

fn extract_timestamp(value: &Value) -> Option<DateTime<Local>> {
    for path in [
        &["timestamp"][..],
        &["payload", "timestamp"][..],
        &["updated_at"][..],
        &["payload", "updated_at"][..],
    ] {
        if let Some(dt) = get_nested_str(value, path).and_then(parse_datetime) {
            return Some(dt);
        }
    }
    None
}

fn extract_message_snippet(value: &Value, expected_role: &str) -> Option<String> {
    let (role, text) = extract_role_and_text(value)?;
    if role != expected_role {
        return None;
    }
    Some(truncate_display_width(&text, 80))
}

fn extract_role_and_text(value: &Value) -> Option<(&str, String)> {
    if value.get("type").and_then(Value::as_str) == Some("event_msg")
        && get_nested_str(value, &["payload", "type"]) == Some("user_message")
    {
        let text = get_nested_str(value, &["payload", "message"]).and_then(clean_text)?;
        if !is_noise_text(&text) {
            return Some(("user", text));
        }
    }

    if value.get("type").and_then(Value::as_str) == Some("response_item")
        && get_nested_str(value, &["payload", "type"]) == Some("message")
    {
        let role = get_nested_str(value, &["payload", "role"])?;
        let text = collect_message_text(value.get("payload")?.get("content")?)?;
        if !is_noise_text(&text) {
            return Some((role, text));
        }
    }

    None
}

fn collect_message_text(value: &Value) -> Option<String> {
    let items = value.as_array()?;
    for item in items {
        if let Some(text) = item
            .get("text")
            .and_then(Value::as_str)
            .and_then(clean_text)
            .filter(|text| !is_noise_text(text))
        {
            return Some(text);
        }
    }
    None
}

fn clean_text(text: &str) -> Option<String> {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let cleaned = collapsed.trim();
    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.to_string())
    }
}

fn is_noise_text(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("<environment_context>")
        || lower.contains("# agents.md instructions")
        || lower.contains("<permissions instructions>")
        || lower.contains("<app-context>")
        || lower.contains("<collaboration_mode>")
        || lower.contains("filesystem sandboxing defines")
        || lower.contains("developer_instructions")
}

fn fallback_session_id(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("session");
    let shortened = stem.chars().take(24).collect::<String>();
    format!("file:{}", shortened)
}

fn get_nested_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str()
}

fn parse_datetime(value: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|dt| dt.with_timezone(&Local))
        .or_else(|| {
            DateTime::<Utc>::from_timestamp(value.parse().ok()?, 0)
                .map(|dt| dt.with_timezone(&Local))
        })
}

fn system_time_to_local(system_time: std::time::SystemTime) -> Option<DateTime<Local>> {
    Some(DateTime::<Local>::from(DateTime::<Utc>::from(system_time)))
}

pub fn truncate_display_width(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }

    let mut current = String::new();
    let mut width = 0;
    for ch in text.chars() {
        let ch_width = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width + 3 > max_width {
            break;
        }
        current.push(ch);
        width += ch_width;
    }
    current.push_str("...");
    current
}

#[cfg(test)]
mod tests {
    use super::{extract_role_and_text, fallback_session_id, truncate_display_width};
    use serde_json::json;
    use std::path::Path;

    #[test]
    fn truncates_wide_strings() {
        assert_eq!(truncate_display_width("hello world", 8), "hello...");
    }

    #[test]
    fn builds_file_based_fallback_id() {
        let id = fallback_session_id(Path::new("/tmp/rollout-abc-123.jsonl"));
        assert!(id.starts_with("file:rollout-abc-123"));
    }

    #[test]
    fn extracts_message_text() {
        let value = json!({
            "type": "response_item",
            "payload": {
                "type": "message",
                "role": "user",
                "content": [
                    {
                        "type": "input_text",
                        "text": "hello   codex"
                    }
                ]
            }
        });

        let message = extract_role_and_text(&value).unwrap();
        assert_eq!(message.0, "user");
        assert_eq!(message.1, "hello codex");
    }
}
