//! File header management transformer

use regex::Regex;
use std::fs;
use std::path::Path;

/// Options for header management
#[derive(Debug, Clone)]
pub struct HeaderOptions {
    /// The header text to insert (without comment markers -- those are part of the text)
    pub text: String,
    /// If true, replace {year} in the header text with the current year
    pub update_year: bool,
    /// File extensions to process
    pub file_extensions: Vec<String>,
    /// Process directories recursively
    pub recursive: bool,
    /// Dry run mode (don't modify files)
    pub dry_run: bool,
}

impl Default for HeaderOptions {
    fn default() -> Self {
        HeaderOptions {
            text: String::new(),
            update_year: false,
            file_extensions: vec![
                ".py", ".pyx", ".pxd", ".pxi", ".c", ".h", ".cpp", ".hpp", ".rs", ".go", ".java",
                ".js", ".ts", ".jsx", ".tsx",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            recursive: true,
            dry_run: false,
        }
    }
}

/// File header manager: insert or update headers at the top of source files
pub struct HeaderManager {
    options: HeaderOptions,
    /// The resolved header text (with year substitution applied)
    resolved_header: String,
    /// Regex to detect if the header (or a year-variant of it) already exists
    header_detector: Option<Regex>,
}

impl HeaderManager {
    /// Creates a new header manager with the given options
    pub fn new(options: HeaderOptions) -> crate::Result<Self> {
        let resolved_header = if options.update_year {
            let year = chrono::Utc::now().format("%Y").to_string();
            options.text.replace("{year}", &year)
        } else {
            options.text.clone()
        };

        // Build a detector regex: escape the header text but replace any 4-digit year
        // with \d{4} so we can find year-variant headers
        let header_detector =
            if !resolved_header.is_empty() {
                let escaped = regex::escape(&resolved_header);
                // Replace any 4-digit year (19xx or 20xx) with a flexible year
                // pattern, so a header written in a previous year is still
                // recognised. Note `\d{2}` here is a quantifier: escaping the
                // braces (`\d\{2\}`) made this match a literal "{2}", so the
                // substitution never fired and year updates silently inserted
                // a second header instead of replacing the first.
                let flexible = Regex::new(r"(?:19|20)\d{2}")
                    .unwrap()
                    .replace_all(&escaped, r"\d{4}")
                    .to_string();
                Some(Regex::new(&flexible).map_err(|e| {
                    anyhow::anyhow!("failed to compile header detection regex: {}", e)
                })?)
            } else {
                None
            };

        Ok(HeaderManager {
            options,
            resolved_header,
            header_detector,
        })
    }

    /// Byte offset at which a header may legitimately begin: the start of the
    /// file, skipping a shebang line and any leading blank lines.
    fn header_zone(content: &str) -> usize {
        let mut pos = 0;
        if content.starts_with("#!") {
            pos = content.find('\n').map(|i| i + 1).unwrap_or(content.len());
        }
        let rest = &content[pos..];
        let trimmed = rest.trim_start_matches(['\n', '\r', ' ', '\t']);
        pos + (rest.len() - trimmed.len())
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

        if let Some(ext) = path.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            self.options.file_extensions.contains(&ext_str)
        } else {
            false
        }
    }

