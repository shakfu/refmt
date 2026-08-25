mod config;

use clap::{Parser, Subcommand};
use log::{debug, info, warn};
use logging_timer::time;
use reformat_core::config::{
    CleanConfig, ConvertConfig, EmojiConfig, EndingsConfig, GroupConfig, HeaderConfig,
    IndentConfig, RenameConfig, ReplaceConfig, ReplacePatternEntry,
};
use reformat_core::{
    CombinedOptions, CombinedProcessor, ContentReplacer, EmojiTransformer, EndingsNormalizer,
    FileGrouper, FileRenamer, HeaderManager, IndentNormalizer, ReferenceFixer, ReferenceScanner,
    ScanOptions, WhitespaceCleaner,
};
use simplelog::*;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(
    name = "reformat",
    version = env!("CARGO_PKG_VERSION"),
    about = "Code transformation tool for case conversion and cleaning",
    long_about = "A modular code transformation framework.\n\n\
                  Usage:\n\
                  - reformat <path>: Run all transformations (rename to lowercase, emojis, clean)\n\
                  - reformat -p <preset> <path>: Run a named preset from reformat.json\n\
                  - reformat --job <file|-> <path>: Run an ad-hoc job from a file or stdin\n\n\
                  Commands:\n\
                  - convert: Convert between case formats\n\
                  - clean: Remove trailing whitespace\n\
                  - emojis: Remove or replace emojis with text alternatives\n\
                  - rename_files: Rename files with various transformations\n\
                  - group: Group files by common prefix into subdirectories\n\
                  - endings: Normalize line endings (LF/CRLF/CR)\n\
                  - indent: Normalize indentation (tabs/spaces)\n\
                  - replace: Regex find-and-replace across files\n\
                  - header: Insert or update file headers"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The directory or file to process (when no subcommand is specified)
    #[arg(value_name = "PATH")]
    path: Option<PathBuf>,

    /// Run a named preset from reformat.json
    #[arg(
        short = 'p',
        long = "preset",
        requires = "path",
        conflicts_with = "job"
    )]
    preset: Option<String>,

    /// Run an ad-hoc job from a JSON file (or "-" for stdin)
    #[arg(
        short = 'j',
        long = "job",
        requires = "path",
        conflicts_with = "preset"
    )]
    job: Option<String>,

    /// Process files recursively (when no subcommand is specified)
    #[arg(short = 'r', long, requires = "path")]
    recursive: bool,

    /// Dry run (don't modify files)
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,

    /// Enable verbose output (can be used multiple times: -v, -vv, -vvv)
    #[arg(short = 'v', long = "verbose", global = true, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output except errors
    #[arg(short = 'q', long = "quiet", global = true)]
    quiet: bool,

    /// Write logs to file
    #[arg(long = "log-file", global = true)]
    log_file: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert between case formats
    #[command(group(clap::ArgGroup::new("from").required(true).multiple(false)))]
    #[command(group(clap::ArgGroup::new("to").required(true).multiple(false)))]
    Convert {
        /// Convert FROM camelCase
        #[arg(long = "from-camel", group = "from")]
        from_camel: bool,

        /// Convert FROM PascalCase
        #[arg(long = "from-pascal", group = "from")]
        from_pascal: bool,

        /// Convert FROM snake_case
        #[arg(long = "from-snake", group = "from")]
        from_snake: bool,

        /// Convert FROM SCREAMING_SNAKE_CASE
        #[arg(long = "from-screaming-snake", group = "from")]
        from_screaming_snake: bool,

        /// Convert FROM kebab-case
        #[arg(long = "from-kebab", group = "from")]
        from_kebab: bool,

        /// Convert FROM SCREAMING-KEBAB-CASE
        #[arg(long = "from-screaming-kebab", group = "from")]
        from_screaming_kebab: bool,

        /// Convert TO camelCase
        #[arg(long = "to-camel", group = "to")]
        to_camel: bool,

        /// Convert TO PascalCase
        #[arg(long = "to-pascal", group = "to")]
        to_pascal: bool,

        /// Convert TO snake_case
        #[arg(long = "to-snake", group = "to")]
        to_snake: bool,

        /// Convert TO SCREAMING_SNAKE_CASE
        #[arg(long = "to-screaming-snake", group = "to")]
        to_screaming_snake: bool,

        /// Convert TO kebab-case
        #[arg(long = "to-kebab", group = "to")]
        to_kebab: bool,

        /// Convert TO SCREAMING-KEBAB-CASE
        #[arg(long = "to-screaming-kebab", group = "to")]
        to_screaming_kebab: bool,

        /// The directory or file to convert
        path: PathBuf,

        /// Convert files recursively
        #[arg(short = 'r', long)]
        recursive: bool,

        /// Dry run the conversion
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,

        /// Prefix to add to all converted words
        #[arg(long, default_value = "")]
        prefix: String,

        /// Suffix to add to all converted words
        #[arg(long, default_value = "")]
        suffix: String,

        /// Strip prefix before conversion (e.g., 'm_' from 'm_userName')
        #[arg(long = "strip-prefix")]
        strip_prefix: Option<String>,

        /// Strip suffix before conversion
        #[arg(long = "strip-suffix")]
        strip_suffix: Option<String>,

        /// Replace prefix (from) before conversion (e.g., 'I' in 'IUserService')
        #[arg(long = "replace-prefix-from")]
        replace_prefix_from: Option<String>,

        /// Replace prefix (to) before conversion (e.g., 'Abstract')
        #[arg(long = "replace-prefix-to", requires = "replace_prefix_from")]
        replace_prefix_to: Option<String>,

        /// Replace suffix (from) before conversion
        #[arg(long = "replace-suffix-from")]
        replace_suffix_from: Option<String>,

        /// Replace suffix (to) before conversion
        #[arg(long = "replace-suffix-to", requires = "replace_suffix_from")]
        replace_suffix_to: Option<String>,

        /// Glob pattern to filter files
        #[arg(long)]
        glob: Option<String>,

        /// Regex pattern to filter which words get converted
        #[arg(long = "word-filter")]
        word_filter: Option<String>,
    },

    /// Remove trailing whitespace from files
    Clean {
        /// The directory or file to clean
        path: PathBuf,

        /// Process files recursively
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,
    },

    /// Remove or replace emojis with text alternatives
    Emojis {
        /// The directory or file to process
        path: PathBuf,

        /// Process files recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process (default: .md, .txt, and common source files)
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,

        /// Replace task completion emojis with text (e.g., ✅ -> [x]) [default: true]
        #[arg(long = "replace-task", default_value_t = true)]
        replace_task: bool,

        /// Remove all other emojis [default: true]
        #[arg(long = "remove-other", default_value_t = true)]
        remove_other: bool,
    },

    /// Rename files with various transformations
    #[command(name = "rename_files")]
    RenameFiles {
        /// The directory or file to rename
        path: PathBuf,

        /// Process directories recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't rename files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// Include symbolic links in processing
        #[arg(long = "include-symlinks")]
        include_symlinks: bool,

        /// Convert to lowercase
        #[arg(long = "to-lowercase")]
        to_lowercase: bool,

        /// Convert to UPPERCASE
        #[arg(long = "to-uppercase")]
        to_uppercase: bool,

        /// Capitalize (first letter uppercase, rest lowercase)
        #[arg(long = "to-capitalize")]
        to_capitalize: bool,

        /// Replace separators (spaces, hyphens, underscores) with underscores
        #[arg(long = "underscored")]
        underscored: bool,

        /// Replace separators (spaces, hyphens, underscores) with hyphens
        #[arg(long = "hyphenated")]
        hyphenated: bool,

        /// Add prefix to filename
        #[arg(long = "add-prefix")]
        add_prefix: Option<String>,

        /// Remove prefix from filename
        #[arg(long = "rm-prefix")]
        rm_prefix: Option<String>,

        /// Add suffix to filename (before extension)
        #[arg(long = "add-suffix")]
        add_suffix: Option<String>,

        /// Remove suffix from filename (before extension)
        #[arg(long = "rm-suffix")]
        rm_suffix: Option<String>,

        /// Replace prefix in filename (two arguments: <old> <new>)
        #[arg(long = "replace-prefix", num_args = 2, value_names = ["OLD", "NEW"])]
        replace_prefix: Option<Vec<String>>,

        /// Replace suffix in filename (two arguments: <old> <new>)
        #[arg(long = "replace-suffix", num_args = 2, value_names = ["OLD", "NEW"])]
        replace_suffix: Option<Vec<String>>,

        /// Add timestamp prefix in YYYYMMDD format (e.g., 20250915_)
        #[arg(long = "timestamp-long")]
        timestamp_long: bool,

        /// Add timestamp prefix in YYMMDD format (e.g., 250915_)
        #[arg(long = "timestamp-short")]
        timestamp_short: bool,
    },

    /// Group files by common prefix into subdirectories
    #[command(name = "group")]
    Group {
        /// The directory to process
        path: PathBuf,

        /// Process subdirectories recursively
        #[arg(short = 'r', long)]
        recursive: bool,

        /// Dry run (don't move files or create directories)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// Separator character that divides prefix from rest of filename
        #[arg(short = 's', long = "separator", default_value_t = '_')]
        separator: char,

        /// Minimum number of files with same prefix to create a group
        #[arg(short = 'm', long = "min-count", default_value_t = 2)]
        min_count: usize,

        /// Remove the prefix from filenames after moving to subdirectory
        #[arg(long = "strip-prefix")]
        strip_prefix: bool,

        /// Group by suffix: split at LAST separator, use suffix as filename
        /// e.g., "activity_relationships_list.tmpl" -> "activity_relationships/list.tmpl"
        /// Implies --strip-prefix
        #[arg(long = "from-suffix")]
        from_suffix: bool,

        /// Preview groups without making changes (shows what would be grouped)
        #[arg(long = "preview")]
        preview: bool,

        /// Skip interactive prompts for reference scanning
        #[arg(long = "no-interactive")]
        no_interactive: bool,

        /// Directory to scan recursively for broken references caused by the grouping
        #[arg(long = "scope")]
        scope: Option<PathBuf>,

        /// Show verbose output during reference scanning (useful for debugging hangs)
        #[arg(long = "verbose-scan")]
        verbose_scan: bool,

        /// Where to write the record of moves [default: ./changes.json]
        #[arg(long = "changes-file")]
        changes_file: Option<PathBuf>,

        /// Where to write proposed reference fixes [default: ./fixes.json]
        #[arg(long = "fixes-file")]
        fixes_file: Option<PathBuf>,
    },

    /// Normalize line endings across files
    Endings {
        /// The directory or file to process
        path: PathBuf,

        /// Target line ending style: lf, crlf, or cr
        #[arg(short = 's', long = "style", default_value = "lf")]
        style: String,

        /// Process files recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,
    },

    /// Normalize indentation (convert between tabs and spaces)
    Indent {
        /// The directory or file to process
        path: PathBuf,

        /// Target indent style: spaces or tabs
        #[arg(short = 's', long = "style", default_value = "spaces")]
        style: String,

        /// Number of spaces per indent level (or tab width for conversion)
        #[arg(short = 'w', long = "width", default_value_t = 4)]
        width: usize,

        /// Process files recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,
    },

    /// Regex find-and-replace across files
    Replace {
        /// The directory or file to process
        path: PathBuf,

        /// Find pattern (regex)
        #[arg(short = 'f', long = "find")]
        find: String,

        /// Replacement string (supports capture groups: $1, $2, etc.)
        #[arg(long = "replace-with")]
        replace_with: String,

        /// Process files recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,
    },

    /// Insert or update file headers (license, copyright, etc.)
    Header {
        /// The directory or file to process
        path: PathBuf,

        /// Header text to insert (use \n for newlines)
        #[arg(short = 't', long = "text")]
        text: String,

        /// Replace {year} in header text with the current year
        #[arg(long = "update-year")]
        update_year: bool,

        /// Process files recursively [default: true]
        #[arg(short = 'r', long, default_value_t = true)]
        recursive: bool,

        /// Dry run (don't modify files)
        #[arg(short = 'd', long = "dry-run")]
        dry_run: bool,

        /// File extensions to process
        #[arg(short = 'e', long = "extensions")]
        extensions: Option<Vec<String>>,
    },
}

