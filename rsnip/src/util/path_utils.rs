use anyhow::Result;
use std::path::{Path, PathBuf};
use tracing::instrument;

/// Expands a path that may contain a tilde for the home directory
#[instrument(level = "trace")]
pub fn expand_path<P: AsRef<Path>  + std::fmt::Debug>(path: P) -> Result<PathBuf> {
    let path_str = path.as_ref().to_string_lossy();

    if path_str.starts_with('~') {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

        if path_str == "~" {
            Ok(home)
        } else if path_str.starts_with("~/") {
            Ok(home.join(&path_str[2..]))
        } else {
            Err(anyhow::anyhow!("Invalid path: {}", path_str))
        }
    } else {
        Ok(path.as_ref().to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_path_with_tilde_when_expanding_then_replaces_with_home() -> Result<()> {
        if let Some(home) = dirs::home_dir() {
            let path = expand_path("~/test/path")?;
            assert_eq!(path, home.join("test/path"));
        }
        Ok(())
    }

    #[test]
    fn given_absolute_path_when_expanding_then_returns_unchanged() -> Result<()> {
        let path = expand_path("/abs/path")?;
        assert_eq!(path, PathBuf::from("/abs/path"));
        Ok(())
    }

    #[test]
    fn given_relative_path_when_expanding_then_returns_unchanged() -> Result<()> {
        let path = expand_path("rel/path")?;
        assert_eq!(path, PathBuf::from("rel/path"));
        Ok(())
    }

    #[test]
    fn given_invalid_tilde_path_when_expanding_then_returns_error() {
        assert!(expand_path("~invalid").is_err());
    }
}
