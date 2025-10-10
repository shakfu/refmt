# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-10

### Overview
This release represents a major architectural overhaul and feature expansion. The project has been restructured as a Cargo workspace with a library-first design, enabling both CLI and programmatic usage. Three new subcommands have been added (`convert`, `clean`, `emojis`), along with comprehensive logging and UI enhancements.

### Changed
- **BREAKING**: Restructured project as Cargo workspace
  - **codeconvert-core**: Core library for transformations
  - **codeconvert-cli**: Command-line binary
  - **codeconvert-plugins**: Plugin system foundation
- Library-first architecture enables programmatic usage
- CLI now supports modern subcommand architecture with three commands:
  - `codeconvert convert`: Case format conversion
  - `codeconvert clean`: Whitespace cleaning
  - `codeconvert emojis`: Emoji transformation
- Enhanced CLI with comprehensive logging and UI features
- Maintained full backwards compatibility for legacy CLI interface (direct flags still work)

### Added

#### New Transformers

**Whitespace Cleaning Transformer** (`clean` subcommand)
- Removes trailing whitespace from lines while preserving line endings
- Supports dry-run mode (`--dry-run`) for previewing changes
- Recursive processing (default: enabled, `-r` flag)
- Extension filtering with sensible defaults for common code files
- Automatically skips hidden files and build directories (`.git`, `node_modules`, `target`, etc.)
- Example: `codeconvert clean src/`

**Emoji Transformation** (`emojis` subcommand)
- Replaces task completion emojis with text alternatives for better compatibility
- **Smart emoji mappings**:
  - ✅ → `[x]` (white check mark)
  - ☐ → `[ ]` (ballot box)
  - ☑ → `[x]` (ballot box with check)
  - ✓ → `[x]` (check mark)
  - ✔ → `[x]` (heavy check mark)
  - ☒ → `[X]` (ballot box with X)
  - ❌ → `[X]` (cross mark)
  - ❎ → `[X]` (negative squared cross mark)
  - ⚠ → `[!]` (warning sign)
  - 📝 → `[note]` (memo)
  - 📋 → `[list]` (clipboard)
  - 📌 → `[pin]` (pushpin)
  - 📎 → `[clip]` (paperclip)
- Removes non-task emojis from documentation and code
- Configurable behavior:
  - `--replace-task`: Replace task emojis with text (default: true)
  - `--remove-other`: Remove non-task emojis (default: true)
- Support for markdown, text, and source code files
- Example: `codeconvert emojis README.md`

#### Logging & UI Enhancements

- **Multi-level verbosity control**:
  - Default: WARN level (minimal output)
  - `-v`: INFO level (shows progress and completion)
  - `-vv`: DEBUG level (detailed operation information)
  - `-vvv`: TRACE level (maximum verbosity)
- **Quiet mode** (`-q`): Suppresses all output except errors
- **File logging** (`--log-file <PATH>`): Write debug logs to file for troubleshooting
- **Progress indicators**: Animated spinners during file processing using `indicatif`
- **Automatic timing**: Operations log execution time at INFO level
  - Example output: `run_convert(), Elapsed=4.089125ms`
- **Color-coded output**: Structured, timestamped logs with `simplelog`
- **Global flags**: `-v`, `-q`, and `--log-file` work with all subcommands

#### Library Features

- **Public API** exports for all transformers:
  - `CaseConverter` and `CaseFormat` for case conversion
  - `WhitespaceCleaner` and `WhitespaceOptions` for whitespace cleaning
  - `EmojiTransformer` and `EmojiOptions` for emoji transformation
- Modular workspace structure for easier feature additions
- Plugin system foundation in `codeconvert-plugins`
- Comprehensive inline documentation and module docs
- Example library usage in integration tests

### Testing

**Comprehensive Test Coverage**:
- **Unit tests** (24 total):
  - 12 tests for case conversion module (`case.rs`, `converter.rs`)
  - 6 tests for whitespace cleaning module
  - 6 tests for emoji transformation module
- **Library integration tests** (7 total):
  - Tests for programmatic API usage
  - Validation of library behavior
- **CLI integration tests** (20 total):
  - 13 tests for case conversion CLI
  - 7 tests for whitespace cleaning CLI
  - Tests cover: version, help, basic operations, dry-run, recursive processing, error handling
- **Total: 51 tests** - all passing with zero functional regressions

**Test Features**:
- Isolated test environments using temp directories
- Tests for dry-run modes across all transformers
- Extension filtering validation
- Hidden file and build directory skipping
- Pattern matching and glob filtering
- All edge cases covered

### Technical Details

**Architecture**:
- Split monolithic `src/main.rs` (437 lines) into organized modules across 3 crates
- **Core modules**:
  - `codeconvert-core/src/case.rs` - Case format definitions and conversion logic
  - `codeconvert-core/src/converter.rs` - File processing and pattern matching
  - `codeconvert-core/src/whitespace.rs` - Trailing whitespace removal
  - `codeconvert-core/src/emoji.rs` - Emoji detection and replacement
  - `codeconvert-core/src/lib.rs` - Public API exports
- **CLI module**:
  - `codeconvert-cli/src/main.rs` - Clap-based CLI with subcommands and logging

**Implementation Highlights**:
- Whitespace cleaner preserves file line endings (CRLF/LF)
- Emoji transformer uses Unicode regex patterns for comprehensive detection
- Smart emoji replacement mappings maintain markdown compatibility
- Manual character iteration for camelCase/PascalCase splitting (Rust regex limitation)
- Regex-based pattern matching for case format identification
- Glob matching supports both filename and relative path patterns

**Dependencies Added**:
- `log` (0.4) - Logging facade
- `simplelog` (0.12) - Logging implementation with color support
- `indicatif` (0.17) - Progress bars and spinners
- `logging_timer` (1.1) - Automatic function timing

**Performance**:
- All transformations complete in milliseconds for typical projects
- Example timing: `run_convert(), Elapsed=4.089125ms`
- Efficient regex-based pattern matching
- Minimal memory overhead with streaming file processing

## [0.1.0] - 2025-10-10

### Added
- Initial Rust implementation of codeconvert CLI tool with Python-compatible API
- Support for 6 case format conversions:
  - camelCase
  - PascalCase
  - snake_case
  - SCREAMING_SNAKE_CASE
  - kebab-case
  - SCREAMING-KEBAB-CASE
- Core conversion features:
  - Single file and directory processing
  - Recursive directory traversal (`-r, --recursive`)
  - Dry-run mode for previewing changes (`-d, --dry-run`)
  - Custom file extension filtering (`-e, --extensions`)
  - Glob pattern filtering for file selection (`--glob`)
  - Regex pattern filtering for selective word conversion (`--word-filter`)
  - Prefix and suffix support for converted identifiers (`--prefix`, `--suffix`)
- Default support for common file extensions: `.c`, `.h`, `.py`, `.md`, `.js`, `.ts`, `.java`, `.cpp`, `.hpp`
- Comprehensive unit test suite (8 tests) covering:
  - Bidirectional conversions between formats
  - Pattern matching accuracy
  - Prefix/suffix functionality
- CLI built with clap v4.5 using derive macros, matching Python argparse API:
  - `--from-camel`, `--from-pascal`, `--from-snake`, etc.
  - `--to-camel`, `--to-pascal`, `--to-snake`, etc.
- Project documentation:
  - README.md with usage examples
  - CLAUDE.md with architecture details
  - Inline code documentation

### Technical Details
- Manual character-by-character word splitting for camelCase/PascalCase (Rust regex doesn't support lookahead/lookbehind)
- Regex-based pattern matching for identifying case formats
- Glob matching supports both filename and relative path patterns
- Error handling with user-friendly messages

### Legacy
- Python implementation (case_converter.py) remains available for compatibility

[0.1.0]: https://github.com/yourusername/code-convert/releases/tag/v0.1.0
