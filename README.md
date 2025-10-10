# codeconvert

A modular code transformation framework for applying code transformations to code in a set of source code files.

Organized as a Cargo workspace:
- **codeconvert-core**: Core transformation library
- **codeconvert-cli**: Command-line interface
- **codeconvert-plugins**: Plugin system (foundation)

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
cargo install --path codeconvert-cli
```

Or build from source:

```bash
cargo build --release -p codeconvert
```

The binary will be at `./target/release/codeconvert`

## Library Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
codeconvert-core = { path = "../codeconvert-core" }
```

### Case Conversion

```rust
use codeconvert_core::{CaseConverter, CaseFormat};

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
use codeconvert_core::{WhitespaceCleaner, WhitespaceOptions};

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
codeconvert convert --from-camel --to-snake myfile.py
```

Or legacy mode (backwards compatible):
```bash
codeconvert --from-camel --to-snake myfile.py
```

Recursive directory conversion:
```bash
codeconvert convert --from-snake --to-camel -r src/
```

Dry run (preview changes):
```bash
codeconvert convert --from-camel --to-kebab --dry-run mydir/
```

Add prefix to all converted identifiers:
```bash
codeconvert convert --from-camel --to-snake --prefix "old_" myfile.py
```

Filter files by pattern:
```bash
codeconvert convert --from-camel --to-snake -r --glob "*test*.py" src/
```

Only convert specific identifiers:
```bash
codeconvert convert --from-camel --to-snake --word-filter "^get.*" src/
```

### Whitespace Cleaning

Clean all default file types in current directory:
```bash
codeconvert clean .
```

Clean with dry-run to preview changes:
```bash
codeconvert clean --dry-run src/
```

Clean only specific file types:
```bash
codeconvert clean -e .py -e .rs src/
```

Clean a single file:
```bash
codeconvert clean myfile.py
```

### Emoji Transformation

Replace task emojis with text in markdown files:
```bash
codeconvert emojis docs/
```

Process with dry-run to preview changes:
```bash
codeconvert emojis --dry-run README.md
```

Only replace task emojis, keep other emojis:
```bash
codeconvert emojis --replace-task --no-remove-other docs/
```

Process specific file types:
```bash
codeconvert emojis -e .md -e .txt project/
```

### Logging and Debugging

Control output verbosity:
```bash
# Info level output (-v)
codeconvert -v convert --from-camel --to-snake src/

# Debug level output (-vv)
codeconvert -vv clean src/

# Silent mode (errors only)
codeconvert -q convert --from-camel --to-snake src/

# Log to file
codeconvert --log-file debug.log -v convert --from-camel --to-snake src/
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
codeconvert convert --from-camel --to-snake main.py
```

Convert C++ project from snake_case to PascalCase:
```bash
codeconvert convert --from-snake --to-pascal -r -e .cpp -e .hpp src/
```

Preview converting JavaScript getters to snake_case:
```bash
codeconvert convert --from-camel --to-snake --word-filter "^get.*" -d src/
```

### Whitespace Cleaning Examples

Clean trailing whitespace from entire project:
```bash
codeconvert clean -r .
```

Clean only Python files in src directory:
```bash
codeconvert clean -e .py src/
```

Preview what would be cleaned without making changes:
```bash
codeconvert clean --dry-run .
```

### Emoji Transformation Examples

Transform task emojis in documentation:
```bash
codeconvert emojis -r docs/
```

Example transformation:
```markdown
Before:
- Task done ✅
- Task pending ☐
- Warning ⚠ issue

After:
- Task done [x]
- Task pending [ ]
- Warning [!] issue
```

Process only markdown files:
```bash
codeconvert emojis -e .md README.md
```

## License

See LICENSE file for details.
