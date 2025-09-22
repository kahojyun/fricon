# Fricon Code Conventions

## General Style
- **Indentation**: 4 spaces for most files, 2 spaces for frontend/proto files
- **Line endings**: LF (Unix-style)
- **Encoding**: UTF-8
- **Final newlines**: Required
- **Trailing whitespace**: Trimmed (except in markdown)

## Rust Conventions
- **Edition**: 2024
- **Line length**: 100 characters max
- **Formatting**: rustfmt with custom configuration:
  - `wrap_comments = true`
  - `format_strings = true`
  - `error_on_line_overflow = true`
  - `group_imports = "StdExternalCrate"`
  - `imports_granularity = "Crate"`
- **Linting**: Clippy in pedantic mode
  - `rust-2018-idioms = "warn"`
  - Most pedantic rules enabled
  - Allows: `missing_panics_doc`, `missing_errors_doc`

## Python Conventions
- **Type hints**: Required (using basedpyright for checking)
- **Docstrings**: Standard Python docstring conventions
- **Formatting**: ruff format
- **Linting**: ruff check, mypy, basedpyright
- **Imports**: Follow standard Python conventions

## Frontend Conventions
- **TypeScript**: Strict type checking enabled
- **Vue3**: Composition API with TypeScript
- **Indentation**: 2 spaces
- **Linting**: ESLint with TypeScript and Vue plugins
- **Formatting**: Prettier with default configuration

## Git & Commit Style
- **Branch naming**: feature/bugfix branches with clear names
- **Commit messages**: Conventional commits format
  - `type(scope): description`
  - Types: feat, fix, docs, style, refactor, test, chore
- **PR workflow**: Branch from main, include tests, run linters

## Testing
- **Rust**: cargo test with standard test organization
- **Python**: pytest with importlib mode
- **Frontend**: Standard testing framework for Vue/Tauri

## Documentation
- **Code comments**: Comprehensive documentation for public APIs
- **README**: Clear installation and usage instructions
- **API docs**: Generated documentation for Python package
