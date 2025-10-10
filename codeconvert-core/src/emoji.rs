//! Emoji removal and replacement transformer
//!
//! This module provides functionality to remove or replace emojis in text files,
//! with special handling for task completion emojis.

use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Options for emoji transformation
#[derive(Debug, Clone)]
pub struct EmojiOptions {
    /// Replace task completion emojis with text alternatives
    pub replace_task_emojis: bool,
    /// Remove all other emojis
    pub remove_other_emojis: bool,
    /// File extensions to process
    pub file_extensions: Vec<String>,
    /// Process directories recursively
    pub recursive: bool,
    /// Dry run mode (don't modify files)
    pub dry_run: bool,
}

impl Default for EmojiOptions {
    fn default() -> Self {
        EmojiOptions {
            replace_task_emojis: true,
            remove_other_emojis: true,
            file_extensions: vec![
                ".md", ".txt", ".rst", ".org",
                ".py", ".rs", ".go", ".java",
                ".js", ".ts", ".jsx", ".tsx",
                ".c", ".h", ".cpp", ".hpp",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            recursive: true,
            dry_run: false,
        }
    }
}

/// Emoji transformer for removing and replacing emojis
pub struct EmojiTransformer {
    options: EmojiOptions,
    task_emoji_pattern: Regex,
    general_emoji_pattern: Regex,
}

impl EmojiTransformer {
    /// Creates a new emoji transformer with the given options
    pub fn new(options: EmojiOptions) -> Self {
        // Task completion emojis that should be replaced with text
        let task_emoji_pattern = Regex::new(
            r"(?x)
            [\u2705]|          # White check mark (✅)
            [\u2611]|          # Ballot box with check (☑)
            [\u2714]|          # Heavy check mark (✔)
            [\u2713]|          # Check mark (✓)
            [\u2610]|          # Ballot box (☐)
            [\u2612]|          # Ballot box with X (☒)
            [\u274C]|          # Cross mark (❌)
            [\u274E]|          # Negative squared cross mark (❎)
            [\u26A0]|          # Warning sign (⚠)
            [\u26D4]|          # No entry (⛔)
            [\u{1F4DD}]|       # Memo (📝)
            [\u{1F4CB}]|       # Clipboard (📋)
            [\u{1F4C4}]|       # Page facing up (📄)
            [\u{1F4C5}]|       # Calendar (📅)
            [\u{1F4C6}]|       # Tear-off calendar (📆)
            [\u{1F5D3}]|       # Spiral calendar (🗓)
            [\u{1F4D1}]|       # Bookmark tabs (📑)
            [\u{1F4CC}]|       # Pushpin (📌)
            [\u{1F4CD}]|       # Round pushpin (📍)
            [\u{1F4CE}]        # Paperclip (📎)
            "
        ).unwrap();

        // General emoji pattern (all emojis not covered by task emojis)
        let general_emoji_pattern = Regex::new(
            r"(?x)
            [\u{1F600}-\u{1F64F}]|  # Emoticons
            [\u{1F300}-\u{1F5FF}]|  # Symbols & pictographs
            [\u{1F680}-\u{1F6FF}]|  # Transport & map symbols
            [\u{1F1E0}-\u{1F1FF}]|  # Flags
            [\u{2600}-\u{26FF}]|    # Miscellaneous symbols
            [\u{2700}-\u{27BF}]|    # Dingbats
            [\u{1F900}-\u{1F9FF}]|  # Supplemental symbols
            [\u{1FA00}-\u{1FA6F}]|  # Extended-A
            [\u{1FA70}-\u{1FAFF}]|  # Extended-B
            [\u{FE00}-\u{FE0F}]|    # Variation selectors
            [\u{1F004}]|            # Mahjong tile
            [\u{1F0CF}]|            # Playing card
            [\u{1F18E}]|            # Negative squared AB
            [\u{1F191}-\u{1F19A}]|  # Squared CL, COOL, etc.
            [\u{1F1E6}-\u{1F1FF}]   # Regional indicator symbols
            "
        ).unwrap();

        EmojiTransformer {
            options,
            task_emoji_pattern,
            general_emoji_pattern,
        }
    }

    /// Creates a transformer with default options
    pub fn with_defaults() -> Self {
        EmojiTransformer::new(EmojiOptions::default())
    }

