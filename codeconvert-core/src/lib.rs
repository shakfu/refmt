//! Core library for code transformation and case conversion
//!
//! This library provides the fundamental building blocks for transforming code,
//! including case format conversion, pattern matching, and file processing.

pub mod case;
pub mod converter;
pub mod whitespace;

// Re-export commonly used types
pub use case::CaseFormat;
pub use converter::CaseConverter;
pub use whitespace::{WhitespaceCleaner, WhitespaceOptions};

// Re-export Result type
pub type Result<T> = anyhow::Result<T>;
