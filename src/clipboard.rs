use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};

use crate::color::ColorFormat;
use crate::color::okhsl::Okhsl;

static CLIPBOARD: OnceLock<Mutex<Option<arboard::Clipboard>>> = OnceLock::new();

pub fn copy_color(format: ColorFormat, color: Okhsl) -> Result<String> {
    let value = format.format(color);
    set_text(value.clone())?;
    Ok(value)
}

pub fn set_text(text: impl Into<String>) -> Result<()> {
    let cell = CLIPBOARD.get_or_init(|| Mutex::new(None));
    let mut clipboard = cell.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if clipboard.is_none() {
        *clipboard =
            Some(arboard::Clipboard::new().context("failed to open the system clipboard")?);
    }
    clipboard
        .as_mut()
        .expect("clipboard initialized")
        .set_text(text.into())
        .context("failed to write to the system clipboard")
}
