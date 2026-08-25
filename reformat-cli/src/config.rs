//! Configuration file loading for reformat presets.
//!
//! Looks for `reformat.json` in the current working directory.

use std::fs;
use std::path::Path;

use reformat_core::config::{validate_steps, ReformatConfig};
use reformat_core::Preset;

pub const CONFIG_FILENAME: &str = "reformat.json";

/// Load and parse `reformat.json` from the given directory.
/// Returns `None` if the file does not exist.
pub fn load_config_from(dir: &Path) -> anyhow::Result<Option<ReformatConfig>> {
    let path = dir.join(CONFIG_FILENAME);
    if !path.is_file() {
        return Ok(None);
    }
    log::debug!("Loading config from: {}", path.display());
    let content = fs::read_to_string(&path)?;
    let config: ReformatConfig = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("failed to parse {}: {}", CONFIG_FILENAME, e))?;
    Ok(Some(config))
}

/// Load and parse `reformat.json`, searching the current directory and then
/// each ancestor. Returns `None` if no config file is found.
///
/// Only the current directory used to be searched, so running from a
/// subdirectory of a project found nothing -- despite the documentation
/// describing the file as living at the project root.
pub fn load_config() -> anyhow::Result<Option<ReformatConfig>> {
    let cwd = std::env::current_dir()?;
    for dir in cwd.ancestors() {
        if let Some(config) = load_config_from(dir)? {
            return Ok(Some(config));
        }
    }
    Ok(None)
}

/// Look up a preset by name in the loaded config.
pub fn get_preset<'a>(config: &'a ReformatConfig, name: &str) -> anyhow::Result<&'a Preset> {
    let preset = config.get(name).ok_or_else(|| {
        let available: Vec<&str> = config.keys().map(|k| k.as_str()).collect();
        anyhow::anyhow!(
            "preset '{}' not found in {}. Available presets: {}",
            name,
            CONFIG_FILENAME,
            if available.is_empty() {
                "(none)".to_string()
            } else {
                available.join(", ")
            }
        )
    })?;
    validate_steps(name, &preset.steps)?;
    Ok(preset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_file_not_found() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let tmp = _tmp.path().to_path_buf();
        fs::create_dir_all(&tmp).unwrap();

        let result = load_config_from(&tmp).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_load_config_valid() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let tmp = _tmp.path().to_path_buf();
        fs::create_dir_all(&tmp).unwrap();

        fs::write(
            tmp.join("reformat.json"),
            r#"{"mypreset": {"steps": ["clean"]}}"#,
        )
        .unwrap();

        let config = load_config_from(&tmp).unwrap().unwrap();
        assert!(config.contains_key("mypreset"));
    }

    #[test]
    fn test_load_config_malformed_json() {
        // A unique directory per test: these run in parallel, and a shared
        // fixture path lets them clobber each other. TempDir also cleans up
        // when a test panics, which explicit teardown at the end does not.
        let _tmp = tempfile::tempdir().unwrap();
        let tmp = _tmp.path().to_path_buf();
        fs::create_dir_all(&tmp).unwrap();

        fs::write(tmp.join("reformat.json"), "not valid json {{{").unwrap();

        let result = load_config_from(&tmp);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_searches_ancestors() {
        let tmp = tempfile::Builder::new()
            .prefix("reformat-cfg-")
            .tempdir()
            .unwrap();
        let root = tmp.path();
        let nested = root.join("src").join("deep");
        fs::create_dir_all(&nested).unwrap();
        fs::write(
            root.join("reformat.json"),
            r#"{"mypreset": {"steps": ["clean"]}}"#,
        )
        .unwrap();

        // Found from the root itself...
        assert!(load_config_from(root).unwrap().is_some());
        // ...but not by a directory-only lookup further down.
        assert!(load_config_from(&nested).unwrap().is_none());

        // The ancestor walk finds it from the nested directory.
        let found = nested
            .ancestors()
            .find_map(|d| load_config_from(d).ok().flatten());
        assert!(
            found.is_some(),
            "a preset at the project root should be visible from a subdirectory"
        );
    }

    #[test]
    fn test_get_preset_found() {
        let json = r#"{"code": {"steps": ["rename", "clean"]}}"#;
        let config: ReformatConfig = serde_json::from_str(json).unwrap();
        let preset = get_preset(&config, "code").unwrap();
        assert_eq!(preset.steps, vec!["rename", "clean"]);
    }

    #[test]
    fn test_get_preset_not_found() {
        let json = r#"{"code": {"steps": ["clean"]}}"#;
        let config: ReformatConfig = serde_json::from_str(json).unwrap();
        let err = get_preset(&config, "missing").unwrap_err();
        assert!(err.to_string().contains("preset 'missing' not found"));
        assert!(err.to_string().contains("code"));
    }

    #[test]
    fn test_get_preset_invalid_step() {
        let json = r#"{"bad": {"steps": ["clean", "nope"]}}"#;
        let config: ReformatConfig = serde_json::from_str(json).unwrap();
        let err = get_preset(&config, "bad").unwrap_err();
        assert!(err.to_string().contains("unknown step 'nope'"));
    }
}
