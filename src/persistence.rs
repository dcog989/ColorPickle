use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};

/// Reads a TOML document from `path`, falling back to `T::default()` when the
/// file is absent, unreadable or malformed.
pub fn load<T: Default + DeserializeOwned>(path: &Path) -> T {
    if !path.exists() {
        return T::default();
    }
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!(
                ?error,
                path = %path.display(),
                "failed to read the file; using defaults"
            );
            return T::default();
        }
    };
    match toml::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            tracing::warn!(
                ?error,
                path = %path.display(),
                "failed to parse the file; using defaults"
            );
            T::default()
        }
    }
}

/// Writes `value` to `path` as TOML, replacing the file in one step so a crash
/// or a full disk cannot leave a half-written file behind.
pub fn save<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(value).context("failed to serialize")?;
    let temporary = temporary_path(path);
    std::fs::write(&temporary, text)
        .with_context(|| format!("failed to write {}", temporary.display()))?;
    std::fs::rename(&temporary, path)
        .with_context(|| format!("failed to replace {}", path.display()))
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".tmp");
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::{load, save};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(default)]
    struct Document {
        value: u8,
    }

    fn temporary_path(name: &str) -> std::path::PathBuf {
        let directory =
            std::env::temp_dir().join(format!("colorpickle-persist-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        directory.join("document.toml")
    }

    #[test]
    fn round_trips_through_a_file() {
        let path = temporary_path("round-trip");
        let document = Document { value: 7 };

        save(&path, &document).unwrap();

        assert_eq!(load::<Document>(&path), document);
        assert_eq!(
            load::<Document>(&path.with_file_name("absent.toml")),
            Document::default()
        );

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn falls_back_to_defaults_on_malformed_toml() {
        let path = temporary_path("malformed");
        std::fs::write(&path, "value = ").unwrap();

        assert_eq!(load::<Document>(&path), Document::default());

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
