use std::io::Write;
use clap_complete::Shell;

pub fn generate_completion_script(shell: Shell, mut writer: impl Write) -> anyhow::Result<()> {
    match shell {
        Shell::Bash => {
            let content = include_str!("../rsnip.alias.bash");
            writer.write_all(content.as_bytes())?;
        }
        _ => {
            return Err(anyhow::anyhow!("Only Bash completion is currently supported"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bash_completion() -> anyhow::Result<()> {
        let mut buffer = Vec::new();
        generate_completion_script(Shell::Bash, &mut buffer)?;

        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("_rsnip_complete"), "Output doesn't contain completion function");
        Ok(())
    }

    #[test]
    fn test_unsupported_shell() {
        let mut buffer = Vec::new();
        let result = generate_completion_script(Shell::Fish, &mut buffer);
        assert!(result.is_err());
        assert!(buffer.is_empty());
    }
}