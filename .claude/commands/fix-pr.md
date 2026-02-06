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
    if [ -n "$CURSOR" ]; then
      AFTER=', after: "'"$CURSOR"'"'
    else
      AFTER=""
    fi
    RESP=$(gh api graphql -f query='{
      repository(owner: "'"$OWNER"'", name: "'"$REPO"'") {
        pullRequest(number: '"$PR"') {
          reviewThreads(first: 100'"$AFTER"') {
            pageInfo { hasNextPage endCursor }
            nodes {
              isResolved isOutdated path line startLine diffSide startDiffSide
              comments(first: 100) {
                pageInfo { hasNextPage }
                nodes { body author { login } createdAt updatedAt originalLine diffHunk outdated }
              }
            }
          }
        }
      }
    }')
    echo "$RESP" | jq -c '.data.repository.pullRequest.reviewThreads.nodes[]' >> "$WS/threads-pr-$PR.jsonl"
    [ "$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.hasNextPage')" != "true" ] && break
    CURSOR=$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.endCursor')
  done
  jq -s '
    [.[] | select(.isResolved==false)] |
    group_by(.path) |
    map({key: .[0].path, value: [.[] | {
      line, startLine, isOutdated,
      comments: [.comments.nodes[] | {author: .author.login, body, createdAt}]
    }]}) |
    from_entries
  ' "$WS/threads-pr-$PR.jsonl" > "$WS/unresolved-pr-$PR.json"
  jq -s '
    [.[] | select(.isResolved==true)] |
    group_by(.path) |
    map({key: .[0].path, value: [.[] | {
      line, startLine, isOutdated,
      comments: [.comments.nodes[] | {author: .author.login, body, createdAt}]
    }]}) |
    from_entries
  ' "$WS/threads-pr-$PR.jsonl" > "$WS/resolved-pr-$PR.json"
  TOTAL=$(jq -s 'length' "$WS/threads-pr-$PR.jsonl")
  UNRESOLVED=$(jq '[.[]] | add // [] | length' "$WS/unresolved-pr-$PR.json")
  RESOLVED=$(jq '[.[]] | add // [] | length' "$WS/resolved-pr-$PR.json")
  echo "Review: $UNRESOLVED unresolved / $TOTAL total ($RESOLVED resolved)"
}
refresh_threads
```

```bash
cat "$WS/state-pr-$PR.md"
jq -r 'to_entries[] | "== \(.key) ==", (.value[] | "  L\(.line // "?") [\(.comments[0].author)]: \(.comments[0].body[0:100] | gsub("\n"; " "))", (if (.comments | length) > 1 then .comments[1:][] | "    -> [\(.author)]: \(.body[0:100] | gsub("\n"; " "))" else empty end)), ""' "$WS/unresolved-pr-$PR.json"
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
