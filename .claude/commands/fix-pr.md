---
description: Fix PR review feedback with TDD
allowed-tools: Bash, Read, Write, Grep, Glob, Task, AskUserQuestion
---

# Fix PR: $ARGUMENTS

```bash
PR=$ARGUMENTS
WS=".claude/workspace"
OWNER="ProvableHQ"
REPO="snarkVM"
```

## 1. Context

If missing: run `/fetch pr $PR` first.

Quick-refresh threads to get latest resolved status (faster than full fetch):

```bash
refresh_threads() {
  CURSOR=""
  > "$WS/threads-pr-$PR.jsonl"
  while true; do
    RESP=$(gh api graphql -f query='{
      repository(owner: "'"$OWNER"'", name: "'"$REPO"'") {
        pullRequest(number: '"$PR"') {
          reviewThreads(first: 100'"${CURSOR:+, after: \"$CURSOR\"}"') {
            pageInfo { hasNextPage endCursor }
            nodes { isResolved path line comments(first: 100) { nodes { body author { login } } } }
          }
        }
      }
    }')
    echo "$RESP" | jq -c '.data.repository.pullRequest.reviewThreads.nodes[]' >> "$WS/threads-pr-$PR.jsonl"
    [ "$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.hasNextPage')" != "true" ] && break
    CURSOR=$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.endCursor')
  done
  jq -s '[.[] | select(.isResolved==false) | {path, line, reviewer: .comments.nodes[0].author.login, comment: .comments.nodes[0].body[0:200]}]' "$WS/threads-pr-$PR.jsonl" > "$WS/unresolved-pr-$PR.json"
  jq -s '[.[] | select(.isResolved==true) | {path, line, reviewer: .comments.nodes[0].author.login, comment: .comments.nodes[0].body[0:200]}]' "$WS/threads-pr-$PR.jsonl" > "$WS/resolved-pr-$PR.json"
  TOTAL=$(jq -s 'length' "$WS/threads-pr-$PR.jsonl")
  UNRESOLVED=$(jq 'length' "$WS/unresolved-pr-$PR.json")
  RESOLVED=$(jq 'length' "$WS/resolved-pr-$PR.json")
  echo "Review: $UNRESOLVED unresolved / $TOTAL total ($RESOLVED resolved)"
}
refresh_threads
```

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
