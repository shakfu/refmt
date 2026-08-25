//! Whitespace cleaning transformer

use std::fs;
use std::path::Path;

/// Options for whitespace cleaning
#[derive(Debug, Clone)]
pub struct WhitespaceOptions {
    /// Remove trailing whitespace from lines
    pub remove_trailing: bool,
    /// File extensions to process
    pub file_extensions: Vec<String>,
    /// Process directories recursively
    pub recursive: bool,
    /// Dry run mode (don't modify files)
    pub dry_run: bool,
}

impl Default for WhitespaceOptions {
    fn default() -> Self {
        WhitespaceOptions {
            remove_trailing: true,
            file_extensions: vec![
                ".py", ".pyx", ".pxd", ".pxi", ".c", ".h", ".cpp", ".hpp", ".rs", ".go", ".java",
                ".js", ".ts", ".jsx", ".tsx", ".md", ".qmd", ".txt",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            recursive: true,
            dry_run: false,
        }
    }
}

/// Whitespace cleaner for removing trailing whitespace from files
pub struct WhitespaceCleaner {
    options: WhitespaceOptions,
}

impl WhitespaceCleaner {
    /// Creates a new whitespace cleaner with the given options
    pub fn new(options: WhitespaceOptions) -> Self {
        WhitespaceCleaner { options }
    }

    /// Creates a cleaner with default options
    pub fn with_defaults() -> Self {
        WhitespaceCleaner {
            options: WhitespaceOptions::default(),
        }
    }

    /// Checks if a file should be processed
    fn should_process(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        // Skip hidden entries and build/vendor directories (see crate::walk)
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_none_or(|n| crate::walk::is_excluded_component(n, crate::walk::DEFAULT_SKIP_DIRS))
        {
            return false;
        }

        // Check file extension
        if let Some(ext) = path.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            self.options.file_extensions.contains(&ext_str)
        } else {
            false
        }
    }

    /// Removes trailing whitespace from a single file
    pub fn clean_file(&self, path: &Path) -> crate::Result<usize> {
        if !self.should_process(path) {
            return Ok(0);
        }

        let content = match crate::text::read_text(path)? {
            Some(c) => c,
            None => return Ok(0),
        };
        let mut cleaned_content = String::with_capacity(content.len());
        let mut modified_count = 0;

        // Split so that each line's terminator stays attached to that line.
        // Trimming must only touch the body: rejoining with a fixed "\n"
        // would rewrite a CRLF file as LF as a side effect of stripping
        // whitespace.
        for (body, terminator) in crate::lines::split_lines(&content) {
            let cleaned = if self.options.remove_trailing {
                body.trim_end()
            } else {
                body
            };
            if cleaned != body {
                modified_count += 1;
            }
            cleaned_content.push_str(cleaned);
            cleaned_content.push_str(terminator);
        }

        if modified_count > 0 {
            if self.options.dry_run {
                log::info!(
                    "Would clean {} lines in '{}'",
                    modified_count,
                    path.display()
                );
            } else {
                fs::write(path, cleaned_content)?;
                log::info!("Cleaned {} lines in '{}'", modified_count, path.display());
            }
        }

        Ok(modified_count)
    }

    /// Processes a directory or file
    pub fn process(&self, path: &Path) -> crate::Result<(usize, usize)> {
        let mut total_files = 0;
        let mut total_lines = 0;

        if path.is_file() {
            let lines = self.clean_file(path)?;
            if lines > 0 {
                total_files = 1;
                total_lines = lines;
            }
        } else if path.is_dir() {
            for entry in crate::walk::walk_files(path, self.options.recursive) {
                let lines = self.clean_file(entry.path())?;
                if lines > 0 {
                    total_files += 1;
                    total_lines += lines;
                }
            }
        }

        Ok((total_files, total_lines))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A binary or non-UTF-8 file carrying a processed extension must be
    /// skipped without aborting the walk over its siblings.
    #[test]
    fn test_binary_and_non_utf8_files_are_skipped() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("a_binary.txt"), [0x00, 0x01, 0x02]).unwrap();
        fs::write(dir.join("b_latin1.txt"), [b'x', b' ', b' ', 0xE9, b'\n']).unwrap();
        fs::write(dir.join("c_ok.txt"), "text  \n").unwrap();

        let cleaner = WhitespaceCleaner::with_defaults();
        let (files, _) = cleaner.process(&dir).unwrap();