    /// Process a single file. Returns true if the file was modified (or would be in dry-run).
    pub fn process_file(&self, path: &Path) -> crate::Result<bool> {
        if !self.should_process(path) {
            return Ok(false);
        }

        if self.resolved_header.is_empty() {
            return Ok(false);
        }

        let content = match crate::text::read_text(path)? {
            Some(c) => c,
            None => return Ok(false),
        };

        // Check if header already exists (possibly with a different year).
        // Detection is anchored to the header zone -- the top of the file,
        // after any shebang and leading blank lines -- so that a year-variant
        // string elsewhere in the body (a test fixture, a vendored blob) is
        // not mistaken for this file's own header and rewritten.
        let zone = Self::header_zone(&content);
        if let Some(ref detector) = self.header_detector {
            if let Some(m) = detector
                .find_at(&content, zone)
                .filter(|m| m.start() == zone)
            {
                // Header exists -- check if it needs a year update
                let existing = &content[m.start()..m.end()];
                if existing == self.resolved_header {
                    // Exact match, nothing to do
                    return Ok(false);
                }

                // Replace old header with new one (year update)
                let new_content = format!("{}{}", self.resolved_header, &content[m.end()..]);
                // Preserve content before the header (e.g., shebang lines)
                let prefix = &content[..m.start()];
                let full = format!("{}{}", prefix, new_content);

                if self.options.dry_run {
                    log::info!("Would update header in '{}'", path.display());
                } else {
                    fs::write(path, &full)?;
                    log::info!("Updated header in '{}'", path.display());
                }
                return Ok(true);
            }
        }

        // Header doesn't exist -- insert it
        // Preserve shebang lines (e.g., #!/usr/bin/env python)
        let (prefix, rest) = if content.starts_with("#!") {
            if let Some(pos) = content.find('\n') {
                (&content[..=pos], &content[pos + 1..])
            } else {
                (content.as_str(), "")
            }
        } else {
            ("", content.as_str())
        };

        let new_content = if prefix.is_empty() {
            format!("{}\n\n{}", self.resolved_header, rest)
        } else {
            format!("{}{}\n\n{}", prefix, self.resolved_header, rest)
        };

        if self.options.dry_run {
            log::info!("Would insert header in '{}'", path.display());
        } else {
            fs::write(path, &new_content)?;
            log::info!("Inserted header in '{}'", path.display());
        }

        Ok(true)
    }

    /// Processes a directory or file. Returns (files_changed, operation_description).
    pub fn process(&self, path: &Path) -> crate::Result<(usize, usize)> {
        let mut total_files = 0;
        // We use the second value as "operations" (1 per file touched)
        let mut total_ops = 0;

        if path.is_file() {
            if self.process_file(path)? {
                total_files = 1;
                total_ops = 1;
            }
        } else if path.is_dir() {
            for entry in crate::walk::walk_files(path, self.options.recursive) {
                if self.process_file(entry.path())? {
                    total_files += 1;
                    total_ops += 1;
                }
            }
        }

        Ok((total_files, total_ops))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_insert_header() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        fs::write(&file, "fn main() {}\n").unwrap();

        let options = HeaderOptions {
            text: "// Copyright 2025 TestCorp".to_string(),
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        let (files, _) = manager.process(&file).unwrap();

        assert_eq!(files, 1);

        let content = fs::read_to_string(&file).unwrap();
        assert!(content.starts_with("// Copyright 2025 TestCorp\n\n"));
        assert!(content.contains("fn main() {}"));
    }

    #[test]
    fn test_header_already_present() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        let original = "// Copyright 2025 TestCorp\n\nfn main() {}\n";
        fs::write(&file, original).unwrap();

        let options = HeaderOptions {
            text: "// Copyright 2025 TestCorp".to_string(),
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        let (files, _) = manager.process(&file).unwrap();

        assert_eq!(files, 0);

        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, original);
    }

    /// An existing header carrying a different year must be *replaced*, not
    /// shadowed by a second header inserted above it. Asserting only
    /// `starts_with` is not enough: that passes when the old header is
    /// duplicated below the new one, which is exactly what used to happen.
    #[test]
    fn test_update_year_in_header() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        fs::write(&file, "// Copyright 2020 TestCorp\n\nfn main() {}\n").unwrap();

        let current_year = chrono::Utc::now().format("%Y").to_string();
        let header = format!("// Copyright {} TestCorp", current_year);
        let options = HeaderOptions {
            text: header.clone(),
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        let (files, _) = manager.process(&file).unwrap();

        assert_eq!(files, 1);

        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(
            content,
            format!("{}\n\nfn main() {{}}\n", header),
            "the old header should have been replaced in place"
        );
        assert!(
            !content.contains("2020"),
            "the superseded year is still present -- the header was duplicated"
        );
        assert_eq!(
            content.matches("TestCorp").count(),
            1,
            "the file gained a second header instead of having one updated"
        );
    }

    /// The `{year}` template plus `--update-year` must be idempotent across
    /// years: running it in successive years leaves exactly one header.
    #[test]
    fn test_update_year_is_idempotent_across_years() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        // Simulates a header written in a previous year.
        fs::write(&file, "// Copyright 1999 TestCorp\n\nfn main() {}\n").unwrap();

        let options = HeaderOptions {
            text: "// Copyright {year} TestCorp".to_string(),
            update_year: true,
            ..Default::default()
        };

        // First run updates 1999 -> current year.
        let manager = HeaderManager::new(options.clone()).unwrap();
        assert!(manager.process_file(&file).unwrap());

