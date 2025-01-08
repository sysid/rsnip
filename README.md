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
Hello {{ env_USER }}!
---

--- backup
tar -czf backup-{{ current_date|strftime('%Y%m%d') }}.tar.gz ./
---
```

3. Use your snippets:
```bash
# List available snippets
rsnip list --ctype shell

# Copy a snippet to clipboard
rsnip copy --ctype shell --input greeting

# Interactive fuzzy search
rsnip complete --ctype shell --interactive
```

### Shell Integration

Add to your `.bashrc`:
```bash
# Optional: Add convenient alias
alias ,="rsnip copy --ctype shell --input"

# Enable tab completion
source /path/to/rsnip.bash
```

Now you can use:
```bash
, back<tab>  # Will fuzzy-find and complete 'backup'
```

## ⚙️ Configuration

RSnip looks for configuration in the following locations:
- `~/.config/rsnip/config.toml`
- `~/.rsnip/config.toml`
- `/etc/rsnip/config.toml`

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

Environment variables are available as `env_VARNAME`:
```
--- path
Current path is: {{ env_PATH }}
---
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

Contributions are welcome! Please feel free to submit a Pull Request.

## License

## 🙏 Acknowledgments

- Inspired by various snippet managers and completion tools
- Built with Rust and several awesome crates including clap, minijinja, and skim