//! File renaming transformer

use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Case transformation options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CaseTransform {
    /// Convert to lowercase
    Lowercase,
    /// Convert to UPPERCASE
    Uppercase,
    /// Capitalize first letter only
    Capitalize,
    /// No case transformation
    None,
}

/// Space replacement options
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpaceReplace {
    /// Replace spaces with underscores
    Underscore,
    /// Replace spaces with hyphens
    Hyphen,
    /// No space replacement
    None,
}

/// Options for file renaming
#[derive(Debug, Clone)]
pub struct RenameOptions {
    /// Case transformation to apply
    pub case_transform: CaseTransform,
    /// Space replacement to apply
    pub space_replace: SpaceReplace,
    /// Prefix to add
    pub add_prefix: Option<String>,
    /// Prefix to remove
    pub remove_prefix: Option<String>,
    /// Suffix to add (before extension)
    pub add_suffix: Option<String>,
    /// Suffix to remove (before extension)
    pub remove_suffix: Option<String>,
    /// Process directories recursively
    pub recursive: bool,
    /// Dry run mode (don't rename files)
    pub dry_run: bool,
}

impl Default for RenameOptions {
    fn default() -> Self {
        RenameOptions {
            case_transform: CaseTransform::None,
            space_replace: SpaceReplace::None,
            add_prefix: None,
            remove_prefix: None,
            add_suffix: None,
            remove_suffix: None,
            recursive: true,
            dry_run: false,
        }
    }
}

/// File renamer for transforming file names
pub struct FileRenamer {
    options: RenameOptions,
}

impl FileRenamer {
    /// Creates a new file renamer with the given options
    pub fn new(options: RenameOptions) -> Self {
        FileRenamer { options }
    }

    /// Creates a renamer with default options
    pub fn with_defaults() -> Self {
        FileRenamer {
            options: RenameOptions::default(),
        }
    }

    /// Checks if a path should be processed
    fn should_process(&self, path: &Path) -> bool {
        // Only process files, not directories
        if !path.is_file() {
            return false;
        }

        // Skip hidden files
        if let Some(name) = path.file_name() {
            if name.to_str().map(|s| s.starts_with('.')).unwrap_or(false) {
                return false;
            }
        }

        true
    }

    /// Applies all transformations to a filename
    fn transform_name(&self, name: &str, extension: Option<&str>) -> String {
        let mut result = name.to_string();

        // 1. Remove prefix
        if let Some(prefix) = &self.options.remove_prefix {
            if result.starts_with(prefix) {
                result = result[prefix.len()..].to_string();
            }
        }

        // 2. Remove suffix (before extension)
        if let Some(suffix) = &self.options.remove_suffix {
            if result.ends_with(suffix) {
                result = result[..result.len() - suffix.len()].to_string();
            }
        }

        // 3. Separator replacement (replace spaces, hyphens, underscores with desired separator)
        match self.options.space_replace {
            SpaceReplace::Underscore => {
                // Replace all separators (spaces, hyphens) with underscores
                result = result.replace(' ', "_").replace('-', "_");
            }
            SpaceReplace::Hyphen => {
                // Replace all separators (spaces, underscores) with hyphens
                result = result.replace(' ', "-").replace('_', "-");
            }
            SpaceReplace::None => {}
        }

        // 4. Case transformation
        match self.options.case_transform {
            CaseTransform::Lowercase => {
                result = result.to_lowercase();
            }
            CaseTransform::Uppercase => {
                result = result.to_uppercase();
            }
            CaseTransform::Capitalize => {
                if !result.is_empty() {
                    let mut chars = result.chars();
                    if let Some(first) = chars.next() {
                        result = first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase();
                    }
                }
            }
            CaseTransform::None => {}
        }

        // 5. Add prefix
        if let Some(prefix) = &self.options.add_prefix {
            result = format!("{}{}", prefix, result);
        }

        // 6. Add suffix (before extension)
        if let Some(suffix) = &self.options.add_suffix {
            result = format!("{}{}", result, suffix);
        }

        // 7. Add extension back
        if let Some(ext) = extension {
            result = format!("{}.{}", result, ext);
        }

        result
    }

