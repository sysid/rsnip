# Development Guide

## 🛠️ Development Setup
```bash
# Build debug version
cargo build

# Run tests
cargo test -- --test-threads=1  # Single thread required
```

### Local Development Workflow
1. Set up your environment:
```bash
# Add debug build to your PATH
export PATH="$PWD/rsnip/target/debug:$PATH"

# Enable shell completion for development
source <(rsnip --generate bash)
```

### Testing
```bash
# Run all tests
make test
```

Shell Integration Tests:
```bash
# Test completion mechanism
source <(rsnip --generate bash)
rsnip complete --ctype mytype --interactive --input app<TAB>
```

## Makefile Commands
The project includes a Makefile with useful commands:
```bash
make help
```

## 🔍 Development Guidelines

1. Code Style:
   - Follow Rust standard formatting (`cargo fmt`)
   - Use meaningful variable names
   - Add comments for complex logic
   - Follow the test naming convention: `given_when_then`

2. Testing:
   - Write tests for new features
   - Follow Arrange/Act/Assert pattern
   - Test error cases

3. Error Handling:
   - Use `anyhow` for error propagation
   - Define custom errors with `thiserror`
   - Provide meaningful error messages

4. Commit Guidelines:
   - Write clear commit messages
   - Keep commits focused
   - Reference issues when relevant

## 📝 Documentation
1. Generate and view documentation:
```bash
# Generate and open docs
cargo doc --open

# Generate docs with private items
cargo doc --document-private-items --open
```