# Code Transformation Framework Architecture

## Overview

This document describes the architecture for a modular, composable code transformation framework. The framework supports chained transformations applied to file sets discovered through pattern matching and refined through filtering pipelines.

## Design Principles

1. **Modularity**: Each component (search, filter, transform, analyze) is independent and composable
2. **Composability**: Pipeline builder pattern allows fluent API construction
3. **Extensibility**: Plugin system enables custom transformations without modifying core
4. **Type Safety**: Strong typing with traits ensures compile-time guarantees
5. **Performance**: Parallel execution support with optional AST caching
6. **Observability**: Rich diagnostics, dry-run mode, and rollback capability
7. **Library-First**: Core functionality in library, CLI as thin wrapper
8. **Feature Isolation**: New features added as independent modules without touching core

## Project Structure

### Workspace Organization

The project is organized as a Cargo workspace with clear separation between library and binary:

```
codeconvert/
├── Cargo.toml                 # Workspace definition
├── codeconvert-core/          # Core library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # Library entry point
│       ├── context.rs         # TransformContext, FileEntry
│       ├── pipeline.rs        # Pipeline, PipelineBuilder
│       ├── stage.rs           # Stage enum, execution logic
│       ├── error.rs           # Error types
│       ├── traits/            # Core trait definitions
│       │   ├── mod.rs
│       │   ├── search.rs      # SearchStrategy trait
│       │   ├── filter.rs      # Filter trait
│       │   ├── transform.rs   # Transformer trait
│       │   ├── analyze.rs     # Analyzer trait
│       │   └── pattern.rs     # PatternMatcher trait
│       ├── search/            # Search implementations
│       │   ├── mod.rs
│       │   ├── glob.rs        # GlobSearch
│       │   ├── pattern.rs     # PatternSearch
│       │   ├── git.rs         # GitSearch
│       │   └── dependency.rs  # DependencySearch
│       ├── filters/           # Filter implementations
│       │   ├── mod.rs
│       │   ├── extension.rs   # ExtensionFilter
│       │   ├── content.rs     # ContentFilter
│       │   ├── path.rs        # PathFilter
│       │   ├── size.rs        # SizeFilter
│       │   ├── git.rs         # GitFilter
│       │   ├── semantic.rs    # SemanticFilter
│       │   └── dependency.rs  # DependencyFilter
│       ├── transformers/      # Core transformers
│       │   ├── mod.rs
│       │   ├── case.rs        # CaseTransformer
│       │   ├── quotes.rs      # QuoteTransformer
│       │   ├── prefix.rs      # PrefixTransformer
│       │   ├── whitespace.rs  # WhitespaceTransformer
│       │   └── imports.rs     # ImportTransformer
│       ├── analyzers/         # Analyzer implementations
│       │   ├── mod.rs
│       │   ├── complexity.rs  # ComplexityAnalyzer
│       │   ├── metrics.rs     # MetricsAnalyzer
│       │   └── coverage.rs    # CoverageAnalyzer
│       ├── patterns/          # Pattern matching
│       │   ├── mod.rs
│       │   ├── regex.rs       # RegexMatcher
│       │   ├── ast.rs         # AstMatcher (Tree-sitter)
│       │   └── structural.rs  # StructuralMatcher
│       ├── config/            # Configuration loading
│       │   ├── mod.rs
│       │   ├── yaml.rs        # YAML config parser
│       │   └── builder.rs     # Config to Pipeline conversion
│       └── utils/             # Shared utilities
│           ├── mod.rs
│           ├── transaction.rs # TransactionManager
│           └── parallel.rs    # Parallel execution helpers
│
├── codeconvert-plugins/       # Plugin system (separate crate)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── api.rs             # Plugin API definitions
│       ├── loader.rs          # Dynamic plugin loading
│       └── registry.rs        # Plugin registry
│
├── codeconvert-transformers/  # Extended transformers (optional features)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── structural.rs      # StructuralTransformer
│       ├── api_migration.rs   # ApiMigrationTransformer
│       ├── type_hints.rs      # TypeTransformer
│       └── literals.rs        # LiteralTransformer
│
├── codeconvert-cli/           # CLI binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Entry point
│       ├── cli.rs             # Clap argument parsing
│       ├── commands/          # CLI commands
│       │   ├── mod.rs
│       │   ├── transform.rs   # transform command
│       │   ├── pipeline.rs    # pipeline command
│       │   ├── interactive.rs # interactive mode
│       │   ├── plugin.rs      # plugin management
│       │   └── list.rs        # list transformers/filters
│       ├── output/            # Output formatting
│       │   ├── mod.rs
│       │   ├── formatter.rs   # Output formatter
│       │   └── reporter.rs    # Execution reporter
│       └── compat/            # Backwards compatibility
│           ├── mod.rs
│           └── legacy.rs      # Legacy CLI adapter
│
├── examples/                  # Library usage examples
│   ├── simple_pipeline.rs
│   ├── custom_transformer.rs
│   ├── config_based.rs
│   └── plugin_example.rs
│
├── plugins/                   # Example plugins
│   ├── api-migration/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── custom-transform/
│       ├── Cargo.toml
│       └── src/lib.rs
│
└── tests/                     # Integration tests
    ├── pipelines/
    ├── transformers/
    └── fixtures/
```

