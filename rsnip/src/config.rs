// configuration
use crate::domain::SnippetType;
use crate::path_utils::expand_path;
use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, instrument};

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub snippet_types: HashMap<String, SnippetTypeConfig>,
    #[serde(default = "default_config_paths")]
    pub config_paths: Vec<PathBuf>,
    // Track which config file is active
    #[serde(skip)]
    pub active_config_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SnippetTypeConfig {
    pub source_file: PathBuf,
    #[serde(default)]
    pub description: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            snippet_types: default_snippet_types(),
            config_paths: default_config_paths(),
            active_config_path: None,
        }
    }
}

fn default_config_paths() -> Vec<PathBuf> {
    vec![
        dirs::config_dir().map(|p| p.join("rsnip/config.toml")),
        dirs::home_dir().map(|p| p.join(".config/rsnip/config.toml")),
        Some(PathBuf::from("/etc/rsnip/config.toml")),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn default_snippet_types() -> HashMap<String, SnippetTypeConfig> {
    let mut types = HashMap::new();
    types.insert(
        "default".to_string(),
        SnippetTypeConfig {
            source_file: PathBuf::from("completion_source.txt"),
            description: Some("Default snippet type".to_string()),
        },
    );
    types
}

impl Settings {
    pub fn new() -> Result<Self> {
        let mut builder = config::Config::builder();

        // Load default config first
        builder = builder.add_source(config::File::from_str(
            include_str!("default_config.toml"),
            config::FileFormat::Toml,
        ));

        // Try loading from config paths and track which one succeeded
        let mut active_path = None;
        for path in default_config_paths() {
            debug!("Trying to load config from: {:?}", path);
            if path.exists() {
                builder = builder.add_source(config::File::from(path.as_path()));
                active_path = Some(path.clone());
                break; // Use first found config file
            }
        }

        // Override with environment variables
        builder = builder.add_source(
            config::Environment::with_prefix("RSNIP")
                .separator("_")
                .try_parsing(true),
        );

        let config = builder.build()?;
        let mut settings: Settings = config.try_deserialize()?;

        // Store the active config path
        settings.active_config_path = active_path;

        // Expand paths in snippet types
        for config in settings.snippet_types.values_mut() {
            config.source_file = expand_path(&config.source_file)?;
        }

        debug!("Loaded config: {:?}", settings);
        Ok(settings)
    }
    pub fn get_snippet_type(&self, name: &str) -> Option<SnippetTypeConfig> {
        self.snippet_types.get(name).cloned()
    }
}

// Update infrastructure.rs to use the new config
#[instrument(level = "debug")]
pub fn get_snippet_type(config: &Settings, name: &str) -> Result<SnippetType> {
    let snippet_config = config.get_snippet_type(name).ok_or_else(|| {
        anyhow::anyhow!("Unknown snippet type: {}, update your configuration.", name)
    })?;

    Ok(SnippetType {
        name: name.to_string(),
        source_file: snippet_config.source_file,
    })
}
