//! Integration tests for using codeconvert as a library

use codeconvert_core::{CaseConverter, CaseFormat};
use std::fs;

#[test]
fn test_library_basic_conversion() {
    // Create a temporary test file
    let test_dir = std::env::temp_dir().join("codeconvert_test_lib_basic");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "myVariable = 'test'\nanotherVar = 123").unwrap();

    // Use library to convert
    let converter = CaseConverter::new(
        CaseFormat::CamelCase,
        CaseFormat::SnakeCase,
        Some(vec![".py".to_string()]),
        false,
        false,
        String::new(),
        String::new(),
        None,
        None,
    ).unwrap();

    converter.process_directory(&test_dir).unwrap();

    // Verify conversion
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("my_variable"));
    assert!(content.contains("another_var"));
    assert!(!content.contains("myVariable"));
    assert!(!content.contains("anotherVar"));

    // Cleanup
    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_library_with_prefix() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_lib_prefix");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.js");
    fs::write(&test_file, "let userName = 'alice';").unwrap();

    let converter = CaseConverter::new(
        CaseFormat::CamelCase,
        CaseFormat::SnakeCase,
        Some(vec![".js".to_string()]),
        false,
        false,
        "old_".to_string(),
        String::new(),
        None,
        None,
    ).unwrap();

    converter.process_directory(&test_dir).unwrap();

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("old_user_name"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_library_with_suffix() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_lib_suffix");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.ts");
    fs::write(&test_file, "const myValue = 42;").unwrap();

    let converter = CaseConverter::new(
        CaseFormat::CamelCase,
        CaseFormat::SnakeCase,
        Some(vec![".ts".to_string()]),
        false,
        false,
        String::new(),
        "_v2".to_string(),
        None,
        None,
    ).unwrap();

    converter.process_directory(&test_dir).unwrap();

    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("my_value_v2"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_library_dry_run() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_lib_dry");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    let original_content = "myVariable = 'test'";
    fs::write(&test_file, original_content).unwrap();

    let converter = CaseConverter::new(
        CaseFormat::CamelCase,
        CaseFormat::SnakeCase,
        Some(vec![".py".to_string()]),
        false,
        true,  // dry_run = true
        String::new(),
        String::new(),
        None,
        None,
    ).unwrap();

    converter.process_directory(&test_dir).unwrap();

    // Verify file unchanged
    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, original_content);

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_library_recursive() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_lib_recursive");
    fs::create_dir_all(&test_dir).unwrap();

    // Create nested structure
    let sub_dir = test_dir.join("subdir");
    fs::create_dir_all(&sub_dir).unwrap();

    let file1 = test_dir.join("file1.py");
    let file2 = sub_dir.join("file2.py");

    fs::write(&file1, "topLevel = 1").unwrap();
    fs::write(&file2, "nestedVar = 2").unwrap();

    let converter = CaseConverter::new(
        CaseFormat::CamelCase,
        CaseFormat::SnakeCase,
        Some(vec![".py".to_string()]),
        true,  // recursive = true
        false,
        String::new(),
        String::new(),
        None,
        None,
    ).unwrap();

    converter.process_directory(&test_dir).unwrap();

    // Verify both files converted
    let content1 = fs::read_to_string(&file1).unwrap();
    let content2 = fs::read_to_string(&file2).unwrap();

    assert!(content1.contains("top_level"));
    assert!(content2.contains("nested_var"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_library_word_filter() {
    let test_dir = std::env::temp_dir().join("codeconvert_test_lib_filter");
    fs::create_dir_all(&test_dir).unwrap();

    let test_file = test_dir.join("test.py");
    fs::write(&test_file, "getUserName = lambda: 'alice'\nmyVariable = 123").unwrap();

    let converter = CaseConverter::new(
        CaseFormat::CamelCase,
        CaseFormat::SnakeCase,
        Some(vec![".py".to_string()]),
        false,
        false,
        String::new(),
        String::new(),
        None,
        Some("^get.*".to_string()),  // Only convert identifiers starting with "get"
    ).unwrap();

    converter.process_directory(&test_dir).unwrap();

    let content = fs::read_to_string(&test_file).unwrap();

    // getUserName should be converted
    assert!(content.contains("get_user_name"));

    // myVariable should NOT be converted (doesn't match filter)
    assert!(content.contains("myVariable"));
    assert!(!content.contains("my_variable"));

    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_library_all_case_formats() {
    // Test conversion between all major formats
    let test_cases = vec![
        (CaseFormat::CamelCase, CaseFormat::SnakeCase, "firstName", "first_name"),
        (CaseFormat::SnakeCase, CaseFormat::CamelCase, "first_name", "firstName"),
        (CaseFormat::PascalCase, CaseFormat::KebabCase, "FirstName", "first-name"),
        (CaseFormat::KebabCase, CaseFormat::PascalCase, "first-name", "FirstName"),
        (CaseFormat::SnakeCase, CaseFormat::ScreamingSnakeCase, "first_name", "FIRST_NAME"),
        (CaseFormat::KebabCase, CaseFormat::ScreamingKebabCase, "first-name", "FIRST-NAME"),
    ];

    for (idx, (from, to, input, expected)) in test_cases.iter().enumerate() {
        let test_dir = std::env::temp_dir().join(format!("codeconvert_test_lib_formats_{}", idx));
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.txt");
        fs::write(&test_file, input).unwrap();

        let converter = CaseConverter::new(
            *from,
            *to,
            Some(vec![".txt".to_string()]),
            false,
            false,
            String::new(),
            String::new(),
            None,
            None,
        ).unwrap();

        converter.process_directory(&test_dir).unwrap();

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, *expected, "Failed conversion from {:?} to {:?}", from, to);

        fs::remove_dir_all(&test_dir).unwrap();
    }
}
