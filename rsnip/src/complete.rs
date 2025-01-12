// complete.rs
use crate::config::Settings;
use anyhow::Result;
use std::io::Write;
use clap_complete::Shell;
use minijinja::Environment;

pub fn generate_completion_script(shell: Shell, mut writer: impl Write, config: &Settings) -> Result<()> {
    match shell {
        Shell::Bash => {
            let mut env = Environment::new();

            // Load the template
            env.add_template("bash_completion", include_str!("../rsnip.alias.bash.template"))?;

            // Get template
            let tmpl = env.get_template("bash_completion")?;

            // Create context with snippet types that have aliases
            let snippet_types: Vec<_> = config.snippet_types
                .iter()
                .map(|(name, cfg)| (name, cfg))
                .collect();

            let context = minijinja::context! {
                snippet_types => snippet_types,
            };

            // Render and write
            let rendered = tmpl.render(context)?;
            writer.write_all(rendered.as_bytes())?;

            Ok(())
        },
        _ => {
            Err(anyhow::anyhow!("Only Bash completion is currently supported"))
        }
    }
}