### Workspace Cargo.toml

```toml
[workspace]
members = [
    "codeconvert-core",
    "codeconvert-plugins",
    "codeconvert-transformers",
    "codeconvert-cli",
]

[workspace.package]
version = "0.2.0"
edition = "2021"
authors = ["CodeConvert Contributors"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Shared dependencies across workspace
regex = "1.11"
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
anyhow = "1.0"
thiserror = "1.0"
```

### Core Library (codeconvert-core)

```toml
# codeconvert-core/Cargo.toml
[package]
name = "codeconvert-core"
version.workspace = true
edition.workspace = true

[dependencies]
regex.workspace = true
serde.workspace = true
anyhow.workspace = true
thiserror.workspace = true

# Optional dependencies for features
tree-sitter = { version = "0.20", optional = true }
walkdir = "2.5"
glob = "0.3"
rayon = { version = "1.8", optional = true }

[features]
default = ["parallel"]
parallel = ["rayon"]
ast = ["tree-sitter"]
full = ["parallel", "ast"]
```

### CLI Binary (codeconvert-cli)

```toml
# codeconvert-cli/Cargo.toml
[package]
name = "codeconvert"
version.workspace = true
edition.workspace = true

[[bin]]
name = "codeconvert"
path = "src/main.rs"

[dependencies]
codeconvert-core = { path = "../codeconvert-core", features = ["full"] }
codeconvert-plugins = { path = "../codeconvert-plugins" }
codeconvert-transformers = { path = "../codeconvert-transformers", optional = true }

clap = { version = "4.5", features = ["derive"] }
serde_yaml.workspace = true
anyhow.workspace = true

[features]
default = ["extended"]
extended = ["codeconvert-transformers"]
```

### Module Responsibilities

#### Core Library Modules

| Module | Responsibility | Exports |
|--------|---------------|---------|
| `context` | Data structures for pipeline execution | `TransformContext`, `FileEntry`, `FileSet` |
| `pipeline` | Pipeline construction and execution | `Pipeline`, `PipelineBuilder` |
| `stage` | Stage definitions and orchestration | `Stage` enum, execution logic |
| `traits/*` | Core trait definitions | All core traits |
| `search/*` | File discovery implementations | Concrete search strategies |
| `filters/*` | File set refinement | Concrete filters |
| `transformers/*` | Core transformations | Basic transformers |
| `analyzers/*` | Code analysis | Concrete analyzers |
| `patterns/*` | Pattern matching engines | Pattern matchers |
| `config/*` | Configuration loading | Config parsers, builders |
| `utils/*` | Shared utilities | Helper functions |