/// Initialize logging based on verbosity level
fn init_logging(verbose: u8, quiet: bool, log_file: Option<PathBuf>) -> anyhow::Result<()> {
    // Transformers report what they touch at `info`. That is the default
    // level so ordinary runs look the same as before, and `--quiet` now
    // actually silences them -- previously they were `println!`d straight from
    // the library, where no CLI flag could reach them.
    let log_level = if quiet {
        LevelFilter::Error
    } else {
        match verbose {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        }
    };

    // Terminal output is undecorated: these are user-facing progress lines,
    // not diagnostics, so no timestamp, level or target prefix.
    let term_config = ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .set_max_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .set_location_level(LevelFilter::Off)
        .build();

    // The log file keeps full detail, timestamps included.
    let file_config = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_thread_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .build();

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        log_level,
        term_config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];

    if let Some(log_path) = log_file {
        let file = std::fs::File::create(&log_path)?;
        loggers.push(WriteLogger::new(LevelFilter::Debug, file_config, file));
        warn!("Logging to file: {}", log_path.display());
    }

    CombinedLogger::init(loggers)?;

    debug!("Logging initialized with level: {:?}", log_level);
    Ok(())
}

/// Picks the single selected case format from a group of mutually exclusive
/// clap flags. The group is `required`, so exactly one is set.
fn selected_format(
    camel: bool,
    pascal: bool,
    snake: bool,
    screaming_snake: bool,
    kebab: bool,
) -> &'static str {
    if camel {
        "camel"
    } else if pascal {
        "pascal"
    } else if snake {
        "snake"
    } else if screaming_snake {
        "screaming_snake"
    } else if kebab {
        "kebab"
    } else {
        "screaming_kebab"
    }
}

