//! Reading files as text, safely.
//!
//! Transformers used to call `fs::read_to_string(path)?` and propagate the
//! error, so a single latin-1 `.txt` or a binary file carrying a matching
//! extension aborted the entire directory walk partway through, leaving the
//! tree half-processed. Reading through [`read_text`] skips such files and
//! lets the run continue.

use std::fs;
use std::path::Path;

/// Reads a file as UTF-8 text.
///
/// Returns `Ok(None)` -- logging a warning -- if the file looks binary or is
/// not valid UTF-8. I/O errors are still returned as `Err`, since those
/// indicate a problem worth reporting (missing file, permissions).
pub fn read_text(path: &Path) -> crate::Result<Option<String>> {
    let bytes = fs::read(path)?;

    // A NUL byte is the conventional binary marker; no text format uses one.
    if bytes.contains(&0) {
        log::warn!("Skipping binary file: {}", path.display());
        return Ok(None);
    }

    match String::from_utf8(bytes) {
        Ok(text) => Ok(Some(text)),
        Err(_) => {
            log::warn!("Skipping file with non-UTF-8 contents: {}", path.display());
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Returns an owned temporary directory plus a path inside it. The
    /// caller must keep the `TempDir` alive; dropping it removes the file.
    fn tmp(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::Builder::new()
            .prefix("reformat-text-")
            .tempdir()
            .unwrap();
        let path = dir.path().join(name);
        (dir, path)
    }

    #[test]
    fn test_reads_utf8() {
        let (_dir, p) = tmp("ok.txt");
        fs::write(&p, "caf\u{e9}\n").unwrap();
        assert_eq!(read_text(&p).unwrap().as_deref(), Some("caf\u{e9}\n"));
    }

    #[test]
    fn test_skips_binary() {
        let (_dir, p) = tmp("bin.txt");
        fs::write(&p, [0x50, 0x4b, 0x00, 0x01]).unwrap();
        assert!(read_text(&p).unwrap().is_none());
    }

    #[test]
    fn test_skips_invalid_utf8() {
        let (_dir, p) = tmp("latin1.txt");
        // 0xE9 alone is valid latin-1 but not valid UTF-8.
        fs::write(&p, [b'c', b'a', b'f', 0xE9, b'\n']).unwrap();
        assert!(read_text(&p).unwrap().is_none());
    }

    #[test]
    fn test_io_error_is_reported() {
        let (_dir, p) = tmp("definitely_missing_file.txt");
        assert!(read_text(&p).is_err());
    }
}
