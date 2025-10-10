use clap::{Parser, Subcommand};
use codeconvert_core::{CaseConverter, CaseFormat, WhitespaceCleaner, WhitespaceOptions};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "codeconvert",
    version = "0.2.0",
    about = "Code transformation tool for case conversion and cleaning",
    long_about = "A modular code transformation framework.\n\n\
                  Commands:\n\
                  - convert: Convert between case formats\n\
                  - clean: Remove trailing whitespace"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Legacy flags (when no subcommand is used)
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

    /// The directory or file to convert (legacy mode)
    path: Option<PathBuf>,

    /// Convert files recursively (legacy mode)
    #[arg(short = 'r', long)]
    recursive: bool,

    /// Dry run the conversion (legacy mode)
    #[arg(short = 'd', long = "dry-run")]
    dry_run: bool,

    /// File extensions to process (legacy mode)
    #[arg(short = 'e', long = "extensions")]
    extensions: Option<Vec<String>>,

    /// Prefix to add to all converted words (legacy mode)
    #[arg(long, default_value = "")]
    prefix: String,

    /// Suffix to add to all converted words (legacy mode)
    #[arg(long, default_value = "")]
    suffix: String,

    /// Glob pattern to filter files (legacy mode)
    #[arg(long)]
    glob: Option<String>,

    /// Regex pattern to filter which words get converted (legacy mode)
    #[arg(long = "word-filter")]
    word_filter: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert between case formats
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
}

fn determine_case_format(
    from_camel: bool,
    from_pascal: bool,
    from_snake: bool,
    from_screaming_snake: bool,
    from_kebab: bool,
    _from_screaming_kebab: bool,
) -> CaseFormat {
    if from_camel {
        CaseFormat::CamelCase
    } else if from_pascal {
        CaseFormat::PascalCase
    } else if from_snake {
        CaseFormat::SnakeCase
    } else if from_screaming_snake {
        CaseFormat::ScreamingSnakeCase
    } else if from_kebab {
        CaseFormat::KebabCase
    } else {
        CaseFormat::ScreamingKebabCase
    }
}

fn run_convert(
    from_camel: bool,
    from_pascal: bool,
    from_snake: bool,
    from_screaming_snake: bool,
    from_kebab: bool,
    from_screaming_kebab: bool,
    to_camel: bool,
    to_pascal: bool,
    to_snake: bool,
    to_screaming_snake: bool,
    to_kebab: bool,
    to_screaming_kebab: bool,
    path: PathBuf,
    recursive: bool,
    dry_run: bool,
    extensions: Option<Vec<String>>,
    prefix: String,
    suffix: String,
    glob: Option<String>,
    word_filter: Option<String>,
) -> anyhow::Result<()> {
    let from_format = determine_case_format(
        from_camel,
        from_pascal,
        from_snake,
        from_screaming_snake,
        from_kebab,
        from_screaming_kebab,
    );

    let to_format = determine_case_format(
        to_camel,
        to_pascal,
        to_snake,
        to_screaming_snake,
        to_kebab,
        to_screaming_kebab,
    );

    let converter = CaseConverter::new(
        from_format,
        to_format,
        extensions,
        recursive,
        dry_run,
        prefix,
        suffix,
        glob,
        word_filter,
    )?;

    converter.process_directory(&path)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Convert {
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
            glob,
            word_filter,
        }) => {
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
                glob,
                word_filter,
            )
        }

        Some(Commands::Clean {
            path,
            recursive,
            dry_run,
            extensions,
        }) => {
            let mut options = WhitespaceOptions::default();
            options.recursive = recursive;
            options.dry_run = dry_run;

            if let Some(exts) = extensions {
                options.file_extensions = exts;
            }

            let cleaner = WhitespaceCleaner::new(options);
            let (files, lines) = cleaner.process(&path)?;

            if files > 0 {
                let prefix = if dry_run { "[DRY-RUN] " } else { "" };
                println!(
                    "{}Cleaned {} lines in {} file(s)",
                    prefix, lines, files
                );
            } else {
                println!("No files needed cleaning");
            }

            Ok(())
        }

        None => {
            // Legacy mode - direct flags without subcommand
            if let Some(path) = cli.path {
                // Check if user is trying to use convert flags
                let has_from = cli.from_camel
                    || cli.from_pascal
                    || cli.from_snake
                    || cli.from_screaming_snake
                    || cli.from_kebab
                    || cli.from_screaming_kebab;

                let has_to = cli.to_camel
                    || cli.to_pascal
                    || cli.to_snake
                    || cli.to_screaming_snake
                    || cli.to_kebab
                    || cli.to_screaming_kebab;

                if has_from && has_to {
                    run_convert(
                        cli.from_camel,
                        cli.from_pascal,
                        cli.from_snake,
                        cli.from_screaming_snake,
                        cli.from_kebab,
                        cli.from_screaming_kebab,
                        cli.to_camel,
                        cli.to_pascal,
                        cli.to_snake,
                        cli.to_screaming_snake,
                        cli.to_kebab,
                        cli.to_screaming_kebab,
                        path,
                        cli.recursive,
                        cli.dry_run,
                        cli.extensions,
                        cli.prefix,
                        cli.suffix,
                        cli.glob,
                        cli.word_filter,
                    )
                } else {
                    eprintln!("Error: Missing required arguments for case conversion");
                    eprintln!("Usage: codeconvert --from-<format> --to-<format> <PATH>");
                    eprintln!("   or: codeconvert clean <PATH>");
                    eprintln!("\nRun 'codeconvert --help' for more information");
                    std::process::exit(1);
                }
            } else {
                eprintln!("Error: No command or path specified");
                eprintln!("\nUsage:");
                eprintln!("  codeconvert convert --from-<format> --to-<format> <PATH>");
                eprintln!("  codeconvert clean <PATH>");
                eprintln!("\nRun 'codeconvert --help' for more information");
                std::process::exit(1);
            }
        }
    }
}
