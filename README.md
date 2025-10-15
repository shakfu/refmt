# refmt

A modular code transformation framework for applying code transformations to code in a set of source code files.

Organized as a Cargo workspace:
- **refmt-core**: Core transformation library
- **refmt-cli**: Command-line interface
- **refmt-plugins**: Plugin system (foundation)

## Features

### Case Format Conversion
- Convert between 6 case formats: camelCase, PascalCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case, and SCREAMING-KEBAB-CASE
- Process single files or entire directories (with recursive option)
- Dry-run mode to preview changes
- Filter files by glob patterns
- Filter which words to convert using regex patterns
- Add prefix/suffix to converted identifiers
- Support for multiple file extensions (.c, .h, .py, .md, .js, .ts, .java, .cpp, .hpp)

### Whitespace Cleaning
- Remove trailing whitespace from files
- Preserve line endings and file structure
- Recursive directory processing
- Extension filtering with sensible defaults
- Dry-run mode to preview changes
- Automatically skips hidden files and build directories

### Emoji Transformation
- Replace task completion emojis with text alternatives (✅ → [x], ☐ → [ ], etc.)
- Replace status indicator emojis (🟡 → [yellow], 🟢 → [green], 🔴 → [red])
- Remove non-task emojis from code and documentation
- Smart replacements for common task tracking symbols
- Configurable behavior (replace task emojis, remove others, or both)
- Support for markdown, documentation, and source files

### Logging & UI
- Multi-level verbosity control (`-v`, `-vv`, `-vvv`)
- Quiet mode for silent operation (`-q`)
- File logging for debugging (`--log-file`)
- Progress spinners with indicatif
- Automatic operation timing
- Color-coded console output

## Installation

Install from the workspace:

```bash
cargo install --path refmt-cli
```

Or build from source:

```bash
cargo build --release -p refmt
```

The binary will be at `./target/release/refmt`

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
refmt-core = { path = "../refmt-core" }
```

### Case Conversion

```rust
use refmt_core::{CaseConverter, CaseFormat};

let converter = CaseConverter::new(
    CaseFormat::CamelCase,
    CaseFormat::SnakeCase,
    None, false, false,
    String::new(), String::new(),
    None, None
)?;

converter.process_directory(std::path::Path::new("src"))?;
```

### Whitespace Cleaning

```rust
use refmt_core::{WhitespaceCleaner, WhitespaceOptions};

let mut options = WhitespaceOptions::default();
options.dry_run = false;
options.recursive = true;

let cleaner = WhitespaceCleaner::new(options);
let (files_cleaned, lines_cleaned) = cleaner.process(std::path::Path::new("src"))?;
println!("Cleaned {} lines in {} files", lines_cleaned, files_cleaned);
```

## Usage

### Case Conversion

Basic conversion (using subcommand):
```bash
refmt convert --from-camel --to-snake myfile.py
```

Or legacy mode (backwards compatible):
```bash
refmt --from-camel --to-snake myfile.py
```

Recursive directory conversion:
```bash
refmt convert --from-snake --to-camel -r src/
```

Dry run (preview changes):
```bash
refmt convert --from-camel --to-kebab --dry-run mydir/
```

Add prefix to all converted identifiers:
```bash
refmt convert --from-camel --to-snake --prefix "old_" myfile.py
```

Filter files by pattern:
```bash
refmt convert --from-camel --to-snake -r --glob "*test*.py" src/
```

Only convert specific identifiers:
```bash
refmt convert --from-camel --to-snake --word-filter "^get.*" src/
```

### Whitespace Cleaning

Clean all default file types in current directory:
```bash
refmt clean .
```

Clean with dry-run to preview changes:
```bash
refmt clean --dry-run src/
```

Clean only specific file types:
```bash
refmt clean -e .py -e .rs src/
```

Clean a single file:
```bash
refmt clean myfile.py
```

### Emoji Transformation

Replace task emojis with text in markdown files:
```bash
refmt emojis docs/
```

Process with dry-run to preview changes:
```bash
refmt emojis --dry-run README.md
```

Only replace task emojis, keep other emojis:
```bash
refmt emojis --replace-task --no-remove-other docs/
```

Process specific file types:
```bash
refmt emojis -e .md -e .txt project/
```

### Logging and Debugging

Control output verbosity:
```bash
# Info level output (-v)
refmt -v convert --from-camel --to-snake src/

# Debug level output (-vv)
refmt -vv clean src/

# Silent mode (errors only)
refmt -q convert --from-camel --to-snake src/

# Log to file
refmt --log-file debug.log -v convert --from-camel --to-snake src/
```

Output example with `-v`:
```
2025-10-10T00:15:08.927Z [INFO] Converting from CamelCase to SnakeCase
2025-10-10T00:15:08.927Z [INFO] Target path: /tmp/test.py
2025-10-10T00:15:08.927Z [INFO] Recursive: false, Dry run: false
Converted '/tmp/test.py'
2025-10-10T00:15:08.931Z [INFO] Conversion completed successfully
2025-10-10T00:15:08.931Z [INFO] run_convert(), Elapsed=4.089125ms
```

## Case Format Options

- `--from-camel` / `--to-camel` - camelCase (firstName, lastName)
- `--from-pascal` / `--to-pascal` - PascalCase (FirstName, LastName)
- `--from-snake` / `--to-snake` - snake_case (first_name, last_name)
- `--from-screaming-snake` / `--to-screaming-snake` - SCREAMING_SNAKE_CASE (FIRST_NAME, LAST_NAME)
- `--from-kebab` / `--to-kebab` - kebab-case (first-name, last-name)
- `--from-screaming-kebab` / `--to-screaming-kebab` - SCREAMING-KEBAB-CASE (FIRST-NAME, LAST-NAME)

## Examples

### Case Conversion Examples

Convert Python file from camelCase to snake_case:
```bash
refmt convert --from-camel --to-snake main.py
```

Convert C++ project from snake_case to PascalCase:
```bash
refmt convert --from-snake --to-pascal -r -e .cpp -e .hpp src/
```

Preview converting JavaScript getters to snake_case:
```bash
refmt convert --from-camel --to-snake --word-filter "^get.*" -d src/
```

### Whitespace Cleaning Examples

Clean trailing whitespace from entire project:
```bash
refmt clean -r .
```

Clean only Python files in src directory:
```bash
refmt clean -e .py src/
```

Preview what would be cleaned without making changes:
```bash
refmt clean --dry-run .
```

### Emoji Transformation Examples

Transform task emojis in documentation:
```bash
refmt emojis -r docs/
```

Example transformation:
```markdown
Before:
- Task done ✅
- Task pending ☐
- Warning ⚠ issue
- 🟡 In progress
- 🟢 Complete
- 🔴 Blocked

After:
- Task done [x]
- Task pending [ ]
- Warning [!] issue
- [yellow] In progress
- [green] Complete
- [red] Blocked
```

Process only markdown files:
```bash
refmt emojis -e .md README.md
```

## License

See LICENSE file for details.
