//! Integration tests for CLI functionality

use std::fs;
use std::process::Command;

fn get_binary_path() -> std::path::PathBuf {
    // Get the path to the compiled binary using cargo's test infrastructure
    let mut path = std::env::current_exe()
        .expect("Failed to get current executable path")
        .parent()
        .expect("Failed to get parent directory")
        .parent()
        .expect("Failed to get parent directory")
        .to_path_buf();

    // The binary will be in the same directory as the test executable
    path.push("codeconvert");

    if !path.exists() {
        // Fallback: try to use cargo to build and get the path
        let _output = Command::new("cargo")
            .args(&["build", "-p", "codeconvert", "--message-format=json"])
            .output()
            .expect("Failed to build codeconvert");

        // Parse the JSON to find the binary path (simplified - just return the default path)
        std::env::current_dir()
            .expect("Failed to get current directory")
            .join("target/debug/codeconvert")
    } else {
        path
    }
}

#[test]
fn test_cli_version() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.2.0"));
}

#[test]
fn test_cli_help() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("codeconvert"));
    assert!(stdout.contains("--from-camel"));
    assert!(stdout.contains("--to-snake"));
}

#[test]
fn test_cli_basic_conversion() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_cli_basic");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "myVariable = 'test'").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--to-snake"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("my_variable"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_dry_run() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_cli_dry");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    let original = "myVariable = 'test'";
    fs::write(&test_file, original).unwrap();

    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--to-snake", "--dry-run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());

    // File should be unchanged
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, original);

    // Output should indicate what would be converted
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Would convert"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_recursive() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_cli_recursive");
    fs::create_dir_all(&test_dir).unwrap();

    let sub_dir = test_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    let file1 = test_dir.join("file1.py");
    let file2 = sub_dir.join("file2.py");

    fs::write(&file1, "topLevel = 1").unwrap();
    fs::write(&file2, "nestedVar = 2").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--to-snake", "-r"])
        .arg(&test_dir)
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());

    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();

    assert!(content1.contains("top_level"));
    assert!(content2.contains("nested_var"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_with_prefix() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_cli_prefix");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "myVariable = 'test'").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--to-snake", "--prefix", "old_"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("old_my_variable"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_with_suffix() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_cli_suffix");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "myVariable = 'test'").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--to-snake", "--suffix", "_new"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("my_variable_new"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_word_filter() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_cli_filter");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "getUserName = 'alice'\nmyVariable = 123").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--to-snake", "--word-filter", "^get.*"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("get_user_name"));
    assert!(content.contains("myVariable")); // Should not be converted

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_multiple_extensions() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_cli_exts");
    fs::create_dir_all(&test_dir).unwrap();

    let py_file = test_dir.join("test.py");
    let js_file = test_dir.join("test.js");
    let txt_file = test_dir.join("test.txt");

    fs::write(&py_file, "myVariable = 1").unwrap();
    fs::write(&js_file, "myVariable = 2").unwrap();
    fs::write(&txt_file, "myVariable = 3").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--to-snake", "-e", ".py", "-e", ".js"])
        .arg(&test_dir)
        .output()
        .expect("Failed to execute codeconvert");

    assert!(output.status.success());

    let py_content = fs::read_to_string(&py_file).unwrap();
    let js_content = fs::read_to_string(&js_file).unwrap();
    let txt_content = fs::read_to_string(&txt_file).unwrap();

    assert!(py_content.contains("my_variable"));
    assert!(js_content.contains("my_variable"));
    assert!(txt_content.contains("myVariable")); // Should not be converted

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_error_missing_from() {
    let output = Command::new(get_binary_path())
        .args(&["--to-snake", "dummy.py"])
        .output()
        .expect("Failed to execute codeconvert");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required") || stderr.contains("from"));
}

#[test]
fn test_cli_error_missing_to() {
    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "dummy.py"])
        .output()
        .expect("Failed to execute codeconvert");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("required") || stderr.contains("to"));
}

#[test]
fn test_cli_error_conflicting_from() {
    let output = Command::new(get_binary_path())
        .args(&["--from-camel", "--from-snake", "--to-kebab", "dummy.py"])
        .output()
        .expect("Failed to execute codeconvert");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot be used with"));
}

