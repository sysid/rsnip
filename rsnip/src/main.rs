use anyhow::Result;
use clap::Parser;

/// A command-line snippet manager
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value = "world")]
    name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("Hello, {}!", args.name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_name_when_parsing_args_then_returns_correct_name() {
        let args = Args::parse_from(&["test", "--name", "Rustacean"]);
        assert_eq!(args.name, "Rustacean");
    }
}