    /// Checks if a file should be processed
    fn should_process(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        // Skip hidden files and directories
        if path.components().any(|c| {
            c.as_os_str()
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
        }) {
            return false;
        }

        // Skip build directories
        let skip_dirs = ["build", "__pycache__", ".git", "node_modules", "venv", ".venv", "target"];
        if path.components().any(|c| {
            c.as_os_str()
                .to_str()
                .map(|s| skip_dirs.contains(&s))
                .unwrap_or(false)
        }) {
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

    /// Replace task emojis with text equivalents
    fn replace_task_emoji(&self, emoji: &str) -> &str {
        match emoji {
            "\u{2705}" => "[x]",      // ✅ -> [x]
            "\u{2611}" => "[x]",      // ☑ -> [x]
            "\u{2714}" => "[x]",      // ✔ -> [x]
            "\u{2713}" => "[x]",      // ✓ -> [x]
            "\u{2610}" => "[ ]",      // ☐ -> [ ]
            "\u{2612}" => "[X]",      // ☒ -> [X]
            "\u{274C}" => "[X]",      // ❌ -> [X]
            "\u{274E}" => "[X]",      // ❎ -> [X]
            "\u{26A0}" => "[!]",      // ⚠ -> [!]
            "\u{26D4}" => "[!]",      // ⛔ -> [!]
            "\u{1F4DD}" => "[note]",  // 📝 -> [note]
            "\u{1F4CB}" => "[list]",  // 📋 -> [list]
            "\u{1F4C4}" => "[doc]",   // 📄 -> [doc]
            "\u{1F4C5}" => "[cal]",   // 📅 -> [cal]
            "\u{1F4C6}" => "[cal]",   // 📆 -> [cal]
            "\u{1F5D3}" => "[cal]",   // 🗓 -> [cal]
            "\u{1F4D1}" => "[tab]",   // 📑 -> [tab]
            "\u{1F4CC}" => "[pin]",   // 📌 -> [pin]
            "\u{1F4CD}" => "[pin]",   // 📍 -> [pin]
            "\u{1F4CE}" => "[clip]",  // 📎 -> [clip]
            _ => "",
        }
    }

    /// Transform emojis in a single file
    pub fn transform_file(&self, path: &Path) -> crate::Result<usize> {
        if !self.should_process(path) {
            return Ok(0);
        }

        let content = fs::read_to_string(path)?;
        let original_content = content.clone();

        let mut modified_content = content;
        let mut changes = 0;

        // Replace task emojis with text alternatives
        if self.options.replace_task_emojis {
            let before = modified_content.clone();
            let replaced = self.task_emoji_pattern.replace_all(&modified_content, |caps: &regex::Captures| {
                self.replace_task_emoji(&caps[0])
            });

            if replaced != before {
                // Count the number of replacements made
                let task_emojis_found = self.task_emoji_pattern.find_iter(&before).count();
                changes += task_emojis_found;
                modified_content = replaced.to_string();
            }
        }

        // Remove other emojis
        if self.options.remove_other_emojis {
            let before = modified_content.clone();
            let cleaned = self.general_emoji_pattern.replace_all(&modified_content, "");
            if cleaned != before {
                // Count the number of emojis removed
                let emojis_found = self.general_emoji_pattern.find_iter(&before).count();
                changes += emojis_found;
                modified_content = cleaned.to_string();
            }
        }

        if modified_content != original_content {
            if self.options.dry_run {
                println!(
                    "Would transform emojis in '{}'",
                    path.display()
                );
            } else {
                fs::write(path, modified_content)?;
                println!("Transformed emojis in '{}'", path.display());
            }
            Ok(changes.max(1))
        } else {
            Ok(0)
        }
    }

    /// Processes a directory or file
    pub fn process(&self, path: &Path) -> crate::Result<(usize, usize)> {
        let mut total_files = 0;
        let mut total_changes = 0;

        if path.is_file() {
            let changes = self.transform_file(path)?;
            if changes > 0 {
                total_files = 1;
                total_changes = changes;
            }
        } else if path.is_dir() {
            if self.options.recursive {
                for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        let changes = self.transform_file(entry.path())?;
                        if changes > 0 {
                            total_files += 1;
                            total_changes += changes;
                        }
                    }
                }
            } else {
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let entry_path = entry.path();
                    if entry_path.is_file() {
                        let changes = self.transform_file(&entry_path)?;
                        if changes > 0 {
                            total_files += 1;
                            total_changes += changes;
                        }
                    }
                }
            }
        }

