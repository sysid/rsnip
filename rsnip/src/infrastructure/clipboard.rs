use anyhow::{Context, Result};
use arboard::Clipboard;
use tracing::instrument;

#[instrument(level = "trace")]
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
    let clean_text = text.trim_end_matches('\n');
    clipboard
        .set_text(clean_text)
        .context("Failed to set clipboard text")?;
    Ok(())
}