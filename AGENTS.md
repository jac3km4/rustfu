# AGENTS.md

## Commands
*   **Compile:** `cargo check`
*   **Test:** `cargo test`
*   **Run:** `cargo run -p rustfu-gui`
*   **Lint:** `cargo clippy --all-targets --all-features -- -D warnings`
*   **CI Toolchain Setup:** `rustup toolchain install stable --profile minimal --component clippy --no-self-update`

## Boundaries
### Always do
*   Write idiomatic Rust code and propagate errors using `anyhow`.
*   Pass all tests before submitting changes.
*   Ensure zero clippy warnings.
*   Use technical naming in documentation.

### Ask first
*   Any significant architectural changes to the renderer or gui.

### Never do
*   Never directly reference original Java/Wakfu/Ankama source code, class names, or specific variable names from the reference implementation in any documentation. Just explain the technical details.
*   Never use third-party actions like `dtolnay/rust-toolchain` for GitHub Actions; use the `rustup` command specified above.

## Project Structure
*   `renderer/` - Core rendering library for the ANM2 graphics engine. Do not use `.unwrap()` here, propagate errors.
*   `gui/` - The graphical user interface application built on `notan` and `egui`. Uses `rfd` for native dialogs.
*   `Cargo.toml` - Workspace configuration including strict rust and clippy lint definitions.

## Code Style
```rust
// Preferred: explicit error propagation using anyhow::Result
use anyhow::Result;

pub fn process_data(data: &[u8]) -> Result<ParsedData> {
    if data.is_empty() {
        anyhow::bail!("Data is empty");
    }
    // processing logic
    Ok(ParsedData { /* ... */ })
}
```

## Testing
*   **Framework:** `cargo test`
*   **Determinism:** Tests should run independently and deterministically.
*   **Coverage:** Ensure logic in `renderer` is well tested before checking.

## Git Workflow
*   **Commit format:** Ensure commits are descriptive and logical.