        Ok((total_files, total_changes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_replace_task_emojis() {
        let test_dir = std::env::temp_dir().join("codeconvert_emoji_test");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.md");
        fs::write(&test_file, "- [x] Done task\n- [ ] Todo task\n- Task complete\n").unwrap();

        // Replace checkmarks with [x]
        let content = fs::read_to_string(&test_file).unwrap();
        let updated = content.replace("✅", "[x]");
        fs::write(&test_file, updated).unwrap();

        let transformer = EmojiTransformer::with_defaults();
        let (files, _) = transformer.process(&test_file).unwrap();

        // Should still be valid markdown
        let content = fs::read_to_string(&test_file).unwrap();
        assert!(content.contains("[x]") || content.contains("[ ]"));

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_checkmark_replacement() {
        let test_dir = std::env::temp_dir().join("codeconvert_emoji_checkmark");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.txt");
        fs::write(&test_file, "Task done ✅\nTask pending ☐\n").unwrap();

        let transformer = EmojiTransformer::with_defaults();
        let (files, _) = transformer.process(&test_file).unwrap();

        if files > 0 {
            let content = fs::read_to_string(&test_file).unwrap();
            assert!(content.contains("[x]") || content.contains("[ ]"));
            assert!(!content.contains("✅"));
            assert!(!content.contains("☐"));
        }

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_dry_run_mode() {
        let test_dir = std::env::temp_dir().join("codeconvert_emoji_dry");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.txt");
        let original = "Task ✅ done";
        fs::write(&test_file, original).unwrap();

        let mut opts = EmojiOptions::default();
        opts.dry_run = true;

        let transformer = EmojiTransformer::new(opts);
        transformer.process(&test_file).unwrap();

        // File should be unchanged
        let content = fs::read_to_string(&test_file).unwrap();
        assert_eq!(content, original);

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_skip_hidden_files() {
        let test_dir = std::env::temp_dir().join("codeconvert_emoji_hidden");
        fs::create_dir_all(&test_dir).unwrap();

        let hidden_file = test_dir.join(".hidden.txt");
        fs::write(&hidden_file, "Task ✅\n").unwrap();

        let transformer = EmojiTransformer::with_defaults();
        let (files, _) = transformer.process(&hidden_file).unwrap();

        // Hidden file should be skipped
        assert_eq!(files, 0);

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_extension_filtering() {
        let test_dir = std::env::temp_dir().join("codeconvert_emoji_ext");
        fs::create_dir_all(&test_dir).unwrap();

        let md_file = test_dir.join("test.md");
        let xyz_file = test_dir.join("test.xyz");

        fs::write(&md_file, "✅ Task\n").unwrap();
        fs::write(&xyz_file, "✅ Task\n").unwrap();

        let mut opts = EmojiOptions::default();
        opts.file_extensions = vec![".md".to_string()];

        let transformer = EmojiTransformer::new(opts);
        let (files, _) = transformer.process(&test_dir).unwrap();

        // Only .md should be processed
        assert_eq!(files, 1);

        let md_content = fs::read_to_string(&md_file).unwrap();
        let xyz_content = fs::read_to_string(&xyz_file).unwrap();

        assert!(md_content.contains("[x]") || !md_content.contains("✅"));
        assert_eq!(xyz_content, "✅ Task\n"); // Unchanged

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_recursive_processing() {
        let test_dir = std::env::temp_dir().join("codeconvert_emoji_recursive");
        fs::create_dir_all(&test_dir).unwrap();

        let sub_dir = test_dir.join("subdir");
        fs::create_dir_all(&sub_dir).unwrap();

        let file1 = test_dir.join("file1.md");
        let file2 = sub_dir.join("file2.md");

        fs::write(&file1, "✅ Done\n").unwrap();
        fs::write(&file2, "☐ Todo\n").unwrap();

        let transformer = EmojiTransformer::with_defaults();
        let (files, _) = transformer.process(&test_dir).unwrap();

        assert_eq!(files, 2);

        fs::remove_dir_all(&test_dir).unwrap();
    }
}
