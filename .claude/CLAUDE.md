# snarkVM

## Git
Never commit. Stage with `git add` if asked.

## Before Writing Code
- Search for existing implementations
- Read the target module and match its patterns exactly
- If uncertain, ask
- Think very hard in your planning process.

## Prohibited Without Approval
New files, crates, dependencies, abstractions, traits, error types, or refactoring outside the task.

## Validation (in order)
```bash
cargo check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo fmt --check
cargo test -p <crate>
```


## Code and Patterns
- `unwrap`s should be commented with justification
- Use high performance Rust patterns
- You must always use test driven development

## Style
Rigid. No deviation. Run `cargo fmt` before every check-in.

```
- One blank line between functions
- No trailing whitespace
- Imports: std first, external crates second, crate-local third
- Match existing file exactly — if the file uses `Self::`, you use `Self::`
- Comments must be concise, complete, punctuated sentences
- Variable names should be descriptive
- Each logical component of the code should be commented

## Files
- License header required (enforced by `build.rs`)
- `#![forbid(unsafe_code)]` unless approved