#### CLI Binary Modules

| Module | Responsibility | Purpose |
|--------|---------------|---------|
| `cli` | Argument parsing | Clap-based CLI definition |
| `commands/*` | Command implementations | Individual command logic |
| `output/*` | Result formatting | Pretty printing, reporting |
| `compat/legacy` | Backwards compatibility | Support old CLI syntax |

### Adding New Features

#### Adding a New Transformer

1. **Create transformer module** in `codeconvert-core/src/transformers/`:

```rust
// codeconvert-core/src/transformers/my_feature.rs
use crate::traits::Transformer;
use crate::context::TransformContext;
use anyhow::Result;

pub struct MyFeatureTransformer {
    config: MyFeatureConfig,
}

impl Transformer for MyFeatureTransformer {
    fn transform(&self, context: &mut TransformContext) -> Result<()> {
        // Implementation
        Ok(())
    }

    fn name(&self) -> &str {
        "my_feature"
    }
}
```

2. **Export from module** in `codeconvert-core/src/transformers/mod.rs`:

```rust
pub mod my_feature;
pub use my_feature::MyFeatureTransformer;
```

3. **Add builder method** in `codeconvert-core/src/pipeline.rs`:

```rust
impl PipelineBuilder {
    pub fn transform_my_feature(mut self, config: MyFeatureConfig) -> Self {
        self.stages.push(Stage::Transform(
            Box::new(MyFeatureTransformer::new(config))
        ));
        self
    }
}
```

4. **Add CLI command** in `codeconvert-cli/src/cli.rs`:

```rust
#[derive(Args)]
struct MyFeatureArgs {
    // Configuration
}
```

No core files need modification - the feature is completely isolated!

#### Adding a New Filter

Similar process:

1. Create `codeconvert-core/src/filters/my_filter.rs`
2. Implement `Filter` trait
3. Export from `filters/mod.rs`
4. Add `PipelineBuilder::filter_my_filter()` method
5. Add CLI flag if needed

#### Adding a Complex Feature (as separate crate)

For large features, create a new workspace member:

```bash
cargo new --lib codeconvert-feature-x
```

```toml
# codeconvert-feature-x/Cargo.toml
[package]
name = "codeconvert-feature-x"

[dependencies]
codeconvert-core = { path = "../codeconvert-core" }

# Feature-specific deps
```

```rust
// codeconvert-feature-x/src/lib.rs
use codeconvert_core::traits::Transformer;

pub struct FeatureXTransformer {
    // ...
}

impl Transformer for FeatureXTransformer {
    // Implementation
}
```

Then optionally include in CLI:

```toml
# codeconvert-cli/Cargo.toml
[dependencies]
codeconvert-feature-x = { path = "../codeconvert-feature-x", optional = true }

[features]
feature-x = ["codeconvert-feature-x"]
```

### Library Usage Patterns

#### As a Library Dependency

Users can depend on just the core library:

```toml
# User's Cargo.toml
[dependencies]
codeconvert-core = "0.2"
```

```rust
// User's code
use codeconvert_core::{Pipeline, CaseFormat};

fn main() {
    let pipeline = Pipeline::builder()
        .search_glob("**/*.rs")
        .transform_case(CaseFormat::SnakeCase, CaseFormat::CamelCase)
        .build();

    pipeline.execute(".").unwrap();
}
```

#### Extending with Custom Transformers

```rust
use codeconvert_core::traits::Transformer;
use codeconvert_core::TransformContext;

struct MyCustomTransformer;

impl Transformer for MyCustomTransformer {
    fn transform(&self, context: &mut TransformContext) -> anyhow::Result<()> {
        // Custom logic
        Ok(())
    }

    fn name(&self) -> &str {
        "custom"
    }
}

// Use in pipeline
let pipeline = Pipeline::builder()
    .search_glob("**/*.rs")
    .transform_custom(Box::new(MyCustomTransformer))
    .build();
```

### Testing Structure

