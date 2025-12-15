# Pre-commit Setup for json_formatter

This project has pre-commit hooks configured to run checks specifically for the `examples/json_formatter` directory.

## Installation

1. Install pre-commit:
   ```bash
   # Using pip
   pip install pre-commit
   
   # Or using your package manager
   # Arch Linux
   sudo pacman -S pre-commit
   
   # macOS
   brew install pre-commit
   ```

2. Install the git hooks:
   ```bash
   # From the project root directory
   pre-commit install
   ```

## What Gets Checked

The pre-commit hooks will run the following checks on files in `examples/json_formatter/`:

- **cargo fmt**: Checks Rust code formatting
- **cargo clippy**: Runs linting checks
- **cargo check**: Verifies the code compiles

## Usage

### Automatic (on git commit)

Once installed, the hooks will run automatically when you commit changes to files in `examples/json_formatter/`:

```bash
git add examples/json_formatter/src/main.rs
git commit -m "Update json_formatter"
# Pre-commit hooks will run automatically
```

### Manual Run

You can also run the hooks manually:

```bash
# Run on all files
pre-commit run --all-files

# Run on staged files only
pre-commit run

# Run a specific hook
pre-commit run cargo-fmt-json-formatter
```

### Bypass Hooks (when needed)

If you need to commit without running the hooks:

```bash
git commit --no-verify -m "Your commit message"
```

## Troubleshooting

If a hook fails:

1. **cargo fmt fails**: Run `cd examples/json_formatter && cargo fmt` to auto-fix formatting
2. **cargo clippy fails**: Fix the warnings/errors shown in the output
3. **cargo check fails**: Fix the compilation errors

## Updating Hooks

To update pre-commit hooks to the latest version:

```bash
pre-commit autoupdate
```
