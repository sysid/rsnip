## rsnip

# Smart Snippet Management with Template Support

A powerful command-line snippet manager that helps organize, find, and reuse text snippets with advanced templating capabilities. It features fuzzy search, intelligent shell integration, and dynamic template rendering.

[![Crates.io](https://img.shields.io/crates/v/rsnip)](https://crates.io/crates/rsnip)
[![License](https://img.shields.io/crates/l/rsnip)](LICENSE)

[Fast, Reliable, Yours: Why Snippets Outshine LLMs for boring Tasks](https://sysid.github.io/rsnip/)

## 🌟 Key Features

- **Smart Organization**: Categorize snippets into types (shell commands, code, notes, etc.)
- **Fuzzy Search**: Lightning-fast fuzzy finding with interactive fzf-style interface
- **Deep Shell Integration** (Inspired by [zoxide](https://github.com/ajeetdsouza/zoxide)):
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

[![asciicast](https://asciinema.org/a/699818.svg)](https://asciinema.org/a/699818)

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
`~/.config/rsnip/config.toml`

### Snippet Format

Snippets use a clear, readable format:

```
: Optional file-level comments

--- snippet_name
: Comment describing the snippet (optional)
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
For every alias an associated "edit" alias will be generated automatically (prefix e): `e,`.

2. **Smart Tab Completion**:
- Works with both full command and aliases
- Supports fuzzy matching
- Shows preview window with snippet content

Example usage:
```bash
# Using alias
, back<tab>   # Fuzzy finds 'backup' snippet and copies to clipboard
e, back<tab>  # Fuzzy finds 'backup' snippet and edits it

# Using full command
rsnip copy --ctype shell --input back<tab>
rsnip edit --ctype shell --input back<tab>
```

3. **Interactive Selection**: 
- FZF-style interface
- Live preview
- Fuzzy search
- Vim-style navigation

### Template System

RSnip implements a template engine with:

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

4. **Snippets to be handled as Literal Text**:

If you have snippets which happen to contain Jinja2-style template syntax, you can escape them like:
```
{% raw %}
gh run list --workflow "$workflow" \
    --status success --json name,startedAt,headBranch,databaseId,status \
    --template '{{range .}}{{tablerow (autocolor "white+h" .name) (autocolor "blue+h" .startedAt) .headBranch (autocolor "cyan" .databaseId) (autocolor "grey+h" .status)}}{{end}}' \
    --limit 20
{% endraw %}
```

### Command Reference

```bash
A universal command-line snippet manager

Usage: rsnip [OPTIONS] [COMMAND]

Commands:
  types     List available snippet types
  list      List all snippets
  edit      Edit snippet in system editor
  complete  Find completions with optional interactive selection
  copy      Copy text to clipboard

Options:
  -d, --debug...              Enable debug logging. Multiple flags (-d, -dd, -ddd) increase verbosity
      --generate <GENERATOR>  Generate shell completion scripts [possible values: bash, elvish, fish, powershell, zsh]
      --generate-config       Print default configuration to stdout
      --info                  Display version and configuration information
  -h, --help                  Print help
  -V, --version               Print version

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

Shell integration inspired by:
[GitHub - ajeetdsouza/zoxide: A smarter cd command. Supports all major shells.](https://github.com/ajeetdsouza/zoxide)
