# snarkVM

## Rules
- All changes must be backwards compatible — no consensus forks.
- New files, crates, dependencies, abstractions, or traits require approval.
- `unwrap`s must have a comment justifying why they can't panic.

## Crate-Specific
- **console / circuit**: Must stay in sync. Same structure, same API. When modifying one, check the other. Test circuit equivalence by comparing constraint counts.
- **synthesizer**: Tests are slow — run only the specific test function. Use `--features test,dev_println` for integration tests.

## Patterns
- Prefer `zip_eq` over `zip` when lengths must match.
- Pre-allocate with `with_capacity` when final size is known.
- Prefer `into_iter()` over `iter().cloned()`. Prefer references over `.clone()`.
- Use iterators; avoid intermediate `.collect()` when a single pass follows.
- Trait impls (ToBits, FromBits, ToField, FromField) should follow existing patterns in the same file.
- See @CONTRIBUTING.md for full memory/performance guidelines.

## Validation (BLOCKING)
All must pass before any task is complete:
```bash
cargo check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo +nightly fmt --check            # Requires nightly toolchain
cargo test -p <crate>
```
Pre-commit hook runs workspace-wide: `cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo +nightly fmt --all -- --check`

## Style
- Match existing file patterns exactly. One blank line between functions. No trailing whitespace.
- Imports: std first, external crates second, crate-local third.
- Comments: concise, complete, punctuated sentences.
- License header required (enforced by `build.rs`). `#![forbid(unsafe_code)]` unless approved.

## Git
Never commit unless asked. Run `cargo +nightly fmt --all` before staging.
