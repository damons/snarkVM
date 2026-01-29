---
description: Fix PR based on review feedback with TDD and security focus
allowed-tools: Bash, Read, Write, Grep, Glob, Task, AskUserQuestion
---

# Fix PR: $ARGUMENTS

Follow CLAUDE.md for validation commands and code patterns.

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
- [ ] Read complete files affected by comments
- [ ] Search codebase for related patterns

## 2. Analyze

For each unresolved comment:
- [ ] What is the reviewer asking for?
- [ ] What is the underlying concern (correctness, performance, security)?
- [ ] What are the possible solutions and tradeoffs?
- [ ] What could break? Any downstream dependencies?

Think very hard.

## 3. Plan (APPROVAL REQUIRED)

Present for each comment:

| # | Location | Request | Proposed Fix | Risk |
|---|----------|---------|--------------|------|
| 1 | file:line | ... | ... | Low/Med/High |

Use **AskUserQuestion** to get approval before proceeding.

## 4. Implement

For each approved fix:
- [ ] Baseline: all validation passes before changes
- [ ] Write or update test first
- [ ] Make minimal change
- [ ] Verify: all validation passes

## 5. Report

| # | Comment | Resolution | Verification |
|---|---------|------------|--------------|
| 1 | ... | Implemented / Skipped | Tests pass |

Do not commit.
