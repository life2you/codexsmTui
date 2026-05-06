use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::Local;

pub fn default_trash_root() -> Result<PathBuf> {
    let home = dirs::home_dir().context("unable to resolve home directory")?;
    Ok(home.join(".codex").join("session-trash"))
}

pub fn move_session_to_trash(path: &Path) -> Result<PathBuf> {
    let trash_root = default_trash_root()?;
    let dated_dir = trash_root.join(Local::now().format("%Y-%m-%d").to_string());
    fs::create_dir_all(&dated_dir)
        .with_context(|| format!("unable to create {}", dated_dir.display()))?;

    let target = unique_target_path(&dated_dir, path);

    match fs::rename(path, &target) {
        Ok(_) => Ok(target),
        Err(_) => {
            fs::copy(path, &target).with_context(|| {
                format!("unable to copy {} to {}", path.display(), target.display())
            })?;
            fs::remove_file(path)
                .with_context(|| format!("unable to remove {}", path.display()))?;
            Ok(target)
        }
    }
}

fn unique_target_path(dir: &Path, source: &Path) -> PathBuf {
    let file_name = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("session.jsonl");
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = source
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("session");
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let mut counter = 1usize;
    loop {
        let name = if extension.is_empty() {
            format!("{stem}-{counter}")
        } else {
            format!("{stem}-{counter}.{extension}")
        };
        let next = dir.join(name);
        if !next.exists() {
            return next;
        }
        counter += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::unique_target_path;
    use std::{fs, path::Path};

    #[test]
    fn appends_suffix_when_name_exists() {
        let base = std::env::temp_dir().join(format!(
            "codexsmtui-trash-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&base).unwrap();
        let existing = base.join("sample.jsonl");
        fs::write(&existing, "x").unwrap();

        let candidate = unique_target_path(&base, Path::new("/tmp/sample.jsonl"));
        assert!(candidate.ends_with("sample-1.jsonl"));

        fs::remove_file(existing).unwrap();
        fs::remove_dir_all(base).unwrap();
    }
}
