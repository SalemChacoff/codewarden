use crate::diff::file_change::{ChangeKind, FileChange};
use crate::diff::hunk::{Hunk, HunkLine};

pub fn parse_diff(input: &str) -> Vec<FileChange> {
    let mut files: Vec<FileChange> = Vec::new();
    let mut current_file: Option<FileChange> = None;
    let mut current_hunk: Option<Hunk> = None;

    for line in input.lines() {
        let bytes = line.as_bytes();

        match bytes.first() {
            Some(b'd') if line.starts_with("diff --git ") => {
                flush_hunk(&mut current_hunk, &mut current_file);
                if let Some(file) = current_file.take() {
                    files.push(file);
                }
                let path = line
                    .split_whitespace()
                    .last()
                    .and_then(|p| p.strip_prefix("b/"))
                    .unwrap_or("")
                    .to_string();

                current_file = Some(FileChange {
                    path,
                    kind: ChangeKind::Modified,
                    hunks: Vec::new(),
                });
            }

            Some(b'n') if line.starts_with("new file mode") => {
                if let Some(f) = current_file.as_mut() {
                    f.kind = ChangeKind::Added;
                }
            }
            Some(b'd') if line.starts_with("deleted file mode") => {
                if let Some(f) = current_file.as_mut() {
                    f.kind = ChangeKind::Deleted;
                }
            }
            Some(b'r') if line.starts_with("rename from ") => {
                let from = line["rename from ".len()..].to_string();
                if let Some(f) = current_file.as_mut() {
                    f.kind = ChangeKind::Renamed { from };
                }
            }

            Some(b'@') if line.starts_with("@@") => {
                flush_hunk(&mut current_hunk, &mut current_file);
                current_hunk = parse_hunk_header(line);
            }

            Some(b'+') if !line.starts_with("+++") => {
                if let Some(hunk) = current_hunk.as_mut() {
                    hunk.lines.push(HunkLine::Added(line[1..].to_string()));
                }
            }

            Some(b'-') if !line.starts_with("---") => {
                if let Some(hunk) = current_hunk.as_mut() {
                    hunk.lines.push(HunkLine::Removed(line[1..].to_string()));
                }
            }

            Some(b' ') => {
                if let Some(hunk) = current_hunk.as_mut() {
                    hunk.lines.push(HunkLine::Context(line[1..].to_string()));
                }
            }

            _ => {}
        }
    }

    flush_hunk(&mut current_hunk, &mut current_file);
    if let Some(file) = current_file.take() {
        files.push(file);
    }

    files
}

fn parse_hunk_header(line: &str) -> Option<Hunk> {
    let inner = line.strip_prefix("@@ ")?.split(" @@").next()?;
    let mut parts = inner.split_whitespace();

    let old = parse_range(parts.next()?.strip_prefix('-')?);
    let new = parse_range(parts.next()?.strip_prefix('+')?);

    Some(Hunk {
        old_start: old.0,
        old_lines: old.1,
        new_start: new.0,
        new_lines: new.1,
        lines: Vec::new(),
    })
}

fn parse_range(s: &str) -> (u32, u32) {
    match s.split_once(',') {
        Some((start, count)) => (start.parse().unwrap_or(0), count.parse().unwrap_or(1)),
        None => (s.parse().unwrap_or(0), 1),
    }
}

fn flush_hunk(hunk: &mut Option<Hunk>, file: &mut Option<FileChange>) {
    if let (Some(h), Some(f)) = (hunk.take(), file.as_mut()) {
        f.hunks.push(h);
    }
}