```
tests/
├── integration/           # Integration tests
│   ├── pipeline_test.rs
│   ├── transformers_test.rs
│   └── filters_test.rs
│
├── fixtures/             # Test data
│   ├── sample_code/
│   ├── configs/
│   └── expected/
│
└── plugins/              # Plugin tests
    └── loading_test.rs
```

### Documentation Structure

```
docs/
├── user-guide/
│   ├── getting-started.md
│   ├── transformers.md
│   ├── filters.md
│   └── pipelines.md
│
├── developer-guide/
│   ├── adding-transformers.md
│   ├── plugin-development.md
│   └── architecture.md
│
└── api/                  # Generated rustdoc
```

### Build and Release

```toml
# Root Cargo.toml for releases
[profile.release]
opt-level = 3
lto = true
codegen-units = 1

[package.metadata.release]
# Release configuration
```

Build targets:
- `cargo build -p codeconvert-core` - Library only
- `cargo build -p codeconvert` - CLI with default features
- `cargo build -p codeconvert --all-features` - Full CLI
- `cargo build --workspace` - Everything

### Feature Flags Organization

```toml
[features]
# Core features
default = ["parallel"]
parallel = ["rayon"]
ast = ["tree-sitter"]

# Transformer groups
basic-transforms = []  # Case, quotes, prefix (always included)
structural-transforms = ["codeconvert-transformers/structural"]
language-transforms = ["codeconvert-transformers/language-specific"]

# Full feature set
full = ["parallel", "ast", "structural-transforms", "language-transforms"]
```

### Deployment Artifacts

The structure supports multiple deployment scenarios:

1. **Library crate**: `codeconvert-core` published to crates.io
2. **CLI binary**: `codeconvert` published to crates.io with `cargo install`
3. **Plugins**: Published separately, loaded dynamically
4. **Docker image**: Multi-stage build using workspace
5. **WASM**: Core library compiled to WebAssembly

This organization ensures:
- **Library users** get minimal dependencies (just `codeconvert-core`)
- **CLI users** get full featured binary
- **Developers** can add features without touching core
- **Plugins** are completely independent
- **Testing** is organized and comprehensive

## Core Abstractions

### 1. Transform Context

The central data structure flowing through the pipeline:

```rust
pub struct TransformContext {
    files: FileSet,                          // Set of files being processed
    metadata: HashMap<String, Value>,        // Pipeline-level metadata
    diagnostics: Vec<Diagnostic>,            // Transformation history
}

pub struct FileEntry {
    path: PathBuf,                           // File location
    content: String,                         // File contents
    ast: Option<SyntaxTree>,                 // Lazy-parsed AST
    metadata: FileMetadata,                  // File-level metadata
}
```

### 2. Pipeline Stages

Four distinct stages in the transformation pipeline:

```rust
pub enum Stage {
    Search(Box<dyn SearchStrategy>),         // Discovery: find files
    Filter(Box<dyn Filter>),                 // Refinement: narrow file set
    Transform(Box<dyn Transformer>),         // Modification: apply changes
    Analyze(Box<dyn Analyzer>),              // Inspection: gather metrics
}
```

### 3. Core Traits

```rust
/// File discovery strategies
pub trait SearchStrategy: Send + Sync {
    fn search(&self, root: &Path) -> Result<FileSet>;
}

/// File set refinement
pub trait Filter: Send + Sync {
    fn filter(&self, context: &mut TransformContext) -> Result<()>;
    fn name(&self) -> &str;
}

/// Content transformation
pub trait Transformer: Send + Sync {
    fn transform(&self, context: &mut TransformContext) -> Result<()>;
    fn name(&self) -> &str;
    fn dry_run(&self) -> bool { false }
}

/// Information extraction
pub trait Analyzer: Send + Sync {
    fn analyze(&self, context: &TransformContext) -> Result<AnalysisResult>;
}

/// Pattern matching abstraction
pub trait PatternMatcher: Send + Sync {
    fn matches(&self, content: &str) -> Vec<Match>;
    fn pattern(&self) -> &str;
}
```

