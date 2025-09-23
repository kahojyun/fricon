# Fricon Post-Task Checklist

## After Completing Any Code Changes

### 1. Code Quality Checks
```bash
# Rust code changes
cargo fmt --check                 # Verify formatting
cargo clippy --all-targets --all-features  # Run linter
cargo test                        # Run tests

# Python code changes
uv run ruff format --check         # Verify formatting
uv run ruff check                  # Run linter
uv run basedpyright               # Type checking
uv run pytest                     # Run tests

# Frontend code changes
pnpm run check                     # Lint and format check
```

### 2. Build Verification
```bash
# Build Rust components
cargo build                       # Ensure everything compiles

# Build Python extension
uv run maturin develop            # Rebuild Python bindings

# Test frontend build
pnpm tauri build --help           # Verify Tauri config works
```

### 3. Integration Testing
```bash
# Test workspace initialization
cargo run --bin fricon-cli -- workspace init test_workspace

# Test Python imports
uv run python -c "import fricon; print('Import successful')"

# Test GUI launch (if applicable)
pnpm tauri dev --help             # Verify GUI can start
```

### 4. Documentation Updates
- [ ] Update inline documentation for changed APIs
- [ ] Update README.md if behavior changes
- [ ] Update CONTRIBUTING.md if workflow changes
- [ ] Check docstring accuracy

### 5. Git Status Check
```bash
git status                        # Review staged/unstaged changes
git diff --staged                 # Review what will be committed
git add .                         # Stage all changes
```

## Before Creating a Pull Request

### 1. Final Quality Assurance
```bash
# Complete test suite
cargo test --all                  # Run all Rust tests
uv run pytest -v                  # Run all Python tests with verbose output

# Full linting suite
cargo clippy --all-targets --all-features -- -D warnings  # Treat warnings as errors
uv run ruff check --fix           # Auto-fix linting issues
uv run basedpyright --strict      # Strict type checking
```

### 2. Build Verification
```bash
# Release build test
cargo build --release             # Test release compilation

# Clean build test
cargo clean && cargo build        # Test from clean state
```

### 3. Documentation
- [ ] All new public APIs documented
- [ ] Examples updated if needed
- [ ] CHANGELOG.md updated for user-facing changes
- [ ] Breaking changes clearly documented

### 4. Commit Message Verification
- [ ] Follows conventional commit format: `type(scope): description`
- [ ] Clear description of changes
- [ ] Issue references included if applicable

## Environment-Specific Considerations

### Darwin/macOS Specific
- [ ] Test on macOS target if building platform-specific features
- [ ] Verify Tauri app can be bundled for macOS
- [ ] Check file path handling (macOS path sensitivity)

### Cross-Platform Considerations
- [ ] Test file paths work on Windows/Linux if applicable
- [ ] Verify database paths are handled correctly
- [ ] Check shell command compatibility
