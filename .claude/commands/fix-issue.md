---
description: Fix issue with TDD and security focus
allowed-tools: Bash, Read, Write, Grep, Glob, Task, AskUserQuestion
---

# Fix: $ARGUMENTS

Follow CLAUDE.md for validation commands and code patterns.

## 1. Gather

- [ ] Fetch issue details:
  ```bash
  gh api graphql -f query='
  {
    repository(owner: "ProvableHQ", name: "snarkVM") {
      issue(number: $ARGUMENTS) {
        title
        body
        comments(first: 50) {
          nodes { body author { login } }
        }
      }
    }
  }'
  ```
- [ ] Identify affected crate(s)
- [ ] Locate relevant code
- [ ] Articulate root cause

Think very hard. Do not proceed until you can explain the root cause.

## 2. Plan (APPROVAL REQUIRED)

Present:
- **Root cause:** ...
- **Proposed fix:** ...
- **Files to modify:** ...
- **Tests to add:** ...

Use **AskUserQuestion** to get approval before proceeding.

## 3. Implement

- [ ] Baseline: all validation passes before changes
- [ ] Write failing test that reproduces the bug
- [ ] Make smallest change that makes test pass
- [ ] Verify: all validation passes

## 4. Report

- Root cause
- What changed
- Test added
- All checks pass

Do not commit.