## Pipeline Architecture

### Execution Flow

```
┌─────────────────────────────────────────────────────────┐
│                    Pipeline Executor                    │
└─────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────┐
│  Phase 1: SEARCH - Build Initial File Set               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Glob Search  │  │Pattern Search│  │ Git Search   │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│         │                  │                  │         │
│         └──────────────────┴──────────────────┘         │
│                            │                            │
│                    ┌───────▼────────┐                   │
│                    │    FileSet     │                   │
│                    └───────┬────────┘                   │
└────────────────────────────┼────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│  Phase 2: FILTER - Refine File Set                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │Extension Filt│  │Content Filter│  │ Path Filter  │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │ Size Filter  │  │  Git Filter  │  │Semantic Filt.│   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                            │                            │
│                    ┌───────▼────────┐                   │
│                    │Filtered FileSet│                   │
│                    └───────┬────────┘                   │
└────────────────────────────┼────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│  Phase 3: TRANSFORM - Apply Modifications               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │Case Transform│  │Quote Transform│ │Prefix Trans. │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │Import Trans. │  │Structural T. │  │Custom Plugin │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                            │                            │
│                    ┌───────▼────────┐                   │
│                    │Modified FileSet│                   │
│                    └───────┬────────┘                   │
└────────────────────────────┼────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────┐
│  Phase 4: ANALYZE - Gather Metrics                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │Complexity An.│  │Coverage Analy│  │Metrics Analy.│   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                            │                            │
│                    ┌───────▼────────┐                   │
│                    │Analysis Report │                   │
│                    └────────────────┘                   │
└─────────────────────────────────────────────────────────┘
```

### Pipeline Builder (Fluent API)

```rust
impl PipelineBuilder {
    // Search stage
    pub fn search_glob(self, pattern: &str) -> Self;
    pub fn search_pattern(self, pattern: &str, matcher: PatternType) -> Self;
    pub fn search_git(self, status: GitStatus) -> Self;

    // Filter stage
    pub fn filter_extension(self, exts: Vec<&str>) -> Self;
    pub fn filter_content(self, pattern: &str) -> Self;
    pub fn filter_path(self, include: Vec<&str>, exclude: Vec<&str>) -> Self;
    pub fn filter_size(self, min: Option<usize>, max: Option<usize>) -> Self;
    pub fn filter_semantic(self, criteria: SemanticCriteria) -> Self;

    // Transform stage
    pub fn transform_case(self, from: CaseFormat, to: CaseFormat) -> Self;
    pub fn transform_quotes(self, from: QuoteStyle, to: QuoteStyle) -> Self;
    pub fn transform_prefix(self, action: PrefixAction) -> Self;
    pub fn transform_imports(self, action: ImportAction) -> Self;
    pub fn transform_structural(self, pattern: StructuralPattern) -> Self;
    pub fn transform_custom(self, transformer: Box<dyn Transformer>) -> Self;

    // Analysis stage
    pub fn analyze_complexity(self) -> Self;
    pub fn analyze_coverage(self) -> Self;

    // Configuration
    pub fn dry_run(self, enabled: bool) -> Self;
    pub fn parallel(self, enabled: bool) -> Self;
    pub fn workers(self, count: usize) -> Self;

    pub fn build(self) -> Pipeline;
}
```

## Component Library

### Search Strategies

| Component | Description | Example |
|-----------|-------------|---------|
| `GlobSearch` | Pattern-based file discovery | `**/*.{js,ts}` |
| `PatternSearch` | Content-based file discovery | Files containing regex pattern |
| `GitSearch` | Git status-based discovery | Modified/staged files |
| `DependencySearch` | Import graph traversal | Files importing X |

### Filters

