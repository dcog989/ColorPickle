//! Desktop integration for the AppImage build.
//!
//! An AppImage has no installer, so the menu entry and icon are written on
//! launch. Release filenames carry the version, so `Exec` is repointed whenever
//! the running file differs.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

const DESKTOP_FILE: &str = "colorpickle.desktop";
const ICON_FILE: &str = "colorpickle.svg";
const ICON_SVG: &[u8] = include_bytes!("../assets/colorpickle.svg");
/// The installed entry's `Exec` is absolute; the AppImage entry is the same
/// file with that value replaced.
const DESKTOP_TEMPLATE: &str = include_str!("../packaging/colorpickle.desktop");
const INSTALLED_EXEC: &str = "Exec=/usr/bin/colorpickle";
/// Records the AppImage this entry was generated for, so a stale entry is
/// recognised without parsing the quoted `Exec` value.
const MARKER: &str = "X-ColorPickle-AppImage=";
/// A package install already provides the entry; a user copy would shadow it.
const SYSTEM_DESKTOP_FILE: &str = "/usr/share/applications/colorpickle.desktop";

/// Registers the running AppImage, or removes an entry left behind by one that
/// no longer exists, so the menu always points at the file the user has.
pub fn install() {
    if let Err(error) = sync() {
        tracing::warn!("AppImage desktop integration failed: {error:#}");
    }
}

fn sync() -> Result<()> {
    let dirs = directories::BaseDirs::new().context("no XDG data directory — is HOME set?")?;
    let applications = dirs.data_dir().join("applications");
    let system_install = Path::new(SYSTEM_DESKTOP_FILE).exists();

    let Some(appimage) = running_appimage().filter(|_| !system_install) else {
        remove_generated(&applications);
        return Ok(());
    };

    let icons = dirs.data_dir().join("icons/hicolor/scalable/apps");
    std::fs::create_dir_all(&applications)?;
    std::fs::create_dir_all(&icons)?;
    write_if_changed(&icons.join(ICON_FILE), ICON_SVG)?;

    let path = applications.join(DESKTOP_FILE);
    let content = appimage_entry(&appimage);
    if std::fs::read_to_string(&path).ok().as_deref() != Some(content.as_str()) {
        std::fs::write(&path, content.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
        refresh_menu(&applications);
    }
    Ok(())
}

/// The AppImage launcher sets `APPIMAGE` to the file the user runs. Inside the
/// mount `current_exe()` points at a path that disappears on exit.
fn running_appimage() -> Option<PathBuf> {
    std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .filter(|path| path.is_file())
}

/// Rewrites every `Exec=/usr/bin/colorpickle…` line, so the `Pick` action keeps
/// its `--launch-mode` argument.
fn appimage_entry(appimage: &Path) -> String {
    let exec = exec_quote(&appimage.to_string_lossy());
    let mut entry = String::new();
    for line in DESKTOP_TEMPLATE.lines() {
        if let Some(rest) = line.strip_prefix(INSTALLED_EXEC) {
            entry.push_str(&format!("Exec={exec}{rest}\n"));
        } else {
            entry.push_str(line);
            entry.push('\n');
        }
    }
    entry.push_str(&format!("{MARKER}{}\n", appimage.display()));
    entry
}

/// Deletes the entry only when it carries `MARKER`: a package install could
/// otherwise be removed by a run of a different build.
fn remove_generated(applications: &Path) {
    let path = applications.join(DESKTOP_FILE);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let Some(appimage) = marker_value(&content) else {
        return;
    };
    if Path::new(appimage).is_file() {
        return;
    }
    let _ = std::fs::remove_file(&path);
    refresh_menu(applications);
}

fn marker_value(content: &str) -> Option<&str> {
    content.lines().find_map(|line| line.strip_prefix(MARKER))
}

/// Quotes a value for a desktop-entry `Exec` key: double quotes, with `"`, `` ` ``,
/// `$` and `\` escaped by a backslash and `%` doubled, per the freedesktop
/// Desktop Entry specification.
fn exec_quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                out.push('\\');
                out.push(c);
            }
            '%' => {
                out.push('%');
                out.push('%');
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn write_if_changed(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if std::fs::read(path).ok().as_deref() == Some(bytes) {
        return Ok(());
    }
    std::fs::write(path, bytes)
}

/// Rebuilds the desktop-file cache. Absent on minimal systems; KDE also
/// refreshes through KSycoca, so a failure is not fatal.
fn refresh_menu(applications: &Path) {
    let _ = std::process::Command::new("update-desktop-database")
        .arg(applications)
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_registers_the_running_appimage() {
        let entry = appimage_entry(Path::new("/home/user/Apps/ColorPickle 2.AppImage"));
        assert!(
            entry.contains("Exec=\"/home/user/Apps/ColorPickle 2.AppImage\"\n"),
            "{entry}"
        );
        assert!(
            entry.contains(
                "Exec=\"/home/user/Apps/ColorPickle 2.AppImage\" --launch-mode picker_first\n"
            ),
            "{entry}"
        );
        assert!(
            entry.contains(&format!("{MARKER}/home/user/Apps/ColorPickle 2.AppImage\n")),
            "{entry}"
        );
        // The KWin grant must survive, or the AppImage loses ScreenShot2.
        assert!(
            entry.contains("X-KDE-DBUS-Restricted-Interfaces=org.kde.KWin.ScreenShot2\n"),
            "{entry}"
        );
        assert!(!entry.contains("/usr/bin/colorpickle"), "{entry}");
    }

    #[test]
    fn marker_is_read_back_verbatim() {
        let entry = appimage_entry(Path::new("/opt/ColorPickle-0.13.1-x86_64.AppImage"));
        assert_eq!(
            marker_value(&entry),
            Some("/opt/ColorPickle-0.13.1-x86_64.AppImage")
        );
        assert_eq!(marker_value("[Desktop Entry]\n"), None);
    }

    #[test]
    fn exec_quote_escapes_the_specials_and_spaces() {
        assert_eq!(
            exec_quote("/opt/ColorPickle 2.AppImage"),
            "\"/opt/ColorPickle 2.AppImage\""
        );
        assert_eq!(exec_quote("/opt/a%b.AppImage"), "\"/opt/a%%b.AppImage\"");
    }
}
