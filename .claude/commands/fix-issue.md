---
description: Fix GitHub issue with TDD
allowed-tools: Bash, Read, Write, Grep, Glob, Task, AskUserQuestion
---

# Fix: $ARGUMENTS

```bash
ISSUE=$ARGUMENTS
WS=".claude/workspace"
```

## 1. Context

If missing/stale: run `/fetch issue $ISSUE` first.

```bash
cat "$WS/state-issue-$ISSUE.md"
cat "$WS/comments-issue-$ISSUE.jsonl" | jq -r '"[\(.author.login)]: \(.body[0:150])..."' | head -10
```

## 2. Investigate

Search for related code:
```bash
git grep -n "relevant_term" -- "*.rs" | head -30
```

Answer:
- Can you reproduce?
- Expected vs actual behavior?
- Where does the code path go wrong?

Update `$WS/state-issue-$ISSUE.md` with root cause and evidence.

**Think hard. Do not proceed until you can explain the root cause.**

## 3. Plan (APPROVAL REQUIRED)

Present:
- **Root cause**: [specific]
- **Fix**: [specific changes]
- **Files**: path/to/file.rs — change X to Y
- **Tests**: test_name — verifies Z
- **Risk**: Low/Med/High

**Use AskUserQuestion to get approval.**

## 4. Implement

Baseline first:
```bash
cargo check -p <crate> && cargo clippy -p <crate> -- -D warnings && cargo test -p <crate> --lib
```

1. Write failing test
2. Make minimal fix (match existing style)
3. Verify test passes
4. Log to state file

## 5. Final

```bash
cargo check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo +nightly fmt --check
cargo test -p <crate>
```

## 6. Report

**Issue**: #$ISSUE — [title]
**Root cause**: [brief]
**Fix**: [what changed]
**Test**: [what it verifies]

Do not commit unless asked.
