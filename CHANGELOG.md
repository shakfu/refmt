# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-10-10

### Changed
- **BREAKING**: Restructured project as Cargo workspace
  - Core library: `codeconvert-core`
  - CLI binary: `codeconvert-cli`
  - Plugin system: `codeconvert-plugins`
- Library-first architecture enables programmatic usage
- CLI now supports subcommand architecture (`convert`, `clean`)
- Maintained full backwards compatibility for legacy CLI interface

### Added
- **Whitespace Cleaning Transformer**: New `clean` subcommand for removing trailing whitespace
  - Removes trailing whitespace from lines while preserving line endings
  - Supports dry-run mode (`--dry-run`)
  - Recursive processing (default: enabled)
  - Extension filtering with defaults for common code files
  - Skips hidden files and build directories
- **Subcommand Architecture**:
  - `codeconvert convert`: Case format conversion (new)
  - `codeconvert clean`: Whitespace cleaning (new)
  - Legacy direct flags still work for backwards compatibility
- Public library API for both case conversion and whitespace cleaning
- Modular workspace structure for easier feature additions
- Plugin system foundation
- Comprehensive module documentation

### Testing
- Added 6 unit tests for whitespace cleaning module
- Added 7 CLI integration tests for `clean` subcommand
- Added 1 CLI integration test for `convert` subcommand
- Total: 45 tests (18 core unit + 7 library integration + 20 CLI integration)

### Technical
- Split monolithic `src/main.rs` into organized modules
- Core logic in `codeconvert-core/src/{case.rs, converter.rs, whitespace.rs}`
- Whitespace cleaner preserves file line endings
- All tests passing with zero functional regressions

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
