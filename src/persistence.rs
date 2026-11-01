use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};

const BROKEN_SUFFIX: &str = ".broken";
const TMP_SUFFIX: &str = ".tmp";

/// Reads a TOML document from `path`, falling back to `T::default()` when the
/// file is absent, unreadable or malformed. A malformed file is moved aside
/// first, so the next save cannot silently overwrite what the user wrote.
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
            let aside = set_aside(path);
            tracing::warn!(
                ?error,
                path = %path.display(),
                aside = %aside.display(),
                "failed to parse the file; using defaults"
            );
            T::default()
        }
    }
}

fn set_aside(path: &Path) -> PathBuf {
    let aside = with_suffix(path, BROKEN_SUFFIX);
    if let Err(error) = std::fs::rename(path, &aside) {
        tracing::warn!(?error, path = %path.display(), "failed to move the file aside");
    }
    aside
}

/// Writes `value` to `path` as TOML, replacing the file in one step so a crash
/// or a full disk cannot leave a half-written file behind: the temp file is
/// flushed to disk before the rename makes it visible.
pub fn save<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let text = toml::to_string_pretty(value).context("failed to serialize")?;
    let temporary = with_suffix(path, TMP_SUFFIX);
    let mut file = std::fs::File::create(&temporary)
        .with_context(|| format!("failed to write {}", temporary.display()))?;
    file.write_all(text.as_bytes())
        .with_context(|| format!("failed to write {}", temporary.display()))?;
    file.sync_all()
        .with_context(|| format!("failed to flush {}", temporary.display()))?;
    drop(file);
    std::fs::rename(&temporary, path)
        .with_context(|| format!("failed to replace {}", path.display()))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(suffix);
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
        // The user's text is kept, so the next save cannot destroy it.
        assert!(!path.exists());
        let aside = path.with_extension("toml.broken");
        assert_eq!(std::fs::read_to_string(&aside).unwrap(), "value = ");

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