        // Second run is a no-op: the header already carries the current year.
        let manager = HeaderManager::new(options).unwrap();
        assert!(
            !manager.process_file(&file).unwrap(),
            "a second run in the same year should change nothing"
        );

        let content = fs::read_to_string(&file).unwrap();
        let year = chrono::Utc::now().format("%Y").to_string();
        assert_eq!(
            content,
            format!("// Copyright {} TestCorp\n\nfn main() {{}}\n", year)
        );
        assert_eq!(content.matches("Copyright").count(), 1);
    }

    /// A year-variant header buried in the body of a file (a test fixture, a
    /// vendored blob) must not be mistaken for the file's own header.
    #[test]
    fn test_header_detection_is_anchored_to_top_of_file() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        let body = "fn main() {}\n\nconst FIXTURE: &str = \"// Copyright 2020 TestCorp\";\n";
        fs::write(&file, body).unwrap();

        let header = "// Copyright 2026 TestCorp".to_string();
        let options = HeaderOptions {
            text: header.clone(),
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        manager.process_file(&file).unwrap();

        let content = fs::read_to_string(&file).unwrap();
        assert!(
            content.starts_with(&header),
            "the header should have been inserted at the top"
        );
        assert!(
            content.contains("// Copyright 2020 TestCorp\";"),
            "the mid-file fixture string was rewritten"
        );
    }

    #[test]
    fn test_preserve_shebang() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.py");
        fs::write(&file, "#!/usr/bin/env python\nprint('hello')\n").unwrap();

        let options = HeaderOptions {
            text: "# Copyright 2025 TestCorp".to_string(),
            file_extensions: vec![".py".to_string()],
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        manager.process(&file).unwrap();

        let content = fs::read_to_string(&file).unwrap();
        assert!(content.starts_with("#!/usr/bin/env python\n"));
        assert!(content.contains("# Copyright 2025 TestCorp"));
        assert!(content.contains("print('hello')"));
    }

    #[test]
    fn test_dry_run() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        let original = "fn main() {}\n";
        fs::write(&file, original).unwrap();

        let options = HeaderOptions {
            text: "// License Header".to_string(),
            dry_run: true,
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        let (files, _) = manager.process(&file).unwrap();

        assert_eq!(files, 1);
        let content = fs::read_to_string(&file).unwrap();
        assert_eq!(content, original);
    }

    #[test]
    fn test_empty_header() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        fs::write(&file, "fn main() {}\n").unwrap();

        let options = HeaderOptions {
            text: String::new(),
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        let (files, _) = manager.process(&file).unwrap();

        assert_eq!(files, 0);
    }

    #[test]
    fn test_year_template_substitution() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        fs::write(&file, "fn main() {}\n").unwrap();

        let options = HeaderOptions {
            text: "// Copyright {year} TestCorp".to_string(),
            update_year: true,
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        manager.process(&file).unwrap();

        let current_year = chrono::Utc::now().format("%Y").to_string();
        let content = fs::read_to_string(&file).unwrap();
        assert!(content.contains(&format!("Copyright {} TestCorp", current_year)));
    }

    #[test]
    fn test_recursive_processing() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let sub = dir.join("sub");
        fs::create_dir_all(&sub).unwrap();

        let f1 = dir.join("a.rs");
        let f2 = sub.join("b.rs");
        fs::write(&f1, "fn a() {}\n").unwrap();
        fs::write(&f2, "fn b() {}\n").unwrap();

        let options = HeaderOptions {
            text: "// Header".to_string(),
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        let (files, _) = manager.process(&dir).unwrap();

        assert_eq!(files, 2);
    }

    #[test]
    fn test_multiline_header() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let dir = _tmp.path().to_path_buf();
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("test.rs");
        fs::write(&file, "fn main() {}\n").unwrap();

        let options = HeaderOptions {
            text: "// Copyright 2025 TestCorp\n// Licensed under MIT\n// All rights reserved"
                .to_string(),
            ..Default::default()
        };
        let manager = HeaderManager::new(options).unwrap();
        manager.process(&file).unwrap();

        let content = fs::read_to_string(&file).unwrap();
        assert!(content.starts_with(
            "// Copyright 2025 TestCorp\n// Licensed under MIT\n// All rights reserved\n\n"
        ));
    }
}
