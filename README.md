# codexsmTui

`codexsmTui` is a terminal-first Rust TUI for managing local OpenAI Codex CLI session files.

It scans local Codex session JSONL files, groups them by project path, supports real-time search, opens a metadata/detail view, and deletes safely by moving files into a local trash directory instead of permanently removing them.

## Features

- Launches directly into an interactive TUI
- Recursively scans `~/.codex/sessions/**/*.jsonl`
- Shows all sessions or groups by project path
- Sorts sessions by updated time descending
- Real-time search across title, project path, session id, and file path
- Session detail popup with metadata and recent message snippets
- Single delete with confirmation
- Multi-select and batch delete with confirmation
- Safe delete by moving files to `~/.codex/session-trash/YYYY-MM-DD/`

## Install

### Build from source

```bash
git clone https://github.com/life2you/codexsmTui.git
cd codexsmTui
cargo build --release
```

The binary will be available at:

```bash
target/release/codexsmTui
```

### Homebrew

After the formula is published to the tap:

```bash
brew tap life2you/tap
brew install codexsmtui
```

## Release

Maintainer release steps live in [RELEASING.md](/Users/life2you/vibeCodes/github/codexsmTui/RELEASING.md).

## Usage

Run the binary with no arguments:

```bash
codexsmTui
```

By default it scans:

```text
~/.codex/sessions
```

## Keyboard Shortcuts

```text
q      quit
?      toggle help
tab    switch focus between project list and session list
↑/↓    move selection
enter  open current session detail
/      enter search mode
esc    exit search or close popup
space  select / unselect current session
d      delete current session
D      delete all selected sessions
r      refresh scan
g      jump to top
G      jump to bottom
y      confirm delete in confirmation dialog
```

## Safe Delete

`codexsmTui` does not permanently remove session files in the first version.

Instead, deleting a session moves the JSONL file to:

```text
~/.codex/session-trash/YYYY-MM-DD/
```

This reduces the chance of accidental data loss and keeps a simple recovery path for future restore tooling.

## Session Parsing Notes

Each `.jsonl` file is treated as one session or session fragment.

For list rendering, `codexsmTui` only reads a limited number of lines from the head of each file instead of loading large session files entirely into memory.

The parser tries to extract:

- session id
- project path / cwd
- created time
- updated time
- title or first user message summary
- file path

If some fields are missing or some JSON lines are broken, the scanner keeps going and falls back to safe defaults.

## Relationship to milisp/codexsm

codexsmTui is inspired by [https://github.com/milisp/codexsm](https://github.com/milisp/codexsm).

milisp/codexsm provides a Tauri-based desktop application for managing Codex CLI sessions, including viewing, renaming, deleting, and resuming sessions. codexsmTui focuses on a terminal-first workflow and reimplements the session scanning, parsing, viewing, and safe deletion logic in Rust with ratatui.

This project does not copy source code from milisp/codexsm. It only references the product idea and general Codex session management approach.

This project is explicitly reimplemented from scratch:

- TUI logic is reimplemented with `ratatui`
- Session scanning is reimplemented with `walkdir`
- Session parsing is reimplemented with `serde_json`
- Safe deletion is reimplemented by moving files into a local trash directory

Thanks to `milisp/codexsm` for the inspiration.

## License

This project is released under the MIT License. See [LICENSE](/Users/life2you/vibeCodes/github/codexsmTui/LICENSE).

## Roadmap

- Homebrew formula and tap publishing
- Optional permanent delete mode
- Session metadata cache
- Trash restore workflow
