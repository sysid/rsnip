# rsnip: Smart Snippet Management with Template Support 🚀

rsnip is a powerful command-line snippet manager that helps organize, find, and reuse text snippets with advanced templating capabilities. It features fuzzy search, intelligent shell integration, and dynamic template rendering.

[![Crates.io](https://img.shields.io/crates/v/rsnip)](https://crates.io/crates/rsnip)
[![License](https://img.shields.io/crates/l/rsnip)](LICENSE)

[Fast, Reliable, Yours: Why Snippets Outshine LLMs for boring Tasks](https://sysid.github.io/rsnip/)

## 🌟 Key Features

- **Smart Organization**: Categorize snippets into types (shell commands, code, notes, etc.)
- **Fuzzy Search**: Lightning-fast fuzzy finding with interactive fzf-style interface
- **Deep Shell Integration**: 
  - Tab completion for snippets in bash
  - Customizable aliases per snippet type
  - Interactive fuzzy completion
- **Dynamic Templates**: 
  - Jinja2-style template syntax
  - Date manipulation filters
  - Environment variable access
  - Safe shell command execution
- **Flexible Configuration**: 
  - TOML-based configuration
  - Per-snippet-type settings
- **Developer-Friendly**: Comprehensive debugging support

## 🚀 Quick Start

### Installation

```bash
cargo install rsnip
```

### Basic Setup

1. Initialize configuration:
```bash
# Generate default config
rsnip --generate-config > ~/.config/rsnip/config.toml
```

2. Add shell integration to your `.bashrc`:
```bash
# Enable tab completion and aliases
source <(rsnip --generate bash)
```

3. Create your first snippet file:
```bash
rsnip edit --ctype shell
```

### Configuration

RSnip uses TOML configuration with rich customization options:

```toml
[snippet_types.shell]
source_file = "~/.config/rsnip/shell_snippets.txt"
description = "Shell commands and scripts"
alias = ","  # Quick access alias

[snippet_types.git]
source_file = "~/.config/rsnip/git_snippets.txt"
description = "Git workflows"
alias = ",g"  # Git-specific alias

[snippet_types.docker]
source_file = "~/.config/rsnip/docker_snippets.txt"
description = "Docker commands"
alias = ",d"  # Docker-specific alias
```

Configuration is searched in:
1. `~/.config/rsnip/config.toml`
2. `~/.config/rsnip/config.toml`
3. `/etc/rsnip/config.toml`

### Snippet Format

Snippets use a clear, readable format:

```
: Optional file-level comments

--- snippet_name
: Comment describing the snippet
: Additional comment lines
Content goes here
Multiple lines supported
---

--- template_example
: Example using templates
Hello {{ env_USER }}!
Created on: {{ current_date|strftime('%Y-%m-%d') }}
---
```

## 🛠️ Advanced Features

### Shell Integration & Aliases

RSnip provides powerful shell integration:

1. **Type-Specific Aliases**: Configure quick access aliases per snippet type:
```toml
[snippet_types.shell]
alias = ","    # Use as: , mysnippet
```

2. **Smart Tab Completion**: 
- Works with both full commands and aliases
- Supports fuzzy matching
- Shows preview window with snippet content
- Remembers last used selections

Example usage:
```bash
# Using alias
, back<tab>  # Fuzzy finds 'backup' snippet

# Using full command
rsnip copy --ctype shell --input back<tab>
```

3. **Interactive Selection**: 
- FZF-style interface
- Live preview
- Fuzzy search
- Vim-style navigation

### Template System

RSnip implements a powerful template engine with:

1. **Built-in Filters**:
```
# Date formatting
{{ current_date|strftime('%Y-%m-%d') }}

# Date arithmetic
{{ current_date|add_days(7) }}
{{ current_date|subtract_days(7) }}

# Safe shell execution
{{ 'git rev-parse --short HEAD'|shell }}
```

2. **Environment Variables**:
```
{{ env_HOME }}     # Access $HOME
{{ env_USER }}     # Access $USER
{{ env_PATH }}     # Access $PATH
```

3. **Dynamic Content**:
```
--- git-commit
: Create a dated commit
git commit -m "Update: {{ current_date|strftime('%Y-%m-%d') }} - {{ 'git status -s|wc -l'|shell }} files"
---
```

### Command Reference

```bash
USAGE:
    rsnip [FLAGS] [OPTIONS] [SUBCOMMAND]

FLAGS:
    -d, --debug             Enable debug logging
        --generate-config   Print default configuration
        --info             Show version and config info
    -h, --help             Show help
    -V, --version          Show version

OPTIONS:
        --generate <SHELL>  Generate shell completion

SUBCOMMANDS:
    types      List snippet types
    list       Show all snippets
    edit       Edit snippets
    complete   Find/search snippets
    copy       Copy to clipboard
```

### Debug Support

Multiple verbosity levels for troubleshooting:
```bash
rsnip -d    # Info level
rsnip -dd   # Debug level
rsnip -ddd  # Trace level
```

View system information:
```bash
rsnip --info
```

## 🔍 Usage Examples

### Managing Shell Commands

1. Create shell snippets:
```
--- aws-profile
: Switch AWS profile
export AWS_PROFILE={{ env_AWS_PROFILE|default('default') }}
---

--- docker-clean
: Remove unused Docker resources
docker system prune -af
---
```

2. Use with aliases:
```bash
, aws<tab>     # Fuzzy finds aws-profile
,d clean<tab>  # Finds docker-clean using docker alias
```

### Git Workflows

1. Create git snippets:
```
--- commit-wip
: Create WIP commit with date
git commit -m "WIP: {{ current_date|strftime('%Y-%m-%d %H:%M') }}"
---
```

2. Use with dedicated alias:
```bash
,g wip<tab>  # Finds and applies commit-wip
```

## 🤝 Contributing

Contributions welcome! Please check our [Contributing Guide](CONTRIBUTING.md).

### Development Setup

```bash
# Clone repository
git clone https://github.com/yourusername/rsnip
cd rsnip

# Run tests
cargo test

# Build release
cargo build --release
```

## 📄 License

BSD 3-Clause License - see [LICENSE](LICENSE) for details.

## 🙏 Acknowledgments

Built with excellent Rust crates:
- clap: Command line parsing
- minijinja: Template engine
- skim: Fuzzy finder
- anyhow/thiserror: Error handling
- crossterm: Terminal UI


### Similar Work
[GitHub - knqyf263/pet: Simple command-line snippet manager](https://github.com/knqyf263/pet)
