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

### Correctness
- [ ] Logic traced step-by-step — can you walk through execution mentally?
- [ ] Boundary conditions handled: zero, empty, max, off-by-one
- [ ] Error handling correct; no panics possible in production paths
- [ ] No race conditions or ordering assumptions
- [ ] State transitions valid from all reachable states

### Crypto
- [ ] Field operations safe (no overflow, proper modular arithmetic)
- [ ] Checked arithmetic used where needed
- [ ] Randomness sourced appropriately (not predictable, sufficient entropy)
- [ ] No timing side-channels (constant-time where required)
- [ ] Circuit constraints deterministic and complete

### Memory & Performance
- [ ] No unnecessary allocations in hot paths
- [ ] Pre-allocation with `with_capacity` where size known
- [ ] No unnecessary `.clone()` — prefer references
- [ ] Iterators used efficiently; no intermediate collections
- [ ] No O(n²) or worse hidden in loops

### Security
- [ ] Input validation at trust boundaries
- [ ] No information leakage in error messages
- [ ] Fail-closed (reject on uncertainty)
- [ ] Backwards compatible (no consensus forks)

## Deep Analysis Techniques

When reviewing complex changes:

### Trace Data Flow
1. Identify all inputs (function args, global state, config)
2. Follow each input through transformations
3. Verify constraints are checked before use
4. Verify outputs match expected invariants

### Enumerate Failure Modes
For each operation, ask:
- What if this is zero/empty/max?
- What if this fails/returns error?
- What if this is called twice? Out of order?
- What if an attacker controls this input?

### Check Boundaries
- [ ] Array/slice indices always in bounds
- [ ] Arithmetic never overflows (or overflow is intentional)
- [ ] Casts don't truncate unexpectedly
- [ ] Loop bounds can't be manipulated

### Verify Invariants
Identify what must always be true:
- Before function entry
- After function exit
- Between related data structures

If an invariant can be violated, it's a bug.
