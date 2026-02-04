---
name: snarkvm-fix
description: |
  Fix GitHub issues in snarkVM using TDD workflow.
  WHEN: User says "fix issue", "fix #123", "address issue", "implement fix for issue",
  or wants to resolve a bug/feature request from GitHub issues.
  WHEN NOT: Fixing PR review feedback (use snarkvm-fix-pr), doing security review
  (use snarkvm-review), or working on non-snarkVM code.
allowed-tools: Bash, Read, Write, Grep, Glob, Task, AskUserQuestion
---

# Fix GitHub Issue

Fix snarkVM GitHub issues using test-driven development.

## Usage

```
/snarkvm-fix <issue_number>
```

## Setup

```bash
ISSUE=$ARGUMENTS
WS=".claude/workspace"
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../snarkvm-github" && pwd)"
```

## 1. Load Context

Ensure issue context is loaded. If missing or stale, fetch it first:

```bash
if [[ ! -f "$WS/state-issue-$ISSUE.md" ]]; then
  echo "Context missing. Fetching issue #$ISSUE..."
  "$SKILL_DIR/scripts/fetch-issue.sh" "$ISSUE"
fi
```

Review the context:

```bash
cat "$WS/state-issue-$ISSUE.md"
echo "--- Recent comments ---"
cat "$WS/comments-issue-$ISSUE.jsonl" | jq -r '"[\(.author.login)]: \(.body[0:150])..."' | head -10
```

## 2. Investigate

Search for related code:

```bash
# Search for relevant terms from the issue
git grep -n "relevant_term" -- "*.rs" | head -30

# Find related files
fd -e rs "pattern" | head -20
```

Answer these questions:
1. Can you reproduce the issue?
2. What is the expected vs actual behavior?
3. Where does the code path go wrong?
4. What are the edge cases?

**Update `$WS/state-issue-$ISSUE.md`** with:
- Root cause analysis
- Relevant code locations
- Evidence/reproduction steps

**Think hard. Do not proceed until you can explain the root cause.**

## 3. Plan (APPROVAL REQUIRED)

Present a concrete plan:

| Aspect | Details |
|--------|---------|
| **Root cause** | [specific explanation] |
| **Fix** | [specific code changes] |
| **Files** | path/to/file.rs — change X to Y |
| **Tests** | test_name — verifies Z |
| **Risk** | Low / Medium / High |

**Use AskUserQuestion to get explicit approval before proceeding.**

## 4. Implement (TDD)

### 4.1 Establish baseline

Detect affected crates and verify current state:

```bash
# Auto-detect crates from investigation
CRATES=$("$SKILL_DIR/scripts/detect-crates.sh" <<< "path/to/affected/file.rs")

# Run baseline checks
for crate in $CRATES; do
  cargo check -p "$crate"
  cargo clippy -p "$crate" -- -D warnings
  cargo test -p "$crate" --lib
done
```

### 4.2 Write failing test first

Create a test that:
- Reproduces the issue
- Will pass once the fix is applied
- Covers edge cases identified in investigation

```rust
#[test]
fn test_issue_NNNN_description() {
    // Setup
    // Action  
    // Assert expected behavior
}
```

Verify the test fails:
```bash
cargo test -p <crate> test_issue_NNNN -- --nocapture
```

### 4.3 Implement minimal fix

- Match existing code style
- Make the smallest change that fixes the issue
- Add comments explaining non-obvious changes

### 4.4 Verify test passes

```bash
cargo test -p <crate> test_issue_NNNN -- --nocapture
```

### 4.5 Log progress

Update `$WS/state-issue-$ISSUE.md` with:

| Action | Result |
|--------|--------|
| Wrote test | test_issue_NNNN in path/to/test.rs |
| Applied fix | Changed X to Y in path/to/file.rs |
| Verified | Test passes |

## 5. Final Validation

Run full validation on affected crates:

```bash
"$SKILL_DIR/scripts/cargo-validate.sh" $CRATES
```

Or manually:

```bash
cargo check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo +nightly fmt --check
cargo test -p <crate>
```

## 6. Report

Summarize the fix:

```
**Issue**: #$ISSUE — [title]
**Root cause**: [brief explanation]
**Fix**: [what changed and why]
**Test**: [what the test verifies]
**Files changed**:
- path/to/file.rs — [change description]
- path/to/test.rs — [new test]
```

**Do not commit unless explicitly asked.**

## Memory: Key Patterns

When investigating snarkVM issues:

- **Consensus bugs**: Check `ledger/`, `synthesizer/vm/`, validation logic
- **Circuit issues**: Check `circuit/`, constraint generation, witness computation
- **Serialization bugs**: Check `FromBytes`/`ToBytes` implementations, ensure round-trip
- **Crypto issues**: Check `algorithms/`, `curves/`, `fields/` for field arithmetic edge cases
- **Parser bugs**: Check `synthesizer/parser/`, `console/program/` for grammar handling
