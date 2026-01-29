---
description: Security-focused PR review
allowed-tools: Bash, Read, Write, Grep, Glob, Task
---

# Review: $ARGUMENTS

Assume there is a bug. Your job is to find it. Think very hard.

## 1. Gather

- [ ] `gh pr view $ARGUMENTS --json title,body,state,headRefName,baseRefName`
- [ ] `gh pr diff $ARGUMENTS`
- [ ] Fetch all review threads:
  ```bash
  gh api graphql -f query='
  {
    repository(owner: "ProvableHQ", name: "snarkVM") {
      pullRequest(number: $ARGUMENTS) {
        reviewThreads(first: 100) {
          nodes {
            isResolved
            comments(first: 10) {
              nodes { path line body author { login } }
            }
          }
        }
      }
    }
  }'
  ```
- [ ] Read full files (not just diff) for all modified code
- [ ] Read files that import/are imported by modified files
- [ ] Identify affected crates: `gh pr diff $ARGUMENTS --name-only`

## 2. Understand

- [ ] What problem is this solving?
- [ ] What is the approach?
- [ ] Write down your understanding before proceeding

## 3. Analyze

For each change, apply the **Review Checklist** from AGENTS.md (correctness, crypto, memory & performance).

## 4. Verify

- [ ] Run tests on affected crates
- [ ] If you cannot prove correctness, it's a bug

## 5. Report

**Format:** `[SEVERITY] file:line — Issue. Suggested fix.`

**Severities:** BLOCKER > BUG > ISSUE > NIT

| Severity | Location | Issue |
|----------|----------|-------|

**Recommendation:** Approve / Request changes / Reject

Do not commit.