| Component | Description | Configuration |
|-----------|-------------|---------------|
| `ExtensionFilter` | File extension matching | Include/exclude lists |
| `ContentFilter` | Regex content matching | Pattern + invert flag |
| `PathFilter` | Path glob matching | Include/exclude patterns |
| `SizeFilter` | File size constraints | Min/max bytes |
| `GitFilter` | Git status filtering | Modified, staged, untracked |
| `SemanticFilter` | AST-based filtering | Has function/class/import |
| `DependencyFilter` | Import relationship | Uses/UsedBy |
| `DateFilter` | Modification date | Before/after/between |

### Transformers

| Component | Description | Parameters |
|-----------|-------------|------------|
| `CaseTransformer` | Identifier case conversion | from, to, scope, filter |
| `QuoteTransformer` | Quote style conversion | from, to, exclude_imports |
| `PrefixTransformer` | Prefix/suffix operations | add, remove, replace |
| `ImportTransformer` | Import management | sort, merge, update, remove |
| `WhitespaceTransformer` | Whitespace normalization | tabs/spaces, line endings |
| `CommentTransformer` | Comment style conversion | inline/block, doc formats |
| `StructuralTransformer` | Code structure changes | function→arrow, class→functional |
| `ApiMigrationTransformer` | API rename/update | Old→new mappings |
| `TypeTransformer` | Type annotation changes | Add/remove/update types |
| `LiteralTransformer` | String/number formatting | f-strings, hex/decimal |

### Analyzers

| Component | Description | Output |
|-----------|-------------|--------|
| `ComplexityAnalyzer` | Cyclomatic complexity | Per-function metrics |
| `CoverageAnalyzer` | Pattern coverage | Match statistics |
| `MetricsAnalyzer` | Code metrics | LOC, comments, ratio |
| `DependencyAnalyzer` | Import graph | Dependency tree |
| `DuplicationAnalyzer` | Code duplication | Clone detection |

## Pattern Matching System

### Matcher Types

```rust
pub enum PatternMatcher {
    Regex(RegexMatcher),           // Regex patterns
    Ast(AstMatcher),               // Tree-sitter queries
    Structural(StructuralMatcher), // Structural patterns
    Semantic(SemanticMatcher),     // Semantic understanding
}
```

### AST-Based Matching

Uses Tree-sitter for language-aware matching:

```rust
pub struct AstMatcher {
    query: TreeSitterQuery,
    language: Language,
}

// Example: Match all function declarations
let matcher = AstMatcher::new(
    "(function_declaration name: (identifier) @func.name)",
    Language::JavaScript
);
```

### Structural Patterns

Template-based code patterns with placeholders:

```rust
// Match getter/setter pairs
let pattern = StructuralPattern::parse(
    "get $name() { return this._$name; }"
);

// Match callback patterns
let pattern = StructuralPattern::parse(
    "$obj.$method(function($args) { $body })"
);
```

## Configuration System

### YAML Configuration

```yaml
name: "Migration Pipeline"
version: "1.0"

pipeline:
  search:
    - type: glob
      pattern: "src/**/*.{js,ts}"
    - type: pattern
      pattern: "var\\s+\\w+"
      matcher: regex

  filters:
    - type: extension
      include: [".js", ".ts"]
      exclude: [".min.js", ".d.ts"]

    - type: content
      pattern: "^(?!.*test)"
      invert: true

    - type: path
      include: ["src/**"]
      exclude: ["src/vendor/**"]

    - type: semantic
      criteria:
        has_function: "deprecated*"

  transforms:
    - type: case
      from: snake_case
      to: camelCase
      scope: variables

    - type: quotes
      from: single
      to: double

    - type: prefix
      action: strip
      patterns: ["m_", "_", "str"]

    - type: import
      action: sort
      groups: [stdlib, external, local]

    - type: custom
      plugin: "./plugins/api_migration.so"
      config:
        mappings:
          "oldApi.method": "newApi.method"

  analyze:
    - type: complexity
      threshold: 10

    - type: metrics
      report: detailed

  config:
    dry_run: false
    parallel: true
    workers: 4
    backup: true
```

### Programmatic Configuration

