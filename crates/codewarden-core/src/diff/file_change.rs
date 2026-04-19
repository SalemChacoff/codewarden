use crate::diff::hunk::Hunk;

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed { from: String },
}

#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
    pub hunks: Vec<Hunk>,
}

impl FileChange {
    pub fn added_lines(&self) -> impl Iterator<Item = &str> {
        self.hunks.iter().flat_map(|h| {
            h.lines.iter().filter_map(|l| match l {
                crate::diff::hunk::HunkLine::Added(s) => Some(s.as_str()),
                _ => None,
            })
        })
    }

    pub fn removed_lines(&self) -> impl Iterator<Item = &str> {
        self.hunks.iter().flat_map(|h| {
            h.lines.iter().filter_map(|l| match l {
                crate::diff::hunk::HunkLine::Removed(s) => Some(s.as_str()),
                _ => None,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::hunk::{Hunk, HunkLine};

    fn make_hunk(lines: Vec<HunkLine>) -> Hunk {
        Hunk {
            old_start: 1,
            old_lines: lines.len() as u32,
            new_start: 1,
            new_lines: lines.len() as u32,
            lines,
        }
    }

    fn make_file_change(kind: ChangeKind, hunks: Vec<Hunk>) -> FileChange {
        FileChange {
            path: "src/main.rs".to_string(),
            kind,
            hunks,
        }
    }

    #[test]
    fn change_kind_variants_are_equal_to_themselves() {
        assert_eq!(ChangeKind::Added, ChangeKind::Added);
        assert_eq!(ChangeKind::Modified, ChangeKind::Modified);
        assert_eq!(ChangeKind::Deleted, ChangeKind::Deleted);
        assert_eq!(
            ChangeKind::Renamed { from: "old.rs".to_string() },
            ChangeKind::Renamed { from: "old.rs".to_string() }
        );
    }

    #[test]
    fn change_kind_renamed_differs_by_from_path() {
        assert_ne!(
            ChangeKind::Renamed { from: "a.rs".to_string() },
            ChangeKind::Renamed { from: "b.rs".to_string() }
        );
    }

    #[test]
    fn added_lines_returns_only_added() {
        let hunk = make_hunk(vec![
            HunkLine::Context("unchanged".to_string()),
            HunkLine::Added("new line".to_string()),
            HunkLine::Removed("old line".to_string()),
            HunkLine::Added("another new".to_string()),
        ]);
        let fc = make_file_change(ChangeKind::Modified, vec![hunk]);

        let added: Vec<&str> = fc.added_lines().collect();
        assert_eq!(added, vec!["new line", "another new"]);
    }

    #[test]
    fn added_lines_empty_when_no_additions() {
        let hunk = make_hunk(vec![
            HunkLine::Context("ctx".to_string()),
            HunkLine::Removed("gone".to_string()),
        ]);
        let fc = make_file_change(ChangeKind::Deleted, vec![hunk]);

        assert_eq!(fc.added_lines().count(), 0);
    }

    #[test]
    fn added_lines_spans_multiple_hunks() {
        let hunk1 = make_hunk(vec![HunkLine::Added("a".to_string())]);
        let hunk2 = make_hunk(vec![HunkLine::Added("b".to_string())]);
        let fc = make_file_change(ChangeKind::Modified, vec![hunk1, hunk2]);

        let added: Vec<&str> = fc.added_lines().collect();
        assert_eq!(added, vec!["a", "b"]);
    }

    #[test]
    fn removed_lines_returns_only_removed() {
        let hunk = make_hunk(vec![
            HunkLine::Context("ctx".to_string()),
            HunkLine::Added("new".to_string()),
            HunkLine::Removed("old".to_string()),
            HunkLine::Removed("also old".to_string()),
        ]);
        let fc = make_file_change(ChangeKind::Modified, vec![hunk]);

        let removed: Vec<&str> = fc.removed_lines().collect();
        assert_eq!(removed, vec!["old", "also old"]);
    }

    #[test]
    fn removed_lines_empty_when_no_removals() {
        let hunk = make_hunk(vec![HunkLine::Added("fresh".to_string())]);
        let fc = make_file_change(ChangeKind::Added, vec![hunk]);

        assert_eq!(fc.removed_lines().count(), 0);
    }

    #[test]
    fn removed_lines_spans_multiple_hunks() {
        let hunk1 = make_hunk(vec![HunkLine::Removed("x".to_string())]);
        let hunk2 = make_hunk(vec![HunkLine::Removed("y".to_string())]);
        let fc = make_file_change(ChangeKind::Modified, vec![hunk1, hunk2]);

        let removed: Vec<&str> = fc.removed_lines().collect();
        assert_eq!(removed, vec!["x", "y"]);
    }


    #[test]
    fn added_and_removed_lines_empty_when_no_hunks() {
        let fc = make_file_change(ChangeKind::Added, vec![]);
        assert_eq!(fc.added_lines().count(), 0);
        assert_eq!(fc.removed_lines().count(), 0);
    }
}