    /// Renames a single file
    pub fn rename_file(&self, path: &Path) -> crate::Result<bool> {
        if !self.should_process(path) {
            return Ok(false);
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?;

        // Split filename and extension
        let (name, extension) = if let Some(pos) = file_name.rfind('.') {
            let name = &file_name[..pos];
            let ext = &file_name[pos + 1..];
            (name, Some(ext))
        } else {
            (file_name, None)
        };

        let new_name = self.transform_name(name, extension);

        // If name didn't change, nothing to do
        if new_name == file_name {
            return Ok(false);
        }

        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("No parent directory"))?;
        let new_path = parent.join(&new_name);

        // Check if target already exists (but allow case-only renames on case-insensitive filesystems)
        if new_path.exists() {
            // Check if this is the same file (case-insensitive filesystems)
            // Use canonicalize to resolve to the actual path
            let same_file = match (path.canonicalize(), new_path.canonicalize()) {
                (Ok(p1), Ok(p2)) => p1 == p2,
                _ => false,
            };

            if !same_file {
                return Err(anyhow::anyhow!(
                    "Target file already exists: '{}'",
                    new_path.display()
                ));
            }
        }

        if self.options.dry_run {
            println!(
                "Would rename '{}' -> '{}'",
                path.display(),
                new_path.display()
            );
        } else {
            fs::rename(path, &new_path)?;
            println!("Renamed '{}' -> '{}'", path.display(), new_path.display());
        }

        Ok(true)
    }

