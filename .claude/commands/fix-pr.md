---
description: Fix PR review feedback with TDD
allowed-tools: Bash, Read, Write, Grep, Glob, Task, AskUserQuestion
---

# Fix PR: $ARGUMENTS

```bash
PR=$ARGUMENTS
WS=".claude/workspace"
```

## 1. Context

If missing/stale: run `/fetch pr $PR` first.

```bash
cat "$WS/state-pr-$PR.md"
cat "$WS/unresolved-pr-$PR.json" | jq -r '.[] | "- \(.path):\(.line) [\(.reviewer)]: \(.comment[0:100])..."'
[ -f "$WS/handoff-pr-$PR.md" ] && cat "$WS/handoff-pr-$PR.md"
```

## 2. Analyze

For each unresolved comment:
1. What is the request?
2. What's the concern? (Correctness / Performance / Security / Style)
3. What could break?
4. Risk level?

Update `$WS/state-pr-$PR.md` with analysis.

**Think hard. Do not proceed until you understand each request.**

## 3. Plan (APPROVAL REQUIRED)

Present plan:
| # | Location | Request | Fix | Risk |
|---|----------|---------|-----|------|

**Use AskUserQuestion to get approval.**

## 4. Implement

Baseline first:
```bash
cargo check -p <crate> && cargo clippy -p <crate> -- -D warnings && cargo test -p <crate> --lib
```

For each fix:
1. Write/update test (should fail)
2. Make minimal change (match existing style)
3. Verify: `cargo check && cargo clippy && cargo test`
4. Log to state file

## 5. Final

```bash
cargo check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo +nightly fmt --check
cargo test -p <crate>
```

## 6. Report

| # | Comment | Resolution | Verified |
|---|---------|------------|----------|

Do not commit unless asked.
