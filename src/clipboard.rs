use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use arboard::SetExtLinux;

use crate::color::okhsl::Okhsl;
use crate::color::{ColorFormat, Precision};

const SERVE_ENVIRONMENT_VARIABLE: &str = "COLORPICKLE_CLIPBOARD_SERVE";

static CLIPBOARD: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();

pub fn copy_color(format: ColorFormat, color: Okhsl, precision: Precision) -> Result<String> {
    let value = format.format(color, precision);
    set_text(value.clone())?;
    Ok(value)
}

/// A clipboard selection is owned by the process that set it, so closing the
/// window would take the copied colour with it. Hand the text to a child that
/// keeps serving it and exits once another process claims the clipboard.
pub fn set_text(text: impl Into<String>) -> Result<()> {
    let text = text.into();
    match spawn_server(&text) {
        Ok(()) => Ok(()),
        Err(error) => {
            tracing::warn!(
                ?error,
                "clipboard helper failed; using the in-process clipboard"
            );
            set_text_in_process(text)
        }
    }
}

/// Runs in the helper started by [`set_text`]. `main` calls this before
/// anything else, so the helper opens no window and no log file.
pub fn serve() -> Result<()> {
    let mut text = String::new();
    std::io::stdin()
        .read_to_string(&mut text)
        .context("failed to read the colour")?;
    let mut clipboard = arboard::Clipboard::new().context("failed to open the system clipboard")?;
    clipboard
        .set()
        .wait()
        .text(text)
        .context("failed to write to the system clipboard")
}

pub fn is_server() -> bool {
    std::env::var_os(SERVE_ENVIRONMENT_VARIABLE).is_some_and(|value| value.to_str() == Some("1"))
}

fn spawn_server(text: &str) -> Result<()> {
    let executable = std::env::current_exe().context("failed to find the executable")?;
    let mut child = Command::new(executable)
        .env(SERVE_ENVIRONMENT_VARIABLE, "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .context("failed to start the clipboard helper")?;
    child
        .stdin
        .take()
        .context("the clipboard helper has no input")?
        .write_all(text.as_bytes())
        .context("failed to send the colour to the clipboard helper")?;
    // The helper exits once the clipboard is replaced; reap it then.
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

fn set_text_in_process(text: String) -> Result<()> {
    let cell = CLIPBOARD.get_or_init(|| Mutex::new(None));
    let mut clipboard = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if clipboard.is_none() {
        *clipboard =
            Some(arboard::Clipboard::new().context("failed to open the system clipboard")?);
    }
    clipboard
        .as_mut()
        .expect("clipboard initialized")
        .set_text(text)
        .context("failed to write to the system clipboard")
}
