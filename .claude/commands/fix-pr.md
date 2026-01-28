---
description: Fix PR based on review feedback with TDD and security focus
allowed-tools: Bash, Read, Write, Grep, Glob, Task, AskUserQuestion
---

# Fix PR: $ARGUMENTS

**IMPORTANT**: This command has a mandatory approval gate at Step 3. You MUST use the AskUserQuestion tool and wait for explicit user approval before implementing any changes.

Follow CLAUDE.md for validation commands and code patterns.

## 1. Gather Context
- [ ] `gh pr view --json title,body,state,isDraft,mergeable,statusCheckRollup,url,author,assignees,reviewRequests,reviews,labels,milestone,createdAt,additions,deletions,changedFiles,headRefName,baseRefName,commits`
- [ ] `gh pr diff $ARGUMENTS`
- [ ] List all unresolved review comments with full context
- [ ] Read complete files affected by comments (not just snippets)
- [ ] Search codebase for related patterns and prior art

## 2. Deep Analysis (Think Very Hard)
For each unresolved comment, analyze thoroughly:

### 2.1 Understand the Feedback
- [ ] What exactly is the reviewer asking for?
- [ ] What is the underlying concern (correctness, performance, style, security)?
- [ ] Is there implicit context or history I need to understand?

### 2.2 Evaluate Validity
- [ ] Is the feedback valid? Why or why not?
- [ ] Are there edge cases the reviewer identified that I missed?
- [ ] Does this expose a deeper architectural issue?

### 2.3 Explore Solutions
- [ ] What are ALL the possible ways to address this?
- [ ] What are the tradeoffs of each approach?
- [ ] Which approach best matches existing codebase patterns?
- [ ] Are there security implications to any approach?

### 2.4 Assess Risk
- [ ] What could break if I make this change?
- [ ] Are there downstream dependencies affected?
- [ ] Does this change require updates elsewhere for consistency?

## 3. Comprehensive Plan (REQUIRES APPROVAL - MANDATORY STOP)

Present a detailed plan with full justification:

### Summary
Brief overview of all changes to be made.

### Detailed Changes

For each comment, present:

| # | Location | Reviewer Request | Root Cause Analysis | Proposed Solution | Justification | Risk Assessment |
|---|----------|------------------|---------------------|-------------------|---------------|-----------------|
| 1 | file:line | ... | ... | ... | ... | Low/Med/High |

### Security Considerations
- [ ] Any security implications identified
- [ ] Mitigation strategies if applicable

### Testing Strategy
- [ ] What tests need to be added or modified?
- [ ] How will correctness be verified?

### Alternative Approaches Considered
For non-trivial changes, list alternatives and why they were rejected.

---

### MANDATORY APPROVAL GATE

After presenting the plan above, you MUST use the **AskUserQuestion** tool with this message:

"I have analyzed the PR feedback and prepared a comprehensive plan above.

Please review and respond with one of:
- **Approve**: Proceed with all proposed changes
- **Approve with modifications**: Specify which items to change
- **Skip [#]**: Skip specific items
- **Reject**: Do not proceed

I will not make any changes until you approve."

**DO NOT proceed to Step 4 until you have:**
1. Used the AskUserQuestion tool
2. Received the user's response
3. Confirmed the response is "Approve" or "Approve with modifications"

If the user responds with "Reject", stop execution entirely.

---

## 4. Baseline Verification (ONLY AFTER APPROVAL)
- [ ] Run full validation on affected crate(s)
- [ ] `cargo check -p <crate>`
- [ ] `cargo clippy -p <crate> -- -D warnings`
- [ ] `cargo +nightly fmt --check`
- [ ] `cargo test -p <crate>` (relevant tests only for speed)
- [ ] All checks must pass before making any changes

## 5. Test-Driven Implementation
For each approved fix:
- [ ] Write or update test first (if applicable)
- [ ] Make the minimal change to address the feedback
- [ ] `cargo check -p <crate>`
- [ ] `cargo clippy -p <crate> -- -D warnings`
- [ ] Verify test passes
- [ ] Proceed to next fix only when green

## 6. Final Verification
- [ ] `cargo +nightly fmt --all`
- [ ] `cargo check -p <crate>`
- [ ] `cargo clippy -p <crate> -- -D warnings`
- [ ] `cargo test -p <crate>`
- [ ] All checks pass

## 7. Summary Report

| # | Comment | Resolution | Verification |
|---|---------|------------|--------------|
| 1 | ... | Implemented / Skipped / N/A | Tests pass |

### Changes Made
- List each file modified and what changed

### Remaining Items
- Any comments that were skipped or deferred

Do not commit. Stage with `git add` only if explicitly requested.
