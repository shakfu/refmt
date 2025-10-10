//! Case converter implementation for file processing

use crate::case::CaseFormat;
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Main converter for transforming case formats in files
pub struct CaseConverter {
    from_format: CaseFormat,
    to_format: CaseFormat,
    file_extensions: Vec<String>,
    recursive: bool,
    dry_run: bool,
    prefix: String,
    suffix: String,
    glob_pattern: Option<glob::Pattern>,
    word_filter: Option<Regex>,
    source_pattern: Regex,
}

impl CaseConverter {
    /// Creates a new case converter
    pub fn new(
        from_format: CaseFormat,
        to_format: CaseFormat,
        file_extensions: Option<Vec<String>>,
        recursive: bool,
        dry_run: bool,
        prefix: String,
        suffix: String,
        glob_pattern: Option<String>,
        word_filter: Option<String>,
    ) -> crate::Result<Self> {
        let file_extensions = file_extensions.unwrap_or_else(|| {
            vec![
                ".c", ".h", ".py", ".md", ".js", ".ts", ".java", ".cpp", ".hpp",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect()
        });

        let source_pattern = Regex::new(from_format.pattern())?;
        let glob_pattern = match glob_pattern {
            Some(pattern) => Some(glob::Pattern::new(&pattern)?),
            None => None,
        };
        let word_filter = match word_filter {
            Some(pattern) => Some(Regex::new(&pattern)?),
            None => None,
        };

        Ok(CaseConverter {
            from_format,
            to_format,
            file_extensions,
            recursive,
            dry_run,
            prefix,
            suffix,
            glob_pattern,
            word_filter,
            source_pattern,
        })
    }

    /// Converts a single identifier
    fn convert(&self, name: &str) -> String {
        // Apply word filter if provided
        if let Some(ref filter) = self.word_filter {
            if !filter.is_match(name) {
                return name.to_string();
            }
        }

        let words = self.from_format.split_words(name);
        self.to_format.join_words(&words, &self.prefix, &self.suffix)
    }

    /// Checks if a file matches the glob pattern
    fn matches_glob(&self, filepath: &Path, base_path: &Path) -> bool {
        if let Some(ref pattern) = self.glob_pattern {
            // Match against the filename
            if let Some(filename) = filepath.file_name() {
                if pattern.matches(filename.to_string_lossy().as_ref()) {
                    return true;
                }
            }

            // Also try matching against the full relative path
            if let Ok(rel_path) = filepath.strip_prefix(base_path) {
                if pattern.matches_path(rel_path) {
                    return true;
                }
            }

            false
        } else {
            true
        }
    }

    /// Processes a single file
    pub fn process_file(&self, filepath: &Path, base_path: &Path) -> crate::Result<()> {
        // Check file extension
        let extension = filepath
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e));

        if let Some(ext) = extension {
            if !self.file_extensions.contains(&ext) {
                return Ok(());
            }
        } else {
            return Ok(());
        }

        // Check glob pattern
        if !self.matches_glob(filepath, base_path) {
            return Ok(());
        }

        // Read file content
        let content = fs::read_to_string(filepath)?;

        // Replace all matches of the source pattern
        let modified_content = self.source_pattern.replace_all(&content, |caps: &regex::Captures| {
            self.convert(&caps[0])
        });

        if content != modified_content {
            if self.dry_run {
                println!("Would convert '{}'", filepath.display());
            } else {
                fs::write(filepath, modified_content.as_ref())?;
                println!("Converted '{}'", filepath.display());
            }
        } else if !self.dry_run {
            println!("No changes needed in '{}'", filepath.display());
        }

        Ok(())
    }

    /// Processes a directory or file
    pub fn process_directory(&self, directory_path: &Path) -> crate::Result<()> {
        if !directory_path.exists() {
            eprintln!("Path '{}' does not exist.", directory_path.display());
            return Ok(());
        }

        // If it's a single file, process it directly
        if directory_path.is_file() {
            if let Some(parent) = directory_path.parent() {
                self.process_file(directory_path, parent)?;
            } else {
                self.process_file(directory_path, Path::new("."))?;
            }
            return Ok(());
        }

        // Otherwise, process directory
        if !directory_path.is_dir() {
            eprintln!("Path '{}' is not a directory or file.", directory_path.display());
            return Ok(());
        }

        if self.recursive {
            for entry in WalkDir::new(directory_path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    if let Err(e) = self.process_file(entry.path(), directory_path) {
                        eprintln!("Error processing file '{}': {}", entry.path().display(), e);
                    }
                }
            }
        } else {
            for entry in fs::read_dir(directory_path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Err(e) = self.process_file(&path, directory_path) {
                        eprintln!("Error processing file '{}': {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camel_to_snake() {
        let words = CaseFormat::CamelCase.split_words("firstName");
        assert_eq!(words, vec!["first", "name"]);
        assert_eq!(
            CaseFormat::SnakeCase.join_words(&words, "", ""),
            "first_name"
        );
    }

    #[test]
    fn test_snake_to_camel() {
        let words = CaseFormat::SnakeCase.split_words("first_name");
        assert_eq!(words, vec!["first", "name"]);
        assert_eq!(
            CaseFormat::CamelCase.join_words(&words, "", ""),
            "firstName"
        );
    }

    #[test]
    fn test_pascal_to_kebab() {
        let words = CaseFormat::PascalCase.split_words("FirstName");
        assert_eq!(words, vec!["first", "name"]);
        assert_eq!(
            CaseFormat::KebabCase.join_words(&words, "", ""),
            "first-name"
        );
    }

    #[test]
    fn test_kebab_to_screaming_snake() {
        let words = CaseFormat::KebabCase.split_words("first-name");
        assert_eq!(words, vec!["first", "name"]);
        assert_eq!(
            CaseFormat::ScreamingSnakeCase.join_words(&words, "", ""),
            "FIRST_NAME"
        );
    }

    #[test]
    fn test_camel_pattern_match() {
        let pattern = Regex::new(CaseFormat::CamelCase.pattern()).unwrap();
        assert!(pattern.is_match("firstName"));
        assert!(pattern.is_match("myVariableName"));
        assert!(!pattern.is_match("firstname"));
        assert!(!pattern.is_match("FirstName")); // PascalCase, not camelCase
    }

    #[test]
    fn test_pascal_pattern_match() {
        let pattern = Regex::new(CaseFormat::PascalCase.pattern()).unwrap();
        assert!(pattern.is_match("FirstName"));
        assert!(pattern.is_match("MyVariableName"));
        assert!(!pattern.is_match("firstName")); // camelCase, not PascalCase
        assert!(!pattern.is_match("FIRSTNAME")); // Not PascalCase
    }

    #[test]
    fn test_snake_pattern_match() {
        let pattern = Regex::new(CaseFormat::SnakeCase.pattern()).unwrap();
        assert!(pattern.is_match("first_name"));
        assert!(pattern.is_match("my_variable_name"));
        assert!(!pattern.is_match("firstname"));
        assert!(!pattern.is_match("FIRST_NAME")); // SCREAMING_SNAKE_CASE
    }
}
