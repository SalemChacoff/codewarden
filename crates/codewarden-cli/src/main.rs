//! `codewarden-cli` – Command-line interface for CodeWarden.

mod commands;

#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    Scan { path: &'a str },
    Diff { path: &'a str },
    Usage,
}

pub fn parse_args(args: &[String]) -> Command<'_> {
    match args {
        [_, cmd, path] if cmd == "scan" => Command::Scan { path },
        [_, cmd, path] if cmd == "diff" => Command::Diff { path },
        _ => Command::Usage,
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match parse_args(&args) {
        Command::Scan { path } => {
            commands::scan::run(path);
        }
        Command::Diff { path } => {
            commands::diff::run(path);
        }
        Command::Usage => {
            eprintln!("CodeWarden v{}", codewarden_core::version());
            eprintln!();
            eprintln!("USAGE:");
            eprintln!("  codewarden scan <path>   – scan all files in a directory");
            eprintln!("  codewarden diff <path>   – analyse `git diff HEAD` in a git repo");
            eprintln!("  codewarden diff -         – read a unified diff from stdin");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn scan_command_is_parsed_correctly() {
        let a = args(&["codewarden", "scan", "/tmp/project"]);
        assert_eq!(parse_args(&a), Command::Scan { path: "/tmp/project" });
    }

    #[test]
    fn scan_command_preserves_exact_path() {
        let a = args(&["codewarden", "scan", "relative/path/to/code"]);
        assert_eq!(
            parse_args(&a),
            Command::Scan { path: "relative/path/to/code" }
        );
    }

    #[test]
    fn no_arguments_shows_usage() {
        let a = args(&["codewarden"]);
        assert_eq!(parse_args(&a), Command::Usage);
    }

    #[test]
    fn unknown_command_shows_usage() {
        let a = args(&["codewarden", "lint", "/tmp"]);
        assert_eq!(parse_args(&a), Command::Usage);
    }

    #[test]
    fn scan_without_path_shows_usage() {
        let a = args(&["codewarden", "scan"]);
        assert_eq!(parse_args(&a), Command::Usage);
    }

    #[test]
    fn too_many_arguments_shows_usage() {
        let a = args(&["codewarden", "scan", "/tmp", "extra"]);
        assert_eq!(parse_args(&a), Command::Usage);
    }

    #[test]
    fn empty_args_shows_usage() {
        assert_eq!(parse_args(&[]), Command::Usage);
    }

    #[test]
    fn command_matching_is_case_sensitive() {
        let a = args(&["codewarden", "Scan", "/tmp"]);
        assert_eq!(parse_args(&a), Command::Usage);
    }

    #[test]
    fn scan_command_runs_without_panic_on_empty_dir() {
        let dir = std::env::temp_dir()
            .join(format!("cw_cli_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");

        commands::scan::run(dir.to_str().expect("valid path"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn scan_command_runs_without_panic_on_nonexistent_path() {
        commands::scan::run("/this/path/does/not/exist_cw_cli_test");
    }

    #[test]
    fn diff_command_is_parsed_correctly() {
        let a = args(&["codewarden", "diff", "/repo"]);
        assert_eq!(parse_args(&a), Command::Diff { path: "/repo" });
    }

    #[test]
    fn diff_stdin_flag_is_parsed() {
        let a = args(&["codewarden", "diff", "-"]);
        assert_eq!(parse_args(&a), Command::Diff { path: "-" });
    }

    #[test]
    fn diff_without_path_shows_usage() {
        let a = args(&["codewarden", "diff"]);
        assert_eq!(parse_args(&a), Command::Usage);
    }
}