    /// Processes a directory or file
    pub fn process(&self, path: &Path) -> crate::Result<usize> {
        let mut renamed_count = 0;

        if path.is_file() {
            if self.rename_file(path)? {
                renamed_count = 1;
            }
        } else if path.is_dir() {
            if self.options.recursive {
                // Collect all files first to avoid issues with renaming while iterating
                let mut files: Vec<PathBuf> = WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.file_type().is_file())
                    .map(|e| e.path().to_path_buf())
                    .collect();

                // Sort by depth (deepest first) to avoid parent directory rename issues
                files.sort_by(|a, b| b.components().count().cmp(&a.components().count()));

                for file_path in files {
                    if self.rename_file(&file_path)? {
                        renamed_count += 1;
                    }
                }
            } else {
                let mut files: Vec<PathBuf> = fs::read_dir(path)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_file())
                    .collect();

                // Sort for consistent processing
                files.sort();

                for file_path in files {
                    if self.rename_file(&file_path)? {
                        renamed_count += 1;
                    }
                }
            }
        }

        Ok(renamed_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_lowercase_transform() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_lowercase");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("TestFile.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.case_transform = CaseTransform::Lowercase;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        let new_file = test_dir.join("testfile.txt");
        assert!(new_file.exists());
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "content");

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_uppercase_transform() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_uppercase");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("testfile.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.case_transform = CaseTransform::Uppercase;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        let new_file = test_dir.join("TESTFILE.txt");
        assert!(new_file.exists());
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "content");

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_capitalize_transform() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_capitalize");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("testFile.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.case_transform = CaseTransform::Capitalize;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        let new_file = test_dir.join("Testfile.txt");
        assert!(new_file.exists());
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "content");

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_separators_to_underscore() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_underscore");
        fs::create_dir_all(&test_dir).unwrap();

        // Test space to underscore
        let test_file1 = test_dir.join("test file.txt");
        fs::write(&test_file1, "content").unwrap();

        // Test hyphen to underscore
        let test_file2 = test_dir.join("test-file2.txt");
        fs::write(&test_file2, "content").unwrap();

        // Test mixed separators to underscore
        let test_file3 = test_dir.join("test-file 3.txt");
        fs::write(&test_file3, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.space_replace = SpaceReplace::Underscore;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_dir).unwrap();

        assert_eq!(count, 3);
        assert!(test_dir.join("test_file.txt").exists());
        assert!(test_dir.join("test_file2.txt").exists());
        assert!(test_dir.join("test_file_3.txt").exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_separators_to_hyphen() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_hyphen");
        fs::create_dir_all(&test_dir).unwrap();

        // Test space to hyphen
        let test_file1 = test_dir.join("test file.txt");
        fs::write(&test_file1, "content").unwrap();

        // Test underscore to hyphen
        let test_file2 = test_dir.join("test_file2.txt");
        fs::write(&test_file2, "content").unwrap();

        // Test mixed separators to hyphen
        let test_file3 = test_dir.join("test_file 3.txt");
        fs::write(&test_file3, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.space_replace = SpaceReplace::Hyphen;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_dir).unwrap();

        assert_eq!(count, 3);
        assert!(test_dir.join("test-file.txt").exists());
        assert!(test_dir.join("test-file2.txt").exists());
        assert!(test_dir.join("test-file-3.txt").exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_add_prefix() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_add_prefix");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("file.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.add_prefix = Some("new_".to_string());

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        assert!(test_dir.join("new_file.txt").exists());
        assert!(!test_file.exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_remove_prefix() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_rm_prefix");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("old_file.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.remove_prefix = Some("old_".to_string());

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        assert!(test_dir.join("file.txt").exists());
        assert!(!test_file.exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_add_suffix() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_add_suffix");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("file.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.add_suffix = Some("_backup".to_string());

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        assert!(test_dir.join("file_backup.txt").exists());
        assert!(!test_file.exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_remove_suffix() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_rm_suffix");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("file_old.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.remove_suffix = Some("_old".to_string());

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        assert!(test_dir.join("file.txt").exists());
        assert!(!test_file.exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_combined_transforms() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_combined");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("old_Test File.txt");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.remove_prefix = Some("old_".to_string());
        opts.space_replace = SpaceReplace::Underscore;
        opts.case_transform = CaseTransform::Lowercase;
        opts.add_suffix = Some("_new".to_string());

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        assert!(test_dir.join("test_file_new.txt").exists());
        assert!(!test_file.exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_dry_run_mode() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_dry");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("TestFile.txt");
        let original_content = "content";
        fs::write(&test_file, original_content).unwrap();

        let mut opts = RenameOptions::default();
        opts.case_transform = CaseTransform::Lowercase;
        opts.dry_run = true;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        // File should still exist and be unchanged in dry run
        assert!(test_file.exists());
        assert_eq!(fs::read_to_string(&test_file).unwrap(), original_content);

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_skip_hidden_files() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_hidden");
        fs::create_dir_all(&test_dir).unwrap();

        let hidden_file = test_dir.join(".hidden.txt");
        fs::write(&hidden_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.case_transform = CaseTransform::Uppercase;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&hidden_file).unwrap();

        // Hidden file should be skipped
        assert_eq!(count, 0);
        assert!(hidden_file.exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_recursive_processing() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_recursive");
        fs::create_dir_all(&test_dir).unwrap();

        let sub_dir = test_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();

        let file1 = test_dir.join("File1.txt");
        let file2 = sub_dir.join("File2.txt");

        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let mut opts = RenameOptions::default();
        opts.case_transform = CaseTransform::Lowercase;
        opts.recursive = true;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_dir).unwrap();

        assert_eq!(count, 2);
        assert!(test_dir.join("file1.txt").exists());
        assert!(sub_dir.join("file2.txt").exists());

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_no_extension_file() {
        let test_dir = std::env::temp_dir().join("codeconvert_rename_no_ext");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("TestFile");
        fs::write(&test_file, "content").unwrap();

        let mut opts = RenameOptions::default();
        opts.case_transform = CaseTransform::Lowercase;

        let renamer = FileRenamer::new(opts);
        let count = renamer.process(&test_file).unwrap();

        assert_eq!(count, 1);
        let new_file = test_dir.join("testfile");
        assert!(new_file.exists());
        assert_eq!(fs::read_to_string(&new_file).unwrap(), "content");

        fs::remove_dir_all(&test_dir).unwrap();
    }
}
