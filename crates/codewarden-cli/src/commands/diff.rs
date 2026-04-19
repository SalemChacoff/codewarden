use codewarden_core::diff::file_change::FileChange;
use codewarden_core::diff::parse_diff;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};

pub fn run(path: &str) {
    run_with_writer(path, &mut io::stdout());
}

pub fn run_with_writer(path: &str, w: &mut impl Write) {
    let diff_text = if path == "-" { read_stdin() } else { run_git_diff(path) };

    let files = parse_diff(&diff_text);
    report(path, &files, w);
}

fn run_git_diff(dir: &str) -> String {
    let output = Command::new("git")
        .args(["diff", "HEAD"])
        .current_dir(Path::new(dir))
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output();

    match output {
        Ok(out) if out.status.success() || !out.stdout.is_empty() => String::from_utf8_lossy(&out.stdout).into_owned(),
        _ => String::new(),
    }
}

fn read_stdin() -> String {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf).unwrap_or(0);
    buf
}

fn report(source: &str, files: &[FileChange], w: &mut impl Write) {
    writeln!(w, "Found {} changed file(s) in '{}'", files.len(), source).expect("write failed");

    for file in files {
        let added: usize = file.added_lines().count();
        let removed: usize = file.removed_lines().count();

        writeln!(w, "  {:?} {} (+{} -{} lines)", file.kind, file.path, added, removed).expect("write failed");

        for line in file.added_lines() {
            let _ = line;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn capture_from_diff(diff: &str) -> String {
        let files = parse_diff(diff);
        let mut buf: Vec<u8> = Vec::new();
        report("test-source", &files, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    fn capture_writer(path: &str) -> String {
        let mut buf: Vec<u8> = Vec::new();
        run_with_writer(path, &mut buf);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn empty_diff_reports_zero_files() {
        let out = capture_from_diff("");
        assert!(out.contains("Found 0 changed file(s)"), "got: {out}");
    }

    #[test]
    fn garbage_input_produces_no_files() {
        let out = capture_from_diff("this is not a diff\nrandom text\n");
        assert!(out.contains("Found 0 changed file(s)"), "got: {out}");
    }

    #[test]
    fn parses_added_file() {
        let diff = "\
diff --git a/src/main.rs b/src/main.rs
new file mode 100644
--- /dev/null
+++ b/src/main.rs
@@ -0,0 +1,2 @@
+fn main() {}
+// hello
";
        let out = capture_from_diff(diff);
        assert!(out.contains("Found 1 changed file(s)"), "got: {out}");
        assert!(out.contains("src/main.rs"), "got: {out}");
        assert!(out.contains("Added"), "got: {out}");
        assert!(out.contains("+2"), "got: {out}");
    }

    #[test]
    fn parses_deleted_file() {
        let diff = "\
diff --git a/old.rs b/old.rs
deleted file mode 100644
--- a/old.rs
+++ /dev/null
@@ -1,1 +0,0 @@
-fn old() {}
";
        let out = capture_from_diff(diff);
        assert!(out.contains("Found 1 changed file(s)"), "got: {out}");
        assert!(out.contains("Deleted"), "got: {out}");
        assert!(out.contains("-1"), "got: {out}");
    }

    #[test]
    fn parses_modified_file() {
        let diff = "\
diff --git a/lib.rs b/lib.rs
--- a/lib.rs
+++ b/lib.rs
@@ -1,3 +1,4 @@
 fn existing() {}
-fn old() {}
+fn new() {}
+fn extra() {}
 // end
";
        let out = capture_from_diff(diff);
        assert!(out.contains("Found 1 changed file(s)"), "got: {out}");
        assert!(out.contains("lib.rs"), "got: {out}");
        assert!(out.contains("Modified"), "got: {out}");
        assert!(out.contains("+2"), "got: {out}");
        assert!(out.contains("-1"), "got: {out}");
    }

    #[test]
    fn parses_renamed_file() {
        let diff = "\
diff --git a/old_name.rs b/new_name.rs
rename from old_name.rs
rename to new_name.rs
--- a/old_name.rs
+++ b/new_name.rs
@@ -1,1 +1,1 @@
-fn old() {}
+fn renamed() {}
";
        let out = capture_from_diff(diff);
        assert!(out.contains("Found 1 changed file(s)"), "got: {out}");
        assert!(out.contains("Renamed"), "got: {out}");
        assert!(out.contains("new_name.rs"), "got: {out}");
    }

    #[test]
    fn parses_multiple_files() {
        let diff = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1,1 +1,1 @@
-fn a_old() {}
+fn a_new() {}
diff --git a/b.rs b/b.rs
new file mode 100644
--- /dev/null
+++ b/b.rs
@@ -0,0 +1,1 @@
+fn b() {}
";
        let out = capture_from_diff(diff);
        assert!(out.contains("Found 2 changed file(s)"), "got: {out}");
        assert!(out.contains("a.rs"), "got: {out}");
        assert!(out.contains("b.rs"), "got: {out}");
    }

    #[test]
    fn parses_multiple_hunks_in_one_file() {
        let diff = "\
diff --git a/big.rs b/big.rs
--- a/big.rs
+++ b/big.rs
@@ -1,2 +1,2 @@
-fn first_old() {}
+fn first_new() {}
 // separator
@@ -10,2 +10,2 @@
-fn second_old() {}
+fn second_new() {}
";
        let files = parse_diff(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].hunks.len(), 2);
    }

    #[test]
    fn added_and_removed_line_counts_are_correct() {
        let diff = "\
diff --git a/x.rs b/x.rs
--- a/x.rs
+++ b/x.rs
@@ -1,3 +1,4 @@
+fn added1() {}
+fn added2() {}
+fn added3() {}
-fn removed() {}
 fn context() {}
";
        let files = parse_diff(diff);
        assert_eq!(files[0].added_lines().count(), 3);
        assert_eq!(files[0].removed_lines().count(), 1);
    }

    #[test]
    fn nonexistent_path_yields_zero_changed_files() {
        let out = capture_writer("/this/does/not/exist_cw_diff_test");
        assert!(out.contains("Found 0 changed file(s)"), "got: {out}");
    }

    #[test]
    fn non_git_directory_yields_zero_changed_files() {
        let dir = std::env::temp_dir().join(format!("cw_diff_nogit_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let out = capture_writer(dir.to_str().unwrap());
        let _ = std::fs::remove_dir_all(&dir);

        assert!(out.contains("Found 0 changed file(s)"), "got: {out}");
    }
}
