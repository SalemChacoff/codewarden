use std::path::Path;

pub fn should_scan(path: &Path) -> bool {
    // Skip binary, hidden, and common noise directories
    if is_hidden(path) || is_ignored_dir(path) {
        return false;
    }

    // Scan base files
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some(
            "rs" | "js"
                | "ts"
                | "py"
                | "go"
                | "java"
                | "php"
                | "sh"
                | "bash"
                | "yaml"
                | "yml"
                | "toml"
                | "json"
                | "dockerfile"
                | "tf"
                | "rb"
                | "cs"
                | "cpp"
                | "c"
        )
    )
}

fn is_hidden(path: &Path) -> bool {
    use std::path::Component;
    path.components().any(|c| match c {
        Component::Normal(name) => name.to_str().map(|s| s.starts_with('.')).unwrap_or(false),
        _ => false,
    })
}

fn is_ignored_dir(path: &Path) -> bool {
    path.components().any(|c| {
        matches!(
            c.as_os_str().to_str(),
            Some("target" | "node_modules" | "vendor" | ".git" | "dist" | "build" | "__pycache__")
        )
    })
}

pub fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|n| n.to_str()),
        Some(
            "target"
                | "node_modules"
                | "vendor"
                | ".git"
                | "dist"
                | "build"
                | "__pycache__"
                | ".idea"
                | ".vscode"
                | "out"
                | "bin"
                | "obj"
        )
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    mod should_scan_allowed_extensions {
        use super::*;

        macro_rules! assert_scanned {
            ($path:expr) => {
                assert!(
                    should_scan(Path::new($path)),
                    "expected '{}' to be scanned",
                    $path
                );
            };
        }

        #[test]
        fn rust_file() { assert_scanned!("src/main.rs"); }

        #[test]
        fn javascript_file() { assert_scanned!("app/index.js"); }

        #[test]
        fn typescript_file() { assert_scanned!("app/index.ts"); }

        #[test]
        fn python_file() { assert_scanned!("scripts/deploy.py"); }

        #[test]
        fn go_file() { assert_scanned!("cmd/main.go"); }

        #[test]
        fn java_file() { assert_scanned!("src/Main.java"); }

        #[test]
        fn php_file() { assert_scanned!("public/index.php"); }

        #[test]
        fn shell_sh_file() { assert_scanned!("scripts/setup.sh"); }

        #[test]
        fn shell_bash_file() { assert_scanned!("scripts/run.bash"); }

        #[test]
        fn yaml_file() { assert_scanned!("config/app.yaml"); }

        #[test]
        fn yml_file() { assert_scanned!("config/app.yml"); }

        #[test]
        fn toml_file() { assert_scanned!("Cargo.toml"); }

        #[test]
        fn json_file() { assert_scanned!("package.json"); }

        #[test]
        fn dockerfile_extension() { assert_scanned!("deploy/app.dockerfile"); }

        #[test]
        fn terraform_file() { assert_scanned!("infra/main.tf"); }

        #[test]
        fn ruby_file() { assert_scanned!("lib/helper.rb"); }

        #[test]
        fn csharp_file() { assert_scanned!("src/Program.cs"); }

        #[test]
        fn cpp_file() { assert_scanned!("src/main.cpp"); }

        #[test]
        fn c_file() { assert_scanned!("src/util.c"); }
    }

    mod should_scan_rejected_extensions {
        use super::*;

        macro_rules! assert_not_scanned {
            ($path:expr) => {
                assert!(
                    !should_scan(Path::new($path)),
                    "expected '{}' NOT to be scanned",
                    $path
                );
            };
        }

        #[test]
        fn binary_exe() { assert_not_scanned!("target/debug/app.exe"); }

        #[test]
        fn compiled_rlib() { assert_not_scanned!("target/debug/libfoo.rlib"); }

        #[test]
        fn image_png() { assert_not_scanned!("assets/logo.png"); }

        #[test]
        fn image_jpg() { assert_not_scanned!("assets/photo.jpg"); }

        #[test]
        fn pdf_file() { assert_not_scanned!("docs/report.pdf"); }

        #[test]
        fn zip_archive() { assert_not_scanned!("dist/release.zip"); }

        #[test]
        fn lock_file() { assert_not_scanned!("Cargo.lock"); }

        #[test]
        fn markdown_file() { assert_not_scanned!("README.md"); }

        #[test]
        fn no_extension() { assert_not_scanned!("Makefile"); }

        #[test]
        fn dockerfile_no_extension_is_not_scanned() {
            assert_not_scanned!("Dockerfile");
        }
    }

    mod should_scan_hidden_files {
        use super::*;

        #[test]
        fn dotfile_is_skipped() {
            assert!(!should_scan(Path::new(".env")));
        }

        #[test]
        fn dotfile_with_allowed_extension_is_skipped() {
            assert!(!should_scan(Path::new(".hidden.rs")));
        }

        #[test]
        fn file_inside_hidden_dir_is_skipped() {
            assert!(!should_scan(Path::new(".github/workflows/ci.yml")));
        }

        #[test]
        fn normal_file_is_not_hidden() {
            assert!(should_scan(Path::new("src/main.rs")));
        }
    }

    mod should_scan_ignored_dirs {
        use super::*;

        #[test]
        fn file_in_target_is_skipped() {
            assert!(!should_scan(Path::new("target/debug/build.rs")));
        }

        #[test]
        fn file_in_node_modules_is_skipped() {
            assert!(!should_scan(Path::new("node_modules/lodash/index.js")));
        }

        #[test]
        fn file_in_vendor_is_skipped() {
            assert!(!should_scan(Path::new("vendor/lib/util.go")));
        }

        #[test]
        fn file_in_dist_is_skipped() {
            assert!(!should_scan(Path::new("dist/bundle.js")));
        }

        #[test]
        fn file_in_build_is_skipped() {
            assert!(!should_scan(Path::new("build/output.js")));
        }

        #[test]
        fn file_in_pycache_is_skipped() {
            assert!(!should_scan(Path::new("app/__pycache__/module.py")));
        }

        #[test]
        fn nested_ignored_dir_is_skipped() {
            assert!(!should_scan(Path::new("crates/foo/target/debug/foo.rs")));
        }
    }

    mod should_scan_relative_paths {
        use super::*;

        #[test]
        fn curdir_prefix_is_not_hidden() {
            assert!(should_scan(Path::new("./src/main.rs")));
        }

        #[test]
        fn curdir_prefix_with_toml() {
            assert!(should_scan(Path::new("./Cargo.toml")));
        }

        #[test]
        fn curdir_prefix_inside_ignored_dir_is_still_skipped() {
            assert!(!should_scan(Path::new("./target/debug/foo.rs")));
        }

        #[test]
        fn nested_curdir_prefix() {
            assert!(should_scan(Path::new("./crates/core/src/lib.rs")));
        }
    }


    mod should_skip_dir_tests {
        use super::*;

        macro_rules! assert_skipped {
            ($path:expr) => {
                assert!(
                    should_skip_dir(Path::new($path)),
                    "expected dir '{}' to be skipped",
                    $path
                );
            };
        }

        macro_rules! assert_not_skipped {
            ($path:expr) => {
                assert!(
                    !should_skip_dir(Path::new($path)),
                    "expected dir '{}' NOT to be skipped",
                    $path
                );
            };
        }

        #[test]
        fn target_dir() { assert_skipped!("target"); }

        #[test]
        fn target_dir_nested() { assert_skipped!("crates/foo/target"); }

        #[test]
        fn node_modules_dir() { assert_skipped!("node_modules"); }

        #[test]
        fn vendor_dir() { assert_skipped!("vendor"); }

        #[test]
        fn git_dir() { assert_skipped!(".git"); }

        #[test]
        fn dist_dir() { assert_skipped!("dist"); }

        #[test]
        fn build_dir() { assert_skipped!("build"); }

        #[test]
        fn pycache_dir() { assert_skipped!("__pycache__"); }

        #[test]
        fn idea_dir() { assert_skipped!(".idea"); }

        #[test]
        fn vscode_dir() { assert_skipped!(".vscode"); }

        #[test]
        fn out_dir() { assert_skipped!("out"); }

        #[test]
        fn bin_dir() { assert_skipped!("bin"); }

        #[test]
        fn obj_dir() { assert_skipped!("obj"); }

        #[test]
        fn src_dir_is_not_skipped() { assert_not_skipped!("src"); }

        #[test]
        fn crates_dir_is_not_skipped() { assert_not_skipped!("crates"); }

        #[test]
        fn lib_dir_is_not_skipped() { assert_not_skipped!("lib"); }

        #[test]
        fn tests_dir_is_not_skipped() { assert_not_skipped!("tests"); }

        #[test]
        fn docs_dir_is_not_skipped() { assert_not_skipped!("docs"); }
    }
}
