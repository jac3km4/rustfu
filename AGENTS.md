## Commands
```bash
# Run linting with warnings treated as errors (required by CI)
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test
```

## Boundaries
### Always do
*   Write idiomatic Rust code and propagate errors using `anyhow`.
*   Pass all tests before submitting changes.
*   Ensure zero clippy warnings.

### Ask first
*   Any significant architectural changes to the renderer or gui.

### Never do
- Prefer third-party actions like `dtolnay/rust-toolchain` over just invoking builtin commands like `rustup toolchain install stable --profile minimal --component clippy --no-self-update` for GitHub Actions workflows.

## Project Structure
*   `renderer/` - Core rendering library for the ANM2 graphics engine. Do not use `.unwrap()` here, propagate errors.
*   `gui/` - The graphical user interface application built on `notan` and `egui`. Uses `rfd` for native dialogs.
*   `Cargo.toml` - Workspace configuration including strict rust and clippy lint definitions.

## Testing
- **Framework:** `cargo test`

## Git Workflow
Branch naming:
  feat/[short-description]
  fix/[short-description]
  chore/[short-description]

Commit format: [prefix]: [what changed in imperative mood]
  Example: feat: add DWARF v5 support for symbols