#[allow(clippy::too_many_arguments)]
#[time("debug")]
fn run_convert(
    from_camel: bool,
    from_pascal: bool,
    from_snake: bool,
    from_screaming_snake: bool,
    from_kebab: bool,
    _from_screaming_kebab: bool,
    to_camel: bool,
    to_pascal: bool,
    to_snake: bool,
    to_screaming_snake: bool,
    to_kebab: bool,
    _to_screaming_kebab: bool,
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
    prefix: String,
    suffix: String,
    strip_prefix: Option<String>,
    strip_suffix: Option<String>,
    replace_prefix_from: Option<String>,
    replace_prefix_to: Option<String>,
    replace_suffix_from: Option<String>,
    replace_suffix_to: Option<String>,
    glob: Option<String>,
    word_filter: Option<String>,
) -> anyhow::Result<()> {
    let cfg = ConvertConfig {
        from_format: Some(
            selected_format(
                from_camel,
                from_pascal,
                from_snake,
                from_screaming_snake,
                from_kebab,
            )
            .to_string(),
        ),
        to_format: Some(
            selected_format(to_camel, to_pascal, to_snake, to_screaming_snake, to_kebab)
                .to_string(),
        ),
        file_extensions: extensions,
        recursive: Some(recursive),
        prefix: Some(prefix),
        suffix: Some(suffix),
        glob,
        word_filter,
        strip_prefix,
        strip_suffix,
        replace_prefix_from,
        replace_prefix_to,
        replace_suffix_from,
        replace_suffix_to,
    };

    run_single_step(
        "convert",
        reformat_core::Preset {
            steps: vec!["convert".to_string()],
            convert: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |_| "Conversion complete".to_string(),
        "No changes needed",
    )
}

#[time("debug")]
fn run_clean(
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let cfg = CleanConfig {
        remove_trailing: Some(true),
        file_extensions: extensions,
        recursive: Some(recursive),
    };
    run_single_step(
        "clean",
        reformat_core::Preset {
            steps: vec!["clean".to_string()],
            clean: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |o| match o {
            StepOutcome::Counted { files, units } => {
                format!("Cleaned {} lines in {} file(s)", units, files)
            }
            _ => unreachable!(),
        },
        "No files needed cleaning",
    )
}

#[time("debug")]
fn run_emojis(
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
    replace_task: bool,
    remove_other: bool,
) -> anyhow::Result<()> {
    let cfg = EmojiConfig {
        replace_task_emojis: Some(replace_task),
        remove_other_emojis: Some(remove_other),
        file_extensions: extensions,
        recursive: Some(recursive),
    };
    run_single_step(
        "emojis",
        reformat_core::Preset {
            steps: vec!["emojis".to_string()],
            emojis: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |o| match o {
            StepOutcome::Counted { files, units } => format!(
                "Transformed emojis in {} file(s) ({} changes)",
                files, units
            ),
            _ => unreachable!(),
        },
        "No files contained emojis to transform",
    )
}

#[allow(clippy::too_many_arguments)]
#[time("debug")]
fn run_rename(
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    include_symlinks: bool,
    to_lowercase: bool,
    to_uppercase: bool,
    to_capitalize: bool,
    underscored: bool,
    hyphenated: bool,
    add_prefix: Option<String>,
    rm_prefix: Option<String>,
    add_suffix: Option<String>,
    rm_suffix: Option<String>,
    replace_prefix: Option<Vec<String>>,
    replace_suffix: Option<Vec<String>>,
    timestamp_long: bool,
    timestamp_short: bool,
) -> anyhow::Result<()> {
    let case_transform = if to_lowercase {
        Some("lowercase")
    } else if to_uppercase {
        Some("uppercase")
    } else if to_capitalize {
        Some("capitalize")
    } else {
        None
    };
    let space_replace = if underscored {
        Some("underscore")
    } else if hyphenated {
        Some("hyphen")
    } else {
        None
    };
    let timestamp = if timestamp_long {
        Some("long")
    } else if timestamp_short {
        Some("short")
    } else {
        None
    };

    let cfg = RenameConfig {
        case_transform: case_transform.map(String::from),
        space_replace: space_replace.map(String::from),
        recursive: Some(recursive),
        include_symlinks: Some(include_symlinks),
        add_prefix,
        remove_prefix: rm_prefix,
        add_suffix,
        remove_suffix: rm_suffix,
        replace_prefix,
        replace_suffix,
        timestamp: timestamp.map(String::from),
    };
    run_single_step(
        "rename",
        reformat_core::Preset {
            steps: vec!["rename".to_string()],
            rename: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |o| match o {
            StepOutcome::Renamed(s) => format!("Renamed {} file(s)", s.renamed),
            _ => unreachable!(),
        },
        "No files needed renaming",
    )
}

/// Resolves where a JSON record should be written, warning before replacing
/// an existing file rather than silently clobbering it.
fn resolve_record_path(explicit: Option<PathBuf>, default_name: &str) -> anyhow::Result<PathBuf> {
    let path = match explicit {
        Some(p) => p,
        None => std::env::current_dir()?.join(default_name),
    };
    if path.exists() {
        warn!("Overwriting existing file: {}", path.display());
    }
    Ok(path)
}

/// Prompts the user for a yes/no answer
fn prompt_yes_no(question: &str) -> bool {
    print!("{} [y/N]: ", question);
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Checks if scope_path contains or is a parent of target_path
/// Returns a warning message if there's an overlap, None otherwise
fn check_scope_overlap(scope_path: &Path, target_path: &Path) -> Option<String> {
    // Canonicalize both paths for accurate comparison
    let scope_canonical = scope_path
        .canonicalize()
        .unwrap_or_else(|_| scope_path.to_path_buf());
    let target_canonical = target_path
        .canonicalize()
        .unwrap_or_else(|_| target_path.to_path_buf());

    // Check if scope contains target (scope is parent of target)
    if target_canonical.starts_with(&scope_canonical) {
        return Some(format!(
            "Warning: --scope '{}' contains the target directory '{}'\n\
             This will scan the newly created group directories and may be slow.\n\
             Consider using a more specific --scope that only includes directories\n\
             with files that reference the moved files (e.g., --scope ./src).",
            scope_path.display(),
            target_path.display()
        ));
    }

    // Check if target contains scope (target is parent of scope) - less common but worth noting
    if scope_canonical.starts_with(&target_canonical) {
        return Some(format!(
            "Warning: --scope '{}' is inside the target directory '{}'.\n\
             This is unusual - typically --scope should point to directories\n\
             containing files that reference the moved files.",
            scope_path.display(),
            target_path.display()
        ));
    }

    None
}

/// Prompts the user for directories to scan
fn prompt_scan_dirs(default_dir: &Path) -> Vec<PathBuf> {
    print!(
        "Enter directories to scan (comma-separated, or press Enter for '{}'): ",
        default_dir.display()
    );
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() || input.trim().is_empty() {
        return vec![default_dir.to_path_buf()];
    }

    input
        .trim()
        .split(',')
        .map(|s| PathBuf::from(s.trim()))
        .collect()
}

#[allow(clippy::too_many_arguments)]
#[time("debug")]
fn run_group(
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    separator: char,
    min_count: usize,
    strip_prefix: bool,
    from_suffix: bool,
    preview: bool,
    no_interactive: bool,
    scope: Option<PathBuf>,
    verbose_scan: bool,
    changes_file: Option<PathBuf>,
    fixes_file: Option<PathBuf>,
) -> anyhow::Result<()> {
    debug!("Grouping files by prefix in: {}", path.display());
    debug!(
        "Recursive: {}, Dry run: {}, Separator: '{}', Min count: {}",
        recursive, dry_run, separator, min_count
    );
    if from_suffix {
        debug!("From suffix: enabled (splitting at last separator)");
    }
    if strip_prefix || from_suffix {
        debug!("Strip prefix: enabled");
    }

    let cfg = GroupConfig {
        separator: Some(separator.to_string()),
        min_count: Some(min_count),
        strip_prefix: Some(strip_prefix),
        from_suffix: Some(from_suffix),
        recursive: Some(recursive),
    };

    let grouper = FileGrouper::new(cfg.to_options(dry_run)?);

    if preview {
        let groups = grouper.preview(&path)?;

        if groups.is_empty() {
            info!(
                "No file groups found matching criteria (min_count: {})",
                min_count
            );
        } else {
            info!("Found {} potential group(s):", groups.len());
            for (prefix, files) in &groups {
                info!("\n  {} ({} files):", prefix, files.len());
                for file in files {
                    info!("    - {}", file);
                }
            }
        }
        return Ok(());
    }
    let result = grouper.process_with_changes(&path)?;

    let stats = &result.stats;

    if stats.files_moved > 0 {
        let prefix_str = if dry_run { "[DRY-RUN] " } else { "" };
        debug!(
            "{}Grouping complete: {} directories created, {} files moved",
            prefix_str, stats.dirs_created, stats.files_moved
        );
        info!("{}Grouping complete:", prefix_str);
        if stats.dirs_created > 0 {
            info!("  - Directories created: {}", stats.dirs_created);
        }
        info!("  - Files moved: {}", stats.files_moved);
        if stats.files_renamed > 0 {
            info!(
                "  - Files renamed (prefix stripped): {}",
                stats.files_renamed
            );
        }

        // A dry run must not leave anything behind. Writing the record
        // anyway was worse than untidy: it described moves that had not
        // happened, and feeding it to the reference fixer would rewrite
        // references to files still sitting where they were.
        if dry_run {
            info!("[DRY-RUN] No changes file written.");
        } else if !result.changes.is_empty() {
            let changes_path = resolve_record_path(changes_file, "changes.json")?;
            result.changes.write_to_file(&changes_path)?;
            info!("\nChanges recorded to: {}", changes_path.display());

            // Interactive workflow for reference scanning
            if !dry_run && !no_interactive {
                info!("");
                if prompt_yes_no("Would you like to scan for broken references?") {
                    let dirs_to_scan = if let Some(dir) = scope {
                        vec![dir]
                    } else {
                        prompt_scan_dirs(&path)
                    };

                    // Check for scope/target overlap and warn
                    for scan_dir in &dirs_to_scan {
                        if let Some(warning) = check_scope_overlap(scan_dir, &path) {
                            warn!("\n{}\n", warning);
                        }
                    }

                    // Scan for broken references
                    let scan_options = ScanOptions {
                        verbose: verbose_scan,
                        ..Default::default()
                    };

                    debug!("Scanning for broken references...");
                    let scanner =
                        ReferenceScanner::from_change_record(&result.changes, scan_options)?;
                    let fix_record = scanner.scan(&dirs_to_scan)?;

                    if fix_record.is_empty() {
                        info!("No broken references found.");
                    } else {
                        // Write fixes.json
                        let fixes_path = resolve_record_path(fixes_file.clone(), "fixes.json")?;
                        fix_record.write_to_file(&fixes_path)?;
                        info!("\nFound {} broken reference(s).", fix_record.len());
                        info!("Proposed fixes written to: {}", fixes_path.display());

                        // Show summary of fixes
                        info!("\nProposed fixes:");
                        for fix in fix_record.fixes.iter().take(10) {
                            info!(
                                "  {}:{}: '{}' -> '{}'",
                                fix.file, fix.line, fix.old_reference, fix.new_reference
                            );
                        }
                        if fix_record.len() > 10 {
                            info!("  ... and {} more (see fixes.json)", fix_record.len() - 10);
                        }

                        info!("");
                        if prompt_yes_no("Review fixes.json and apply changes?") {
                            let apply_result = ReferenceFixer::apply_fixes(&fix_record)?;

                            info!(
                                "\nFixed {} reference(s) in {} file(s).",
                                apply_result.references_fixed, apply_result.files_modified
                            );

                            if apply_result.references_skipped > 0 {
                                info!(
                                    "Skipped {} reference(s): the recorded text no longer \
                                     matches the file. The fixes may already have been \
                                     applied, or the file changed since the scan.",
                                    apply_result.references_skipped
                                );
                            }

                            if !apply_result.errors.is_empty() {
                                info!("\nErrors encountered:");
                                for err in &apply_result.errors {
                                    info!("  - {}", err);
                                }
                            }
                        } else {
                            info!("Fixes not applied. You can review fixes.json and apply them later.");
                        }
                    }
                }
            } else if !dry_run && scope.is_some() {
                // Non-interactive mode with --scope specified
                let dirs_to_scan = vec![scope.unwrap()];

                // Check for scope/target overlap and warn
                for scan_dir in &dirs_to_scan {
                    if let Some(warning) = check_scope_overlap(scan_dir, &path) {
                        warn!("\n{}\n", warning);
                    }
                }

                let scan_options = ScanOptions {
                    verbose: verbose_scan,
                    ..Default::default()
                };

                debug!("Scanning for broken references...");
                let scanner = ReferenceScanner::from_change_record(&result.changes, scan_options)?;
                let fix_record = scanner.scan(&dirs_to_scan)?;

                if fix_record.is_empty() {
                    info!("\nNo broken references found.");
                } else {
                    let fixes_path = resolve_record_path(fixes_file, "fixes.json")?;
                    fix_record.write_to_file(&fixes_path)?;
                    info!("\nFound {} broken reference(s).", fix_record.len());
                    info!("Proposed fixes written to: {}", fixes_path.display());
                }
            }
        }
    } else {
        info!("No files needed grouping");
    }

    Ok(())
}

#[time("debug")]
fn run_endings(
    path: PathBuf,
    style: &str,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let cfg = EndingsConfig {
        style: Some(style.to_string()),
        file_extensions: extensions,
        recursive: Some(recursive),
    };
    run_single_step(
        "endings",
        reformat_core::Preset {
            steps: vec!["endings".to_string()],
            endings: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |o| match o {
            StepOutcome::Counted { files, units } => {
                format!("Normalized {} ending(s) in {} file(s)", units, files)
            }
            _ => unreachable!(),
        },
        "No files needed line ending normalization",
    )
}

#[time("debug")]
fn run_indent(
    path: PathBuf,
    style: &str,
    width: usize,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let cfg = IndentConfig {
        style: Some(style.to_string()),
        width: Some(width),
        file_extensions: extensions,
        recursive: Some(recursive),
    };
    run_single_step(
        "indent",
        reformat_core::Preset {
            steps: vec!["indent".to_string()],
            indent: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |o| match o {
            StepOutcome::Counted { files, units } => {
                format!("Normalized {} line(s) in {} file(s)", units, files)
            }
            _ => unreachable!(),
        },
        "No files needed indentation normalization",
    )
}

#[time("debug")]
fn run_replace(
    path: PathBuf,
    find: &str,
    replace_with: &str,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let cfg = ReplaceConfig {
        patterns: Some(vec![ReplacePatternEntry {
            find: find.to_string(),
            replace: replace_with.to_string(),
        }]),
        file_extensions: extensions,
        recursive: Some(recursive),
    };
    run_single_step(
        "replace",
        reformat_core::Preset {
            steps: vec!["replace".to_string()],
            replace: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |o| match o {
            StepOutcome::Counted { files, units } => {
                format!("Made {} replacement(s) in {} file(s)", units, files)
            }
            _ => unreachable!(),
        },
        "No files matched the pattern",
    )
}

#[time("debug")]
fn run_header(
    path: PathBuf,
    text: &str,
    update_year: bool,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
) -> anyhow::Result<()> {
    let cfg = HeaderConfig {
        // Allow \n in the argument to stand for a real newline.
        text: Some(text.replace("\\n", "\n")),
        update_year: Some(update_year),
        file_extensions: extensions,
        recursive: Some(recursive),
    };
    run_single_step(
        "header",
        reformat_core::Preset {
            steps: vec!["header".to_string()],
            header: Some(cfg),
            ..Default::default()
        },
        &path,
        dry_run,
        |o| match o {
            StepOutcome::Counted { files, .. } => {
                format!("Updated headers in {} file(s)", files)
            }
            _ => unreachable!(),
        },
        "All files already have correct headers",
    )
}

/// What a step did. Steps compute; callers present.
///
/// Option assembly is shared -- that was the real duplication -- but a
/// standalone subcommand and a pipeline step word their summaries
/// differently, so formatting stays with the caller.
enum StepOutcome {
    /// Files touched, plus a step-specific unit count (lines, changes,
    /// replacements, endings).
    Counted {
        files: usize,
        units: usize,
    },
    Renamed(reformat_core::RenameStats),
    Grouped(reformat_core::GroupStats),
    Converted,
}

impl StepOutcome {
    fn files(&self) -> usize {
        match self {
            StepOutcome::Counted { files, .. } => *files,
            StepOutcome::Renamed(s) => s.renamed,
            StepOutcome::Grouped(s) => s.files_moved,
            StepOutcome::Converted => 0,
        }
    }
}

/// Runs one pipeline step and reports what it did.
///
/// Both the subcommands and the preset/job runner funnel through here: each
/// builds the step's `*Config` and calls this. Previously the two paths
/// assembled options separately and had drifted -- the preset `convert` step
/// hardcoded `None` for the strip/replace affix settings, so half of
/// `ConvertConfig` was unreachable from a preset.
fn run_step(
    step: &str,
    preset: &reformat_core::Preset,
    path: &Path,
    dry_run: bool,
    label: &str,
) -> anyhow::Result<StepOutcome> {
    if !path.exists() {
        anyhow::bail!("path '{}' does not exist", path.display());
    }

    let missing = |what: &str| {
        anyhow::anyhow!(
            "{}: '{}' step requires a [{}] config with {}",
            label,
            step,
            step,
            what
        )
    };

    let outcome = match step {
        "rename" => {
            let cfg = preset.rename.clone().unwrap_or_default();
            StepOutcome::Renamed(
                FileRenamer::new(cfg.to_options(dry_run)?).process_with_stats(path)?,
            )
        }

        "emojis" => {
            let cfg = preset.emojis.clone().unwrap_or_default();
            let (files, units) = EmojiTransformer::new(cfg.to_options(dry_run)).process(path)?;
            StepOutcome::Counted { files, units }
        }

        "clean" => {
            let cfg = preset.clean.clone().unwrap_or_default();
            let (files, units) = WhitespaceCleaner::new(cfg.to_options(dry_run)).process(path)?;
            StepOutcome::Counted { files, units }
        }

        "convert" => {
            let cfg = preset
                .convert
                .clone()
                .ok_or_else(|| missing("from_format and to_format"))?;
            cfg.to_converter(dry_run)?.process_directory(path)?;
            StepOutcome::Converted
        }

        "group" => {
            let cfg = preset.group.clone().unwrap_or_default();
            StepOutcome::Grouped(
                FileGrouper::new(cfg.to_options(dry_run)?)
                    .process_with_changes(path)?
                    .stats,
            )
        }

        "endings" => {
            let cfg = preset.endings.clone().unwrap_or_default();
            let (files, units) = EndingsNormalizer::new(cfg.to_options(dry_run)?).process(path)?;
            StepOutcome::Counted { files, units }
        }

        "indent" => {
            let cfg = preset.indent.clone().unwrap_or_default();
            let (files, units) = IndentNormalizer::new(cfg.to_options(dry_run)?).process(path)?;
            StepOutcome::Counted { files, units }
        }

        "replace" => {
            let cfg = preset.replace.clone().ok_or_else(|| missing("patterns"))?;
            let (files, units) = ContentReplacer::new(cfg.to_options(dry_run)?)?.process(path)?;
            StepOutcome::Counted { files, units }
        }

        "header" => {
            let cfg = preset.header.clone().ok_or_else(|| missing("text"))?;
            let (files, units) = HeaderManager::new(cfg.to_options(dry_run)?)?.process(path)?;
            StepOutcome::Counted { files, units }
        }

        _ => unreachable!("step validation should have caught this"),
    };

    Ok(outcome)
}

/// Wraps a single step as a standalone subcommand, presenting the result the
/// way that command always has.
fn run_single_step(
    step: &str,
    preset: reformat_core::Preset,
    path: &Path,
    dry_run: bool,
    summary: impl Fn(&StepOutcome) -> String,
    empty: &str,
) -> anyhow::Result<()> {
    let outcome = run_step(step, &preset, path, dry_run, step)?;
    let prefix = if dry_run { "[DRY-RUN] " } else { "" };

    if let StepOutcome::Renamed(ref stats) = outcome {
        if stats.skipped > 0 {
            warn!("Skipped {} file(s):", stats.skipped);
            for message in &stats.errors {
                warn!("  {}", message);
            }
        }
    }

    if outcome.files() > 0 || matches!(outcome, StepOutcome::Converted) {
        info!("{}{}", prefix, summary(&outcome));
    } else {
        info!("{}", empty);
    }
    Ok(())
}

/// Core pipeline execution engine. Both presets and jobs use this.
fn run_pipeline(
    name: &str,
    preset: &reformat_core::Preset,
    path: &Path,
    dry_run: bool,
) -> anyhow::Result<()> {
    reformat_core::config::validate_steps(name, &preset.steps)?;

    debug!(
        "Running '{}' with {} step(s) on: {}",
        name,
        preset.steps.len(),
        path.display()
    );

    let prefix = if dry_run { "[DRY-RUN] " } else { "" };

    for (i, step) in preset.steps.iter().enumerate() {
        debug!(
            "Executing step [{}/{}]: {}",
            i + 1,
            preset.steps.len(),
            step
        );
        let outcome = run_step(step, preset, path, dry_run, name)?;

        match &outcome {
            StepOutcome::Converted => info!("{}  {}: complete", prefix, step),
            StepOutcome::Renamed(stats) => {
                if stats.renamed > 0 {
                    info!("{}  rename: {} file(s) renamed", prefix, stats.renamed);
                } else {
                    info!("  rename: no files needed renaming");
                }
                if stats.skipped > 0 {
                    warn!("  rename: skipped {} file(s)", stats.skipped);
                    for message in &stats.errors {
                        warn!("    {}", message);
                    }
                }
            }
            StepOutcome::Grouped(stats) => {
                if stats.files_moved > 0 {
                    info!(
                        "{}  group: {} dir(s) created, {} file(s) moved",
                        prefix, stats.dirs_created, stats.files_moved
                    );
                } else {
                    info!("  group: no files needed grouping");
                }
            }
            StepOutcome::Counted { files, units } => {
                if *files > 0 {
                    info!(
                        "{}  {}: {} file(s), {} change(s)",
                        prefix, step, files, units
                    );
                } else {
                    info!("  {}: nothing to do", step);
                }
            }
        }
    }

    info!("Pipeline '{}' complete.", name);
    Ok(())
}

#[time("debug")]
fn run_preset(name: &str, path: PathBuf, dry_run: bool) -> anyhow::Result<()> {
    let config = config::load_config()?
        .ok_or_else(|| anyhow::anyhow!("reformat.json not found in current directory"))?;
    let preset = config::get_preset(&config, name)?;
    run_pipeline(name, preset, &path, dry_run)
}

#[time("debug")]
fn run_job(job_source: &str, path: PathBuf, dry_run: bool) -> anyhow::Result<()> {
    let content = if job_source == "-" {
        debug!("Reading job from stdin");
        let mut buf = String::new();
        io::Read::read_to_string(&mut io::stdin(), &mut buf)?;
        buf
    } else {
        debug!("Reading job from file: {}", job_source);
        std::fs::read_to_string(job_source)
            .map_err(|e| anyhow::anyhow!("failed to read job file '{}': {}", job_source, e))?
    };

    let preset: reformat_core::Preset = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse job: {}", e))?;

    let label = if job_source == "-" {
        "stdin"
    } else {
        job_source
    };
    run_pipeline(label, &preset, &path, dry_run)
}

#[time("debug")]
fn run_combined(path: PathBuf, recursive: bool, dry_run: bool) -> anyhow::Result<()> {
    debug!("Running combined transformations on: {}", path.display());
    debug!("Recursive: {}, Dry run: {}", recursive, dry_run);

    if !path.exists() {
        anyhow::bail!("path '{}' does not exist", path.display());
    }

    let options = CombinedOptions { recursive, dry_run };

    let processor = CombinedProcessor::new(options);
    let stats = processor.process(&path)?;

    let prefix = if dry_run { "[DRY-RUN] " } else { "" };

    // Print summary
    if stats.files_renamed > 0
        || stats.files_emoji_transformed > 0
        || stats.files_whitespace_cleaned > 0
    {
        debug!(
            "{}Combined processing complete: {} renamed, {} emoji-transformed ({} changes), {} whitespace-cleaned ({} lines)",
            prefix, stats.files_renamed, stats.files_emoji_transformed, stats.emoji_changes,
            stats.files_whitespace_cleaned, stats.whitespace_lines_cleaned
        );
        info!("{}Processed files:", prefix);
        if stats.files_renamed > 0 {
            info!("  - Renamed: {} file(s)", stats.files_renamed);
        }
        if stats.files_emoji_transformed > 0 {
            info!(
                "  - Emoji transformations: {} file(s) ({} changes)",
                stats.files_emoji_transformed, stats.emoji_changes
            );
        }
        if stats.files_whitespace_cleaned > 0 {
            info!(
                "  - Whitespace cleaned: {} file(s) ({} lines)",
                stats.files_whitespace_cleaned, stats.whitespace_lines_cleaned
            );
        }
    } else {
        info!("No files needed processing");
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    if let Err(e) = init_logging(cli.verbose, cli.quiet, cli.log_file.clone()) {
        warn!("Warning: Failed to initialize logging: {}", e);
    }

    debug!("CLI arguments parsed successfully");

    let result = match cli.command {
        None => {
            if let Some(preset_name) = cli.preset {
                // Preset mode: run named preset from reformat.json
                if let Some(path) = cli.path {
                    debug!("Running preset '{}'", preset_name);
                    run_preset(&preset_name, path, cli.dry_run)
                } else {
                    log::error!("No path specified. Usage: reformat -p <preset> <path>");
                    std::process::exit(1);
                }
            } else if let Some(job_source) = cli.job {
                // Job mode: run ad-hoc pipeline from file or stdin
                if let Some(path) = cli.path {
                    debug!("Running job from '{}'", job_source);
                    run_job(&job_source, path, cli.dry_run)
                } else {
                    log::error!("No path specified. Usage: reformat --job <file|-> <path>");
                    std::process::exit(1);
                }
            } else if let Some(path) = cli.path {
                // Default command: run combined processing
                debug!("Running combined processing (default command)");
                run_combined(path, cli.recursive, cli.dry_run)
            } else {
                // Neither command nor path specified - print help
                log::error!("No command or path specified. Use --help for usage information.");
                std::process::exit(1);
            }
        }

        Some(cmd) => match cmd {
            Commands::Convert {
                from_camel,
                from_pascal,
                from_snake,
                from_screaming_snake,
                from_kebab,
                from_screaming_kebab,
                to_camel,
                to_pascal,
                to_snake,
                to_screaming_snake,
                to_kebab,
                to_screaming_kebab,
                path,
                recursive,
                dry_run,
                extensions,
                prefix,
                suffix,
                strip_prefix,
                strip_suffix,
                replace_prefix_from,
                replace_prefix_to,
                replace_suffix_from,
                replace_suffix_to,
                glob,
                word_filter,
            } => {
                debug!("Running convert subcommand");
                run_convert(
                    from_camel,
                    from_pascal,
                    from_snake,
                    from_screaming_snake,
                    from_kebab,
                    from_screaming_kebab,
                    to_camel,
                    to_pascal,
                    to_snake,
                    to_screaming_snake,
                    to_kebab,
                    to_screaming_kebab,
                    path,
                    recursive,
                    dry_run,
                    extensions,
                    prefix,
                    suffix,
                    strip_prefix,
                    strip_suffix,
                    replace_prefix_from,
                    replace_prefix_to,
                    replace_suffix_from,
                    replace_suffix_to,
                    glob,
                    word_filter,
                )
            }

            Commands::Clean {
                path,
                recursive,
                dry_run,
                extensions,
            } => {
                debug!("Running clean subcommand");
                run_clean(path, recursive, dry_run, extensions)
            }

            Commands::Emojis {
                path,
                recursive,
                dry_run,
                extensions,
                replace_task,
                remove_other,
            } => {
                debug!("Running emojis subcommand");
                run_emojis(
                    path,
                    recursive,
                    dry_run,
                    extensions,
                    replace_task,
                    remove_other,
                )
            }

            Commands::RenameFiles {
                path,
                recursive,
                dry_run,
                include_symlinks,
                to_lowercase,
                to_uppercase,
                to_capitalize,
                underscored,
                hyphenated,
                add_prefix,
                rm_prefix,
                add_suffix,
                rm_suffix,
                replace_prefix,
                replace_suffix,
                timestamp_long,
                timestamp_short,
            } => {
                debug!("Running rename subcommand");
                run_rename(
                    path,
                    recursive,
                    dry_run,
                    include_symlinks,
                    to_lowercase,
                    to_uppercase,
                    to_capitalize,
                    underscored,
                    hyphenated,
                    add_prefix,
                    rm_prefix,
                    add_suffix,
                    rm_suffix,
                    replace_prefix,
                    replace_suffix,
                    timestamp_long,
                    timestamp_short,
                )
            }

            Commands::Group {
                path,
                recursive,
                dry_run,
                separator,
                min_count,
                strip_prefix,
                from_suffix,
                preview,
                no_interactive,
                scope,
                verbose_scan,
                changes_file,
                fixes_file,
            } => {
                debug!("Running group subcommand");
                run_group(
                    path,
                    recursive,
                    dry_run,
                    separator,
                    min_count,
                    strip_prefix,
                    from_suffix,
                    preview,
                    no_interactive,
                    scope,
                    verbose_scan,
                    changes_file,
                    fixes_file,
                )
            }

            Commands::Endings {
                path,
                style,
                recursive,
                dry_run,
                extensions,
            } => {
                debug!("Running endings subcommand");
                run_endings(path, &style, recursive, dry_run, extensions)
            }

            Commands::Indent {
                path,
                style,
                width,
                recursive,
                dry_run,
                extensions,
            } => {
                debug!("Running indent subcommand");
                run_indent(path, &style, width, recursive, dry_run, extensions)
            }

            Commands::Replace {
                path,
                find,
                replace_with,
                recursive,
                dry_run,
                extensions,
            } => {
                debug!("Running replace subcommand");
                run_replace(path, &find, &replace_with, recursive, dry_run, extensions)
            }

            Commands::Header {
                path,
                text,
                update_year,
                recursive,
                dry_run,
                extensions,
            } => {
                debug!("Running header subcommand");
                run_header(path, &text, update_year, recursive, dry_run, extensions)
            }
        },
    };

    if result.is_ok() {
        debug!("Operation completed successfully");
    }

    // The error itself is reported once, by anyhow's top-level handler.
    result
}
