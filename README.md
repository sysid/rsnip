# rsnip

A powerful command-line snippet manager with fuzzy search capabilities, written in Rust.

## Features

- 🔍 Fuzzy search with interactive selection
- 📋 Direct clipboard integration
- ⚡ Fast and memory efficient
- 🛠️ Shell completion support (bash)
- 📝 System editor integration ($EDITOR)

## Installation

### From Source

```bash
git clone https://github.com/yourusername/rsnip.git
cd rsnip
cargo install --path .
```

### Configuration

rsnip looks for configuration in the following locations (in order of precedence):
1. `~/.config/rsnip/config.toml`
2. `/etc/rsnip/config.toml`

Example configuration:

```toml
[snippet_types.general]
source_file = "~/.config/rsnip/general_snippets.txt"
description = "General text snippets"

[snippet_types.shell]
source_file = "~/.config/rsnip/shell_snippets.txt"
description = "Shell command snippets"
```

## Usage

### Basic Commands

```bash
# Show help
rsnip --help

# List available snippet types
rsnip types

# Edit snippets (opens in your $EDITOR)
rsnip edit --ctype general

# Find completions interactively
rsnip complete --ctype general --interactive

# Copy a snippet to clipboard
rsnip copy --ctype general --input "my-snippet"

# Show version and configuration info
rsnip --info
```

### Debug Levels

Use `-d` flags to increase debug verbosity:
- `-d`: Info level
- `-dd`: Debug level
- `-ddd`: Trace level

### Shell Completion

To enable shell completion for bash:

```bash
source /path/to/rsnip.bash
```

## Snippet File Format

Snippets are stored in a simple text format:

```text
--- snippet-name
This is the content of the snippet
It can span multiple lines
---

--- another-snippet
Single line content
---
```

## Development

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Building

```bash
cargo build --release
```

### Running Tests

```bash
make test
```

### Debug Logging

Set the `RUST_LOG` environment variable for additional debug output:

```bash
RUST_LOG=debug cargo run
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the BSD 3 License - see the LICENSE file for details.