```rust
let pipeline = Pipeline::builder()
    .search_glob("src/**/*.js")
    .filter_content(r"var\s+\w+")
    .filter_extension(vec![".js"])
    .transform_case(CaseFormat::SnakeCase, CaseFormat::CamelCase)
    .transform_quotes(QuoteStyle::Single, QuoteStyle::Double)
    .analyze_complexity()
    .parallel(true)
    .build();
```

## Plugin System

### Plugin Interface

```rust
pub trait TransformerPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn create(&self, config: Value) -> Box<dyn Transformer>;
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn TransformerPlugin> {
    Box::new(MyCustomPlugin::new())
}
```

### Plugin Loading

```rust
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn TransformerPlugin>>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, path: &Path) -> Result<()>;
    pub fn get_transformer(&self, name: &str, config: Value)
        -> Result<Box<dyn Transformer>>;
}
```

## Advanced Features

### Conditional Transformations

```rust
pub struct ConditionalTransformer {
    condition: Box<dyn Fn(&FileEntry) -> bool>,
    transformer: Box<dyn Transformer>,
}

// Example: Only transform if file has tests
let transformer = ConditionalTransformer::new(
    |file| file.content.contains("test"),
    Box::new(CaseTransformer::new(...))
);
```

### Transformation Composition

```rust
pub enum CompositionStrategy {
    Sequential,   // Apply transformers in order
    Parallel,     // Apply independently and merge
    FirstMatch,   // Apply first applicable transformer
}

pub struct CompositeTransformer {
    transformers: Vec<Box<dyn Transformer>>,
    strategy: CompositionStrategy,
}
```

### Transaction Support

```rust
pub struct TransactionManager {
    snapshots: HashMap<PathBuf, String>,
}

impl TransactionManager {
    pub fn begin(&mut self, context: &TransformContext);
    pub fn commit(&mut self) -> Result<()>;
    pub fn rollback(&mut self) -> Result<()>;
}
```

### Parallel Execution

```rust
impl Pipeline {
    fn execute_parallel(&self, transformer: &dyn Transformer,
                       context: &mut TransformContext) -> Result<()> {
        use rayon::prelude::*;

        context.files.par_iter_mut()
            .try_for_each(|file| {
                transformer.transform_file(file)
            })?;

        Ok(())
    }
}
```

## Usage Patterns

### Simple Case Conversion

```rust
Pipeline::builder()
    .search_glob("**/*.rs")
    .transform_case(CaseFormat::SnakeCase, CaseFormat::CamelCase)
    .dry_run(true)
    .build()
    .execute(".")?;
```

### Complex Migration

```rust
Pipeline::builder()
    // Discovery
    .search_pattern(r"var\s+\w+", PatternType::Regex)
    .search_git(GitStatus::Modified)

    // Filtering
    .filter_extension(vec![".js", ".ts"])
    .filter_path(vec!["src/**"], vec!["src/tests/**"])
    .filter_semantic(SemanticCriteria::HasImport("oldApi"))

    // Transformations
    .transform_case(CaseFormat::SnakeCase, CaseFormat::CamelCase)
    .transform_quotes(QuoteStyle::Single, QuoteStyle::Double)
    .transform_imports(ImportAction::Sort {
        groups: vec![ImportGroup::Stdlib, ImportGroup::External]
    })
    .transform_custom(Box::new(ApiMigrationTransformer::new(mappings)))

    // Analysis
    .analyze_complexity()
    .analyze_metrics()

    .parallel(true)
    .workers(4)
    .build()
    .execute(".")?;
```

### From Configuration File

```rust
let pipeline = Pipeline::from_config("transform.yaml")?;
let report = pipeline.execute(".")?;

println!("Processed {} files", report.files_processed);
println!("Applied {} changes", report.changes.len());
```

### Interactive CLI

