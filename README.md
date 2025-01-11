# RSnip: Fast & Flexible Text Snippets 🚀

RSnip is a command-line text snippet manager built in Rust that helps you save and reuse frequently used text snippets with powerful templating capabilities.

[![CI Status](https://img.shields.io/github/workflow/status/yourusername/rsnip/CI)](https://github.com/yourusername/rsnip/actions)
[![Crates.io](https://img.shields.io/crates/v/rsnip)](https://crates.io/crates/rsnip)
[![License](https://img.shields.io/crates/l/rsnip)](LICENSE)

## 🌟 Features

- **Multiple Snippet Types**: Organize snippets into different categories (shell commands, code snippets, etc.)
- **Fuzzy Search**: Fast fuzzy finding with interactive selection using fzf-style interface
- **Shell Integration**: Tab completion for your snippets in bash
- **Dynamic Templates**: Support for dynamic content using Jinja2-style templates
- **Shell-friendly**: Direct shell integration with aliases and completions
- **Configurable**: TOML-based configuration with multiple config file locations
- **Fast**: Written in Rust for optimal performance
- **Debug Support**: Configurable debug levels for troubleshooting
- **Smart Completion**: Both exact and fuzzy matching for snippet names

## 🚀 Quick Start

### Installation

```bash
cargo install rsnip
```

### Basic Usage

1. Create a snippet:
```bash
rsnip edit --ctype shell  # Opens your default editor
```

2. Add some snippets in the format:
```
--- greeting
: This is a comment about the greeting snippet
Hello {{ env_USER }}!
---

--- backup
: Creates a dated backup archive
tar -czf backup-{{ current_date|strftime('%Y%m%d') }}.tar.gz ./
---
```

3. Use your snippets:
```bash
# List all snippet types
rsnip types

# List available snippets for a type
rsnip list --ctype shell

# Copy a snippet to clipboard
rsnip copy --ctype shell --input greeting

# Interactive fuzzy search
rsnip complete --ctype shell --interactive
```

### Command Line Options

```bash
USAGE:
    rsnip [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -d, --debug             Enable debug logging (multiple -d increases verbosity)
        --generate-config   Print default configuration to stdout
        --info             Display version and configuration information
    -h, --help             Prints help information
    -V, --version          Prints version information

OPTIONS:
        --generate <SHELL>  Generate shell completion scripts (bash)

SUBCOMMANDS:
    types      List available snippet types
    list       List all snippets
    edit       Edit snippet file in system editor
    complete   Find completions with optional interactive selection
    copy       Copy text to clipboard
    help       Prints this message or help for given subcommand
```

### Shell Integration

Add to your `.bashrc`:
```bash
# Optional: Add convenient alias
alias ,="rsnip copy --ctype shell --input"

# Enable tab completion
source <(rsnip --generate bash)
```

Now you can use:
```bash
, back<tab>  # Will fuzzy-find and complete 'backup'
```

## ⚙️ Configuration

RSnip looks for configuration in the following locations (in order):
1. `~/.config/rsnip/config.toml`
2. `~/.config/rsnip/config.toml`
3. `/etc/rsnip/config.toml`

Example configuration:
```toml
[snippet_types.shell]
source_file = "~/.config/rsnip/shell_snippets.txt"
description = "Shell command snippets"

[snippet_types.git]
source_file = "~/.config/rsnip/git_snippets.txt"
description = "Git commands and workflows"
```

## 🛠️ Template Features

RSnip supports Jinja2-style templates with several built-in filters:

- `strftime`: Format dates - `{{ current_date|strftime('%Y-%m-%d') }}`
- `add_days`: Add days to date - `{{ current_date|add_days(7) }}`
- `subtract_days`: Subtract days - `{{ current_date|subtract_days(7) }}`
- `shell`: Execute shell commands (safely) - `{{ 'date +%Y' | shell }}`

Built-in variables:
- `current_date`: Current date in ISO format
- Environment variables are available as `env_VARNAME`:
```
--- path
Current path is: {{ env_PATH }}
---
```

### Snippet File Format

Snippets are stored in text files with a simple format:
```
: File-level comments (optional)

--- snippet_name
: Comment about this snippet
: Another comment
Content goes here
Can be multiple lines
---
```

## 🔍 Debug and Troubleshooting

RSnip supports multiple debug levels:
```bash
rsnip -d        # Info level
rsnip -dd       # Debug level
rsnip -ddd      # Trace level
```

View configuration and version info:
```bash
rsnip --info
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development

```bash
# Clone the repository
git clone https://github.com/yourusername/rsnip
cd rsnip

# Run tests
cargo test

# Build in release mode
cargo build --release
```

## License

This project is licensed under the BSD 3 License - see the LICENSE file for details.

## 🙏 Acknowledgments

- Inspired by various snippet managers and completion tools
- Built with Rust and several awesome crates including clap, minijinja, skim, and more