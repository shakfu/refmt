//! Safety tests: transformers must never modify version-control metadata,
//! and must not silently no-op on ordinary relative paths.
//!
//! These guard the invariant that `reformat` only ever touches the files a
//! user actually pointed it at. A transformer that walks into `.git/` can
//! destroy a repository irrecoverably, since renames are not journalled.

use std::path::Path;
use std::process::Command;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_reformat")
}

/// Runs a git command in `dir`, returning true on success.
fn git(dir: &Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Creates a repository with one committed file whose name and contents are
/// deliberately transformable (mixed case, trailing whitespace).
fn init_repo(dir: &Path) {
    assert!(git(dir, &["init", "-q"]), "git init failed");
    std::fs::write(dir.join("README.md"), "Title  \ntext  \n").unwrap();
    std::fs::write(dir.join("Notes.md"), "Body  \n").unwrap();
    assert!(git(dir, &["add", "."]), "git add failed");
    assert!(
        git(
            dir,
            &[
                "-c",
                "user.email=test@example.com",
                "-c",
                "user.name=test",
                "commit",
                "-qm",
                "initial",
            ],
        ),
        "git commit failed"
    );
}

/// Asserts the repository is still intact and its metadata untouched.
fn assert_repo_intact(dir: &Path, what: &str) {
    for name in ["HEAD", "config", "index", "description"] {
        assert!(
            dir.join(".git").join(name).exists(),
            "{what}: .git/{name} is missing -- the repository metadata was modified"
        );
    }
    assert!(
        git(dir, &["rev-parse", "--git-dir"]),
        "{what}: `git rev-parse` failed -- the repository was destroyed"
    );
    assert!(
        git(dir, &["status", "--porcelain"]),
        "{what}: `git status` failed -- the repository was destroyed"
    );
}

/// Runs reformat with `args` from inside `dir`.
fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(binary())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("failed to run reformat")
}

#[test]
fn test_rename_files_does_not_touch_git_metadata() {
    if !git_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    init_repo(dir);

    run(dir, &["rename_files", "--to-uppercase", "."]);

    assert_repo_intact(dir, "rename_files --to-uppercase");
    // The rename must still have done its job on the tracked file.
    assert!(
        dir.join("README.MD").exists() || dir.join("README.md").exists(),
        "rename_files did nothing at all -- the test would pass vacuously"
    );
}

#[test]
fn test_default_command_does_not_touch_git_metadata() {
    if !git_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    init_repo(dir);

    run(dir, &["-r", "."]);

    assert_repo_intact(dir, "default command (-r .)");
    // The default pipeline lowercases filenames; confirm it actually ran.
    assert!(
        dir.join("notes.md").exists(),
        "default command did not lowercase Notes.md -- the test would pass vacuously"
    );
}

#[test]
fn test_convert_does_not_touch_git_metadata() {
    if !git_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    init_repo(dir);
    std::fs::write(dir.join("code.py"), "someName = 1\n").unwrap();

    run(dir, &["convert", "--from-camel", "--to-snake", "-r", "."]);

    assert_repo_intact(dir, "convert --from-camel --to-snake");
    let converted = std::fs::read_to_string(dir.join("code.py")).unwrap();
    assert_eq!(
        converted, "some_name = 1\n",
        "convert did not process the target file -- the test would pass vacuously"
    );
}

#[test]
fn test_clean_does_not_touch_git_metadata() {
    if !git_available() {
        eprintln!("skipping: git not available");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    init_repo(dir);

    run(dir, &["clean", "-r", "."]);

    assert_repo_intact(dir, "clean");
}

#[test]
fn test_transformers_skip_build_directories() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    for vendored in ["node_modules", "target", "__pycache__", ".venv"] {
        let sub = dir.join(vendored);
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("Vendored.md"), "text  \n").unwrap();
    }
    std::fs::write(dir.join("Own.md"), "text  \n").unwrap();

    run(dir, &["rename_files", "--to-lowercase", "."]);
    run(dir, &["clean", "-r", "."]);

    for vendored in ["node_modules", "target", "__pycache__", ".venv"] {
        let sub = dir.join(vendored);
        assert!(
            sub.join("Vendored.md").exists(),
            "{vendored}/Vendored.md was renamed -- build directories must be skipped"
        );
        assert_eq!(
            std::fs::read_to_string(sub.join("Vendored.md")).unwrap(),
            "text  \n",
            "{vendored}/Vendored.md was cleaned -- build directories must be skipped"
        );
    }
    assert!(
        dir.join("own.md").exists(),
        "the tool skipped everything, including files it should have processed"
    );
}

/// `reformat clean .` is the most natural invocation of the tool and must not
/// be a silent no-op. The `.` component is not a hidden directory.
#[test]
fn test_relative_dot_path_is_processed() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    std::fs::write(dir.join("a.txt"), "line1  \nline2  \n").unwrap();

    run(dir, &["clean", "-r", "."]);

    assert_eq!(
        std::fs::read_to_string(dir.join("a.txt")).unwrap(),
        "line1\nline2\n",
        "`clean .` did nothing -- a leading `.` path component must not be treated as hidden"
    );
}

/// An explicitly named hidden file is still skipped: the exclusion applies to
/// real directory names, not to `.` or `..` path components.
#[test]
fn test_explicit_hidden_file_is_still_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    std::fs::write(dir.join(".hidden.txt"), "line1  \n").unwrap();

    run(dir, &["clean", ".hidden.txt"]);

    assert_eq!(
        std::fs::read_to_string(dir.join(".hidden.txt")).unwrap(),
        "line1  \n",
        "hidden files must be skipped even when named explicitly"
    );
}
