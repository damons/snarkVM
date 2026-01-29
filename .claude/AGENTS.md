# snarkVM

## Before Writing Code
- Search for existing implementations.
- Read the target module and match its patterns exactly.
- Scope out necessary tests ahead of time.
- If uncertain, ask.
- Think very hard in your planning process.
- New files, crates, dependencies, abstractions, traits, or error types require approval.

## Architecture
- All changes must be backwards compatible and must not introduce forks.
- New features may be gated by `ConsensusVersion` if needed.
- Validation logic should exist in one place per layer.
- Trait impls (ToBits, FromBits, ToField, FromField) should follow existing patterns.

## Crates

**console / circuit**
- These layers must stay in sync — same structure, same API surface.
- Circuits must be deterministic.
- Test circuit equivalence by comparing constraint counts.
- When modifying one layer, check if the other needs the same change.

**synthesizer**
- Tests are slow. Run only the relevant test module or function.
- Use `--features test,dev_println` for integration tests.

## Code and Patterns
- Test-driven development: write failing tests first.
- `unwrap`s must be commented with justification.
- Pre-allocate with `with_capacity` when final size is known.
- Prefer arrays/slices over `Vec` when size is known at compile time.
- Use iterators; avoid intermediate vectors and unnecessary `.collect()`.
- Prefer references and `into_iter()` over `.clone()` and `iter().cloned()`.

See @CONTRIBUTING.md for detailed memory and performance guidelines.

## Validation

Run in order:
```bash
cargo check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo +nightly fmt --check
cargo test -p <crate>
```

Clippy warnings are errors. Formatting requires nightly (`cargo +nightly fmt --all` to fix).

## Git
- Never commit unless explicitly asked.
- Stage with `git add` only if requested.
- Pre-commit hooks run fmt and clippy. Run `cargo +nightly fmt --all` before staging.

## Style
- One blank line between functions.
- No trailing whitespace.
- Imports: std first, external crates second, crate-local third.
- Match existing file patterns exactly.
- Comments must be concise, complete, punctuated sentences.
- License header required (enforced by `build.rs`).
- `#![forbid(unsafe_code)]` unless approved.

## Review Checklist

**Correctness**
- Logic traced step-by-step.
- Boundary conditions handled: zero, empty, max.
- Error handling correct; no panics possible.
- No race conditions.

**Crypto**
- Field operations safe (no overflow, proper modular arithmetic).
- Checked arithmetic used.
- Randomness sourced appropriately.
- No timing side-channels.

**Performance**
- See Code and Patterns above.
