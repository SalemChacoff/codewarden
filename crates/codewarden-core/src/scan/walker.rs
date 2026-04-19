use std::path::Path;

use crate::diff::file_change::{ChangeKind, FileChange};
use crate::diff::hunk::{Hunk, HunkLine};
use crate::scan::filter;
use crate::scan::filter::should_scan;

pub fn walk_directory(root: &Path) -> Vec<FileChange> {
    let mut results = Vec::new();
    walk_recursive(root, root, &mut results);
    results
}

fn walk_recursive(root: &Path, dir: &Path, out: &mut Vec<FileChange>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            if !filter::should_skip_dir(&path) {
                walk_recursive(root, &path, out);
            }
        } else if path.is_file() && should_scan(&path) {
            if let Some(fc) = file_to_file_change(root, &path) {
                out.push(fc);
            }
        }
    }
}

fn file_to_file_change(root: &Path, path: &Path) -> Option<FileChange> {
    let content = std::fs::read_to_string(path).ok()?;

    let relative = path.strip_prefix(root).unwrap_or(path);
    let path_str = relative.to_string_lossy().replace('\\', "/");

    let lines: Vec<HunkLine> = content.lines().map(|l| HunkLine::Added(l.to_string())).collect();

    let hunk = Hunk {
        old_start: 0,
        old_lines: 0,
        new_start: 1,
        new_lines: lines.len() as u32,
        lines,
    };

    Some(FileChange {
        path: path_str,
        kind: ChangeKind::Added,
        hunks: vec![hunk],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::hunk::HunkLine;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("codewarden_walker_test__{label}__{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn cleanup(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    fn write(dir: &PathBuf, rel: &str, content: &str) -> PathBuf {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(&path, content).expect("write file");
        path
    }

    #[test]
    fn empty_directory_yields_no_results() {
        let root = temp_dir("empty");
        let results = walk_directory(&root);
        cleanup(&root);
        assert!(results.is_empty());
    }

    #[test]
    fn single_scannable_file_is_returned() {
        let root = temp_dir("single");
        write(&root, "main.rs", "fn main() {}");

        let results = walk_directory(&root);
        cleanup(&root);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "main.rs");
        assert!(matches!(results[0].kind, ChangeKind::Added));
    }

    #[test]
    fn non_scannable_file_is_ignored() {
        let root = temp_dir("non_scan");
        write(&root, "README.md", "# hello");
        write(&root, "binary.exe", "\x00\x01");

        let results = walk_directory(&root);
        cleanup(&root);

        assert!(results.is_empty(), "expected no results, got: {results:?}");
    }

    #[test]
    fn only_scannable_files_are_collected_from_mixed_dir() {
        let root = temp_dir("mixed");
        write(&root, "app.py", "print('hello')");
        write(&root, "notes.md", "some notes");
        write(&root, "config.toml", "[profile]");

        let results = walk_directory(&root);
        cleanup(&root);

        let paths: Vec<&str> = results.iter().map(|fc| fc.path.as_str()).collect();
        assert!(paths.contains(&"app.py"), "missing app.py");
        assert!(paths.contains(&"config.toml"), "missing config.toml");
        assert!(!paths.contains(&"notes.md"), "notes.md should be excluded");
    }

    #[test]
    fn skipped_directory_is_not_walked() {
        let root = temp_dir("skip_dir");
        write(&root, "target/debug/main.rs", "fn main() {}");
        write(&root, "src/lib.rs", "pub fn hello() {}");

        let results = walk_directory(&root);
        cleanup(&root);

        let paths: Vec<&str> = results.iter().map(|fc| fc.path.as_str()).collect();
        assert!(paths.iter().all(|p| !p.contains("target")), "target/ contents should be skipped, got: {paths:?}");
        assert!(paths.contains(&"src/lib.rs"));
    }

    #[test]
    fn files_are_found_recursively() {
        let root = temp_dir("recursive");
        write(&root, "a/b/c/deep.rs", "struct Deep;");

        let results = walk_directory(&root);
        cleanup(&root);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].path, "a/b/c/deep.rs");
    }

    #[test]
    fn nested_skipped_dirs_are_all_excluded() {
        let root = temp_dir("nested_skip");
        write(&root, "node_modules/pkg/index.js", "module.exports = {}");
        write(&root, ".git/config", "[core]");
        write(&root, "src/main.go", "package main");

        let results = walk_directory(&root);
        cleanup(&root);

        let paths: Vec<&str> = results.iter().map(|fc| fc.path.as_str()).collect();
        assert_eq!(paths, vec!["src/main.go"]);
    }

    #[test]
    fn file_lines_become_added_hunk_lines() {
        let root = temp_dir("hunk_lines");
        write(&root, "script.py", "line1\nline2\nline3");

        let results = walk_directory(&root);
        cleanup(&root);

        assert_eq!(results.len(), 1);
        let fc = &results[0];
        assert_eq!(fc.hunks.len(), 1);

        let lines: Vec<&HunkLine> = fc.hunks[0].lines.iter().collect();
        assert_eq!(lines.len(), 3);
        assert!(matches!(&lines[0], HunkLine::Added(s) if s == "line1"));
        assert!(matches!(&lines[1], HunkLine::Added(s) if s == "line2"));
        assert!(matches!(&lines[2], HunkLine::Added(s) if s == "line3"));
    }

    #[test]
    fn empty_file_produces_hunk_with_no_lines() {
        let root = temp_dir("empty_file");
        write(&root, "empty.rs", "");

        let results = walk_directory(&root);
        cleanup(&root);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].hunks[0].lines.len(), 0);
        assert_eq!(results[0].hunks[0].new_lines, 0);
    }

    #[test]
    fn hunk_old_fields_are_zero() {
        let root = temp_dir("hunk_old");
        write(&root, "main.rs", "fn main() {}");

        let results = walk_directory(&root);
        cleanup(&root);

        let hunk = &results[0].hunks[0];
        assert_eq!(hunk.old_start, 0);
        assert_eq!(hunk.old_lines, 0);
        assert_eq!(hunk.new_start, 1);
    }

    #[test]
    fn path_uses_forward_slashes() {
        let root = temp_dir("slashes");
        write(&root, "sub/dir/file.ts", "export {}");

        let results = walk_directory(&root);
        cleanup(&root);

        assert_eq!(results.len(), 1);
        assert!(!results[0].path.contains('\\'), "path should use forward slashes: {}", results[0].path);
        assert_eq!(results[0].path, "sub/dir/file.ts");
    }

    #[test]
    fn non_existent_root_returns_empty_vec() {
        let fake = PathBuf::from("/this/path/does/not/exist_codewarden_test");
        let results = walk_directory(&fake);
        assert!(results.is_empty());
    }

    /// When a scannable file exists but cannot be read (e.g. no read permission),
    /// `file_to_file_change` returns `None` and the file is silently skipped —
    /// covering the `.ok()?` early-return branch.
    #[test]
    #[cfg(unix)]
    fn unreadable_file_is_silently_skipped() {
        use std::os::unix::fs::PermissionsExt;

        let root = temp_dir("unreadable");
        let file = write(&root, "secret.rs", "fn secret() {}");

        // Remove read permission so read_to_string fails
        fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).expect("set permissions");

        let results = walk_directory(&root);

        // Restore so cleanup can remove the file
        let _ = fs::set_permissions(&file, fs::Permissions::from_mode(0o644));
        cleanup(&root);

        assert!(results.is_empty(), "unreadable file should be skipped, got: {results:?}");
    }

    /// Windows equivalent: deny read via `icacls`, covering the same `None` branch.
    #[test]
    #[cfg(windows)]
    fn unreadable_file_is_silently_skipped() {
        let root = temp_dir("unreadable");
        let file = write(&root, "secret.rs", "fn secret() {}");
        let file_str = file.to_string_lossy().to_string();

        // Deny the current user read access
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "Everyone".into());
        let deny = std::process::Command::new("icacls")
            .args([&file_str, "/deny", &format!("{username}:(R)"), "/t"])
            .output();

        // Only run the assertion if icacls succeeded; skip gracefully otherwise
        if deny.map(|o| o.status.success()).unwrap_or(false) {
            let results = walk_directory(&root);

            // Restore before cleanup
            let _ = std::process::Command::new("icacls")
                .args([&file_str, "/remove:d", &username])
                .output();
            cleanup(&root);

            assert!(results.is_empty(), "unreadable file should be skipped, got: {results:?}");
        } else {
            cleanup(&root);
        }
    }
}