        assert_eq!(files, 1, "the walk should continue past unreadable files");
        assert_eq!(fs::read_to_string(dir.join("c_ok.txt")).unwrap(), "text\n");
        assert_eq!(
            fs::read(dir.join("a_binary.txt")).unwrap(),
            [0x00, 0x01, 0x02]
        );
    }
    use std::fs;

    #[test]
    fn test_remove_trailing_whitespace() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let test_dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.txt");
        fs::write(&test_file, "line1   \nline2\t\nline3\n").unwrap();

        let cleaner = WhitespaceCleaner::with_defaults();
        let (files, lines) = cleaner.process(&test_file).unwrap();

        assert_eq!(files, 1);
        assert_eq!(lines, 2); // line1 and line2 had trailing whitespace

        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, "line1\nline2\nline3\n");
    }

    /// Trailing whitespace must be stripped without rewriting the file's line
    /// terminators. Cleaning a CRLF file used to silently convert it to LF,
    /// turning a whitespace tidy-up into a whole-file diff.
    #[test]
    fn test_preserve_line_endings() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let test_dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&test_dir).unwrap();

        // (input, expected output)
        let cases = [
            ("lf", "line1  \nline2\n", "line1\nline2\n"),
            ("crlf", "line1  \r\nline2\r\n", "line1\r\nline2\r\n"),
            ("cr", "line1  \rline2\r", "line1\rline2\r"),
            (
                "mixed",
                "line1  \r\nline2  \nline3  \r",
                "line1\r\nline2\nline3\r",
            ),
            // Trailing whitespace on the final line, with no terminator at all.
            ("no_final_newline", "line1  \nline2  ", "line1\nline2"),
            // A file that ends with a blank line keeps that blank line.
            ("blank_last", "line1  \n\n", "line1\n\n"),
            // Whitespace-only lines collapse to empty, terminator preserved.
            ("ws_only", "a\r\n   \r\nb\r\n", "a\r\n\r\nb\r\n"),
        ];

        for (name, input, expected) in cases {
            let test_file = test_dir.join(format!("{}.txt", name));
            fs::write(&test_file, input).unwrap();

            let cleaner = WhitespaceCleaner::with_defaults();
            cleaner.process(&test_file).unwrap();

            let content = fs::read_to_string(&test_file).unwrap();
            assert_eq!(
                content, expected,
                "case '{}': line endings were not preserved",
                name
            );
        }
    }

    /// A file with no trailing whitespace must not be rewritten at all --
    /// in particular a CRLF file must not be "normalised" as a side effect.
    #[test]
    fn test_clean_file_untouched_when_nothing_to_strip() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let test_dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.txt");
        let original = "line1\r\nline2\r\n";
        fs::write(&test_file, original).unwrap();

        let cleaner = WhitespaceCleaner::with_defaults();
        let (files, lines) = cleaner.process(&test_file).unwrap();

        assert_eq!(files, 0);
        assert_eq!(lines, 0);
        assert_eq!(fs::read_to_string(&test_file).unwrap(), original);
    }

    #[test]
    fn test_dry_run_mode() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let test_dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.txt");
        let original = "line1   \nline2\n";
        fs::write(&test_file, original).unwrap();

        let opts = WhitespaceOptions {
            dry_run: true,

            ..Default::default()
        };

        let cleaner = WhitespaceCleaner::new(opts);
        cleaner.process(&test_file).unwrap();

        // File should be unchanged
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, original);
    }

    #[test]
    fn test_skip_hidden_files() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let test_dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&test_dir).unwrap();

        let hidden_file = test_dir.join(".hidden.txt");
        fs::write(&hidden_file, "line1   \n").unwrap();

        let cleaner = WhitespaceCleaner::with_defaults();
        let (files, _) = cleaner.process(&hidden_file).unwrap();

        // Hidden file should be skipped
        assert_eq!(files, 0);
    }

    #[test]
    fn test_file_extension_filtering() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let test_dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&test_dir).unwrap();

        let txt_file = test_dir.join("test.txt");
        let other_file = test_dir.join("test.xyz");

        fs::write(&txt_file, "line1   \n").unwrap();
        fs::write(&other_file, "line1   \n").unwrap();

        let opts = WhitespaceOptions {
            file_extensions: vec![".txt".to_string()],

            ..Default::default()
        };

        let cleaner = WhitespaceCleaner::new(opts);
        let (files, _) = cleaner.process(&test_dir).unwrap();

        // Only .txt should be processed
        assert_eq!(files, 1);

        let txt_content = fs::read_to_string(&txt_file).unwrap();
        let other_content = fs::read_to_string(&other_file).unwrap();

        assert_eq!(txt_content, "line1\n");
        assert_eq!(other_content, "line1   \n"); // Unchanged
    }

    #[test]
    fn test_recursive_processing() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let test_dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&test_dir).unwrap();

        let sub_dir = test_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();

        let file1 = test_dir.join("file1.txt");
        let file2 = sub_dir.join("file2.txt");

        fs::write(&file1, "line1   \n").unwrap();
        fs::write(&file2, "line2\t\n").unwrap();

        let cleaner = WhitespaceCleaner::with_defaults();
        let (files, lines) = cleaner.process(&test_dir).unwrap();

        assert_eq!(files, 2);
        assert_eq!(lines, 2);
    }
}