```bash
$ codeconvert interactive
> search glob "**/*.js"
Found 142 files

> filter content "var\\s+\\w+"
Filtered to 37 files

> filter semantic has_function "deprecated*"
Filtered to 12 files

> transform case snake camel --scope variables
> transform custom api_migration --config mappings.json

> analyze complexity --threshold 10
Files with high complexity: 3

> execute --dry-run
Would modify 12 files:
  - src/api/old.js: 15 changes
  - src/utils/helpers.js: 8 changes
  ...

> execute --confirm
Applied changes to 12 files
```

## Error Handling

### Error Types

```rust
pub enum TransformError {
    SearchError(String),
    FilterError(String),
    TransformError(String),
    AnalysisError(String),
    ConfigError(String),
    PluginError(String),
    IoError(std::io::Error),
}

pub type Result<T> = std::result::Result<T, TransformError>;
```

### Recovery Strategies

```rust
pub enum ErrorStrategy {
    Fail,              // Stop on first error
    Skip,              // Skip failed files, continue
    Rollback,          // Revert all changes on error
    Collect,           // Collect all errors, report at end
}
```

## Performance Considerations

1. **Lazy AST Parsing**: Parse syntax tree only when needed
2. **Parallel Execution**: Process files concurrently with rayon
3. **Incremental Processing**: Skip unchanged files
4. **Memory Mapping**: Use mmap for large files
5. **Cache Management**: Cache compiled regexes and AST queries

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_case_transformer() {
    let transformer = CaseTransformer::new(
        CaseFormat::SnakeCase,
        CaseFormat::CamelCase
    );

    let mut context = TransformContext::from_str("my_variable = 1");
    transformer.transform(&mut context).unwrap();

    assert_eq!(context.content(), "myVariable = 1");
}
```

### Integration Tests

```rust
#[test]
fn test_pipeline_execution() {
    let pipeline = Pipeline::builder()
        .search_glob("tests/fixtures/**/*.js")
        .transform_case(CaseFormat::SnakeCase, CaseFormat::CamelCase)
        .build();

    let report = pipeline.execute("tests/fixtures").unwrap();
    assert_eq!(report.files_processed, 5);
}
```

### Property-Based Tests

```rust
#[quickcheck]
fn prop_case_conversion_reversible(input: String) -> bool {
    let to_camel = transform_case(&input, CaseFormat::Snake, CaseFormat::Camel);
    let back_to_snake = transform_case(&to_camel, CaseFormat::Camel, CaseFormat::Snake);
    normalize(&input) == normalize(&back_to_snake)
}
```

## Future Extensions

1. **Language Server Protocol (LSP)**: IDE integration for real-time transformations
2. **Web Assembly**: Run transformations in browser
3. **Distributed Execution**: Process large codebases across multiple machines
4. **AI-Assisted Transformations**: ML-based code understanding and migration
5. **Version Control Integration**: Automatic branch/commit creation
6. **Collaboration**: Multi-user transformation review and approval
7. **Transformation Marketplace**: Share and discover transformation plugins

## Migration Path

### From Current Implementation

1. **Phase 1**: Extract core traits and pipeline structure
2. **Phase 2**: Migrate existing CaseTransformer to new architecture
3. **Phase 3**: Implement filter library
4. **Phase 4**: Add plugin system and configuration loading
5. **Phase 5**: Implement parallel execution and optimization

### Backwards Compatibility

Provide compatibility layer for existing CLI:

```rust
// Old API still works
codeconvert --from-camel --to-snake src/

// Maps to new pipeline:
Pipeline::builder()
    .search_glob("src/**/*")
    .transform_case(CaseFormat::CamelCase, CaseFormat::SnakeCase)
    .build()
```

## References

- [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) - Parser generator for AST-based matching
- [Comby](https://comby.dev/) - Structural code search and replace inspiration
- [Semgrep](https://semgrep.dev/) - Semantic code analysis patterns
- [Codemod](https://github.com/facebook/codemod) - Facebook's transformation framework
- [jscodeshift](https://github.com/facebook/jscodeshift) - JavaScript codemods
