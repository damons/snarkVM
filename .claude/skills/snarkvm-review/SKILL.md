---
name: snarkvm-review
description: |
  Security-focused PR review for snarkVM codebase.
  WHEN: User says "review PR", "audit PR", "security review", "check PR changes",
  or wants thorough analysis of PR changes for bugs/vulnerabilities.
  WHEN NOT: Fixing review feedback (use snarkvm-fix-pr), fetching context only
  (use snarkvm-github), or fixing issues (use snarkvm-fix).
context: fork
agent: general-purpose
allowed-tools: Bash, Read, Write, Grep, Glob, Task
disable-model-invocation: true
---

# Security-Focused PR Review

**Mindset: Assume there is a bug. Find it.**

This skill runs in a forked context to keep exploration out of your main conversation.

## Usage

```
/snarkvm-review <pr_number>
```

## Setup

```bash
PR=$ARGUMENTS
WS=".claude/workspace"
SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../snarkvm-github" && pwd)"
```

## 1. Load Context

Ensure PR context exists:

```bash
if [[ ! -f "$WS/state-pr-$PR.md" ]]; then
  echo "Context missing. Fetching PR #$PR..."
  "$SKILL_DIR/scripts/fetch-pr.sh" "$PR"
fi
```

Read initial state:

```bash
cat "$WS/state-pr-$PR.md"
cat "$WS/files-pr-$PR.txt"
```

## 2. Triage by Risk

Categorize changed files:

| Risk | Directories | Rationale |
|------|-------------|-----------|
| **HIGH** | `consensus/`, `synthesizer/vm/`, `circuit/`, `ledger/block/`, `ledger/store/`, validation logic | Consensus-critical, can cause forks or invalid state |
| **HIGH** | `algorithms/`, `curves/`, `fields/`, crypto primitives | Cryptographic correctness, side-channel risks |
| **MEDIUM** | `synthesizer/` (non-vm), `console/`, core types | Can cause program failures, data corruption |
| **MEDIUM** | Serialization (`FromBytes`, `ToBytes`), parsing | Can cause incompatibilities, DoS |
| **LOW** | `utilities/`, `wasm/`, `metrics/` | Support code, lower blast radius |
| **LOW** | Tests, docs, CI config | Non-runtime code |

Count files per risk level and update `$WS/state-pr-$PR.md`.

**For large PRs (30+ files):** Focus on HIGH risk areas first. Consider parallel analysis.

## 3. Understand Intent

Before diving into code, answer:

1. **What problem does this PR solve?**
   - Read PR description: `jq -r .body "$WS/context-pr-$PR.json"`
   - Check linked issues: `cat "$WS/linked-issues-pr-$PR.txt"`

2. **What invariants must hold?**
   - Consensus rules?
   - Cryptographic properties?
   - API contracts?

3. **What could go wrong?**
   - Edge cases (zero, empty, max)?
   - Concurrent access?
   - Resource exhaustion?

## 4. Analyze Code

### For small/medium PRs (< 30 files)

Sequential analysis. For each HIGH/MEDIUM risk file:

1. **Read the full file** (not just diff) for context
2. **Trace the logic** step-by-step
3. **Check boundaries**: zero, empty, max, overflow
4. **Write findings to `$WS/state-pr-$PR.md` immediately**
5. **Release file from working memory** and move on

Fetch diffs selectively:
```bash
gh pr diff $PR -- path/to/file.rs
```

### For large PRs (30+ HIGH/MEDIUM files)

Use **parallel subagents** for faster analysis:

**Spawn Task subagents by category:**

```
Use the Task tool to spawn parallel subagents:

Task 1 (Consensus): Analyze files in ledger/block/, ledger/store/, consensus/
  - Check: State transitions, validation logic, fork handling
  - Return: Findings table

Task 2 (Crypto): Analyze files in algorithms/, curves/, fields/
  - Check: Constant-time operations, field arithmetic, randomness
  - Return: Findings table

Task 3 (Synthesizer): Analyze files in synthesizer/, circuit/
  - Check: Constraint generation, witness computation, program execution
  - Return: Findings table
```

Each subagent should:
- Read assigned files completely
- Apply relevant checks from section 4.1
- Return findings in table format
- Note anything requiring cross-file analysis

Merge results into `$WS/state-pr-$PR.md`.

### 4.1 Review Checklist

**Correctness:**
- [ ] Logic matches intent from PR description
- [ ] Edge cases handled (zero, empty, max values)
- [ ] Error paths return appropriate errors
- [ ] Unwrap/expect only used where truly infallible
- [ ] Integer arithmetic checked for overflow
- [ ] Array/slice access bounds-checked

**Crypto (if applicable):**
- [ ] No secret-dependent branches or memory access
- [ ] Randomness from secure source (OsRng)
- [ ] Field operations handle zero/identity correctly
- [ ] No timing side-channels in comparisons
- [ ] Serialization preserves cryptographic properties

**Memory/Resources:**
- [ ] No unbounded allocations from untrusted input
- [ ] Large allocations use try_reserve or similar
- [ ] Sensitive data zeroized after use
- [ ] No reference cycles causing leaks

**Consensus (if applicable):**
- [ ] Deterministic execution (no HashMap iteration order issues)
- [ ] Backward compatible serialization
- [ ] State transitions validated before apply
- [ ] No panics in consensus-critical paths

## 5. Verify Build

```bash
"$SKILL_DIR/scripts/cargo-validate.sh"
```

Or selectively:
```bash
CRATES=$(cut -d' ' -f2 "$WS/files-pr-$PR.txt" | "$SKILL_DIR/scripts/detect-crates.sh")
for crate in $CRATES; do
  cargo check -p "$crate"
  cargo clippy -p "$crate" -- -D warnings
  cargo test -p "$crate"
done
```

## 6. Report

Re-read `$WS/state-pr-$PR.md` to compile findings.

**Output findings table:**

| Sev | Location | Issue | Suggested Fix |
|-----|----------|-------|---------------|
| BLOCKER | path/file.rs:42 | Unchecked overflow in... | Use checked_add or saturating_add |
| BUG | path/file.rs:100 | Missing bounds check | Add length validation |
| ISSUE | path/file.rs:200 | Inefficient clone | Use reference instead |
| NIT | path/file.rs:300 | Inconsistent naming | Rename to match convention |

**Severity guide:**
- **BLOCKER** — Must fix before merge. Correctness/security issue.
- **BUG** — Should fix. Likely causes problems.
- **ISSUE** — Quality concern. Consider fixing.
- **NIT** — Style/preference. Optional.

**Final recommendation:**
- **Approve** — No blockers, code is sound
- **Request changes** — Blockers or bugs require attention
- **Needs discussion** — Design concerns to resolve

## 7. Handoff (if changes needed)

If requesting changes, create handoff for `/snarkvm-fix-pr`:

```bash
TEMPLATE_DIR="$SKILL_DIR/templates"
sed -e "s/{{NUM}}/$PR/g" "$TEMPLATE_DIR/handoff.md" > "$WS/handoff-pr-$PR.md"
```

Then edit `$WS/handoff-pr-$PR.md` to fill in the required fixes table.

## Memory: Common Vulnerability Patterns

**In snarkVM specifically:**
- **Field arithmetic**: Division by zero, modular reduction edge cases
- **Serialization**: Length prefix overflow, malformed input handling
- **Circuit constraints**: Under-constrained witnesses, missing range checks
- **State transitions**: TOCTOU in validation, missing uniqueness checks
- **Resource limits**: Unbounded loops from malicious programs, stack overflow in recursion
