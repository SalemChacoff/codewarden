use codewarden_core::scan::walk_directory;
use std::io::Write;
use std::path::Path;

pub fn run(path: &str) {
    run_with_writer(path, &mut std::io::stdout());
}

pub fn run_with_writer(path: &str, w: &mut impl Write) {
    let root = Path::new(path);
    let files = walk_directory(root);

    writeln!(w, "Found {} scannable files in '{}'", files.len(), path)
        .expect("write failed");

    for file in &files {
        writeln!(w, "  scanning: {}", file.path).expect("write failed");
        for line in file.added_lines() {
            // detectors run here
            let _ = line;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("cw_scan_test__{label}__{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn cleanup(dir: &PathBuf) {
        let _ = fs::remove_dir_all(dir);
    }

    fn write_file(dir: &PathBuf, rel: &str, content: &str) {
        let path = dir.join(rel);
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).expect("create parent");
        }
        fs::write(path, content).expect("write file");
    }

    fn capture(path: &str) -> String {
        let mut buf: Vec<u8> = Vec::new();
        run_with_writer(path, &mut buf);
        String::from_utf8(buf).expect("utf-8 output")
    }

    #[test]
    fn empty_dir_reports_zero_files() {
        let root = temp_dir("empty");
        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(
            out.contains("Found 0 scannable files"),
            "unexpected output: {out}"
        );
    }

    #[test]
    fn nonexistent_path_reports_zero_files() {
        let out = capture("/this/path/does/not/exist_cw_scan");
        assert!(out.contains("Found 0 scannable files"));
    }

    #[test]
    fn single_file_is_reported_in_summary() {
        let root = temp_dir("single");
        write_file(&root, "main.rs", "fn main() {}");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(out.contains("Found 1 scannable files"), "got: {out}");
    }

    #[test]
    fn single_file_path_appears_in_output() {
        let root = temp_dir("single_path");
        write_file(&root, "lib.rs", "pub fn hello() {}");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(out.contains("scanning: lib.rs"), "got: {out}");
    }

    #[test]
    fn multiple_files_count_is_correct() {
        let root = temp_dir("multi");
        write_file(&root, "a.rs", "fn a() {}");
        write_file(&root, "b.py", "def b(): pass");
        write_file(&root, "c.ts", "export const c = 1;");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(out.contains("Found 3 scannable files"), "got: {out}");
    }

    #[test]
    fn each_scannable_file_has_a_scanning_line() {
        let root = temp_dir("each_line");
        write_file(&root, "one.rs", "fn one() {}");
        write_file(&root, "two.go", "package main");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(out.contains("scanning: one.rs"), "got: {out}");
        assert!(out.contains("scanning: two.go"), "got: {out}");
    }

    #[test]
    fn non_scannable_files_do_not_appear_in_output() {
        let root = temp_dir("non_scan");
        write_file(&root, "README.md", "# hello");
        write_file(&root, "image.png", "PNG");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(out.contains("Found 0 scannable files"), "got: {out}");
        assert!(!out.contains("scanning:"), "got: {out}");
    }

    #[test]
    fn summary_line_contains_the_scanned_path() {
        let root = temp_dir("path_in_summary");
        let path_str = root.to_str().unwrap().to_string();

        let out = capture(&path_str);
        cleanup(&root);

        assert!(out.contains(&path_str), "path missing from summary: {out}");
    }

    #[test]
    fn summary_line_comes_before_scanning_lines() {
        let root = temp_dir("order");
        write_file(&root, "app.rs", "fn app() {}");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        let found_pos = out.find("Found").expect("no Found line");
        let scan_pos = out.find("scanning:").expect("no scanning line");
        assert!(found_pos < scan_pos, "summary must precede file lines");
    }

    #[test]
    fn nested_file_path_uses_forward_slashes() {
        let root = temp_dir("nested");
        write_file(&root, "src/lib.rs", "pub fn lib() {}");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(out.contains("scanning: src/lib.rs"), "got: {out}");
    }

    #[test]
    fn skipped_directories_are_not_counted() {
        let root = temp_dir("skip");
        write_file(&root, "src/main.rs", "fn main() {}");
        write_file(&root, "target/debug/main.rs", "fn main() {}");

        let out = capture(root.to_str().unwrap());
        cleanup(&root);

        assert!(out.contains("Found 1 scannable files"), "got: {out}");
        assert!(!out.contains("target"), "target/ should be skipped: {out}");
    }
}
