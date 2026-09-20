use anyhow::{Context, Result};

pub fn set_text(text: impl Into<String>) -> Result<()> {
    let mut clipboard = arboard::Clipboard::new().context("failed to open the system clipboard")?;
    clipboard
        .set_text(text.into())
        .context("failed to write to the system clipboard")
}
