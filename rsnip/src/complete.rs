// complete.rs
use crate::config::{Settings, SnippetTypeConfig};
use anyhow::Result;
use std::io::Write;
use clap_complete::Shell;
use minijinja::Environment;
use serde::Serialize;
use tracing::{debug, instrument};

// Create a serializable type for template context
#[derive(Serialize, Debug)]
struct SnippetTypeContext<'a> {
    name: &'a str,
    alias: Option<String>,
}

#[instrument(level = "debug", , skip(writer))]
pub fn generate_completion_script(
    shell: Shell,
    mut writer: impl Write,
    config: &Settings,
) -> Result<()> {
    match shell {
        Shell::Bash => {
            let mut env = Environment::new();
            env.set_debug(true); // Enable debug mode

            // Load the template
            env.add_template(
                "bash_completion",
                include_str!("../rsnip.alias.bash.template"),
            )?;

            // Get template
            let tmpl = env.get_template("bash_completion")?;
            debug!("Loaded template: {:?}", tmpl);

            // Create context with snippet types and their aliases
            let snippet_types: Vec<_> = config.snippet_types
                .iter()
                .map(|(name, cfg)| {
                    let alias = match cfg {
                        SnippetTypeConfig::Concrete { alias, .. } => alias.clone(),
                        SnippetTypeConfig::Combined { alias, .. } => alias.clone(),
                    };
                    SnippetTypeContext {
                        name,
                        alias,
                    }
                })
                .collect();
            debug!("Found snippet types with aliases: {:?}", snippet_types);

            let context = minijinja::context! {
                snippet_types => snippet_types,
            };

            // Render and write
            let rendered = tmpl.render(context)?;
            writer.write_all(rendered.as_bytes())?;

            Ok(())
        }
        _ => Err(anyhow::anyhow!(
            "Only Bash completion is currently supported"
        )),
    }
}