#[test]
fn test_cli_all_format_combinations() {
    let test_cases = vec![
        ("--from-camel", "--to-pascal", "myName", "MyName"),
        ("--from-pascal", "--to-snake", "MyName", "my_name"),
        ("--from-snake", "--to-kebab", "my_name", "my-name"),
        ("--from-kebab", "--to-screaming-snake", "my-name", "MY_NAME"),
        ("--from-screaming-snake", "--to-camel", "MY_NAME", "myName"),
    ];

    for (idx, (from_arg, to_arg, input, expected)) in test_cases.iter().enumerate() {
        let test_dir = std::env::temp_dir().join(format!("codeconvert_test_cli_combo_{}", idx));
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.txt");
        fs::write(&test_file, input).unwrap();

        let output = Command::new(get_binary_path())
            .args(&[from_arg, to_arg, "-e", ".txt"])
            .arg(&test_file)
            .output()
            .expect("Failed to execute codeconvert");

        assert!(output.status.success(), "Failed for {} -> {}", from_arg, to_arg);

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, *expected, "Failed for {} -> {}", from_arg, to_arg);

        fs::remove_dir_all(&test_dir).unwrap();
    }
}

// Whitespace cleaning tests

#[test]
fn test_cli_clean_basic() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_clean_basic");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.txt");
    fs::write(&test_file, "line1   \nline2\t\nline3\n").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["clean"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert clean");

    assert!(output.status.success());

    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "line1\nline2\nline3\n");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Cleaned"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_clean_dry_run() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_clean_dry");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.txt");
    let original = "line1   \nline2\t\nline3\n";
    fs::write(&test_file, original).unwrap();

    let output = Command::new(get_binary_path())
        .args(&["clean", "--dry-run"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert clean");

    assert!(output.status.success());

    // File should be unchanged
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, original);

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[DRY-RUN]") || stdout.contains("Would clean"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_clean_recursive() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_clean_recursive");
    fs::create_dir_all(&test_dir).unwrap();

    let sub_dir = test_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    let file1 = test_dir.join("file1.txt");
    let file2 = sub_dir.join("file2.txt");

    fs::write(&file1, "line1   \n").unwrap();
    fs::write(&file2, "line2\t\n").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["clean", "-r"])
        .arg(&test_dir)
        .output()
        .expect("Failed to execute codeconvert clean");

    assert!(output.status.success());

    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();

    assert_eq!(content1, "line1\n");
    assert_eq!(content2, "line2\n");

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_clean_extension_filtering() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_clean_exts");
    fs::create_dir_all(&test_dir).unwrap();

    let py_file = test_dir.join("test.py");
    let txt_file = test_dir.join("test.txt");

    fs::write(&py_file, "line1   \n").unwrap();
    fs::write(&txt_file, "line1   \n").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["clean", "-e", ".py"])
        .arg(&test_dir)
        .output()
        .expect("Failed to execute codeconvert clean");

    assert!(output.status.success());

    let py_content = fs::read_to_string(&py_file).unwrap();
    let txt_content = fs::read_to_string(&txt_file).unwrap();

    assert_eq!(py_content, "line1\n"); // Should be cleaned
    assert_eq!(txt_content, "line1   \n"); // Should not be cleaned

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_clean_no_changes_needed() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_clean_no_changes");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.txt");
    fs::write(&test_file, "line1\nline2\nline3\n").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["clean"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert clean");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("No files needed cleaning"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_cli_clean_help() {
    let output = Command::new(get_binary_path())
        .args(&["clean", "--help"])
        .output()
        .expect("Failed to execute codeconvert clean --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Remove trailing whitespace"));
}

#[test]
fn test_cli_convert_subcommand() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_convert_subcommand");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "myVariable = 'test'").unwrap();

    let output = Command::new(get_binary_path())
        .args(&["convert", "--from-camel", "--to-snake"])
        .arg(&test_file)
        .output()
        .expect("Failed to execute codeconvert convert");

    assert!(output.status.success());

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("my_variable"));

    fs::remove_dir_all(&test_dir).unwrap();
}
