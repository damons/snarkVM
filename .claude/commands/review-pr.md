---
description: Security-focused PR review
allowed-tools: Bash, Read, Write, Grep, Glob, Task
---

# Review: $ARGUMENTS

Assume there is a bug. Find it.

```bash
PR=$ARGUMENTS
WS=".claude/workspace"
```

## 1. Context

If missing/stale: run `/fetch pr $PR` first.

Read state and files:
```bash
cat "$WS/state-pr-$PR.md"
cat "$WS/files-pr-$PR.txt"
```

## 2. Triage

Categorize files by risk:
- **High**: consensus/, synthesizer/vm/, circuit/, crypto, validation
- **Medium**: synthesizer/, core types, serialization
- **Low**: tests, docs

Update `$WS/state-pr-$PR.md` with risk levels. For large PRs (30+ files), focus on high-risk areas.

## 3. Understand

Before code analysis, answer:
- What problem does this solve?
- What invariants must hold?
- What could go wrong?

## 4. Analyze

For each file:
1. Read full file (not just diff) for context
2. Trace logic step-by-step
3. Check boundaries (zero, empty, max)
4. **Write findings to state file immediately**
5. Move on (previous content no longer needed)

Fetch diffs selectively: `gh pr diff $PR -- path/to/file.rs`

Apply AGENTS.md checklists: Correctness, Crypto, Memory, Security.

## 5. Verify

```bash
cargo check -p <crate>
cargo clippy -p <crate> -- -D warnings
cargo test -p <crate>
```

## 6. Report

Re-read `$WS/state-pr-$PR.md`, then output:

| Sev | Location | Issue | Fix |
|-----|----------|-------|-----|

**Severities**: BLOCKER (must fix) / BUG (should fix) / ISSUE (quality) / NIT (style)

**Recommendation**: Approve / Request changes / Needs discussion

## 7. Handoff (if needed)

Write `$WS/handoff-pr-$PR.md` with required fixes for `/fix-pr`.
