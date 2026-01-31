---
description: Fetch GitHub context (pr/issue) into .claude/workspace/
allowed-tools: Bash, Read, Write
---

# Fetch: $ARGUMENTS

Usage: `/fetch pr <number>` or `/fetch issue <number>`

## Setup

```bash
TYPE=$(echo "$ARGUMENTS" | awk '{print $1}')
OWNER="ProvableHQ"
REPO="snarkVM"
WS=".claude/workspace"
mkdir -p "$WS"

# Parse --force flag.
FORCE=0
case "$ARGUMENTS" in *--force*|*-f*) FORCE=1 ;; esac
NUM=$(echo "$ARGUMENTS" | sed 's/--force//g; s/-f//g' | awk '{print $2}')

[ -z "$TYPE" ] || [ -z "$NUM" ] && echo "Usage: /fetch pr|issue <number> [--force]" && exit 1
[ "$TYPE" != "pr" ] && [ "$TYPE" != "issue" ] && echo "Unknown type: $TYPE. Use pr or issue." && exit 1
```

## Skip if fresh

```bash
[ "$FORCE" = "0" ] && [ -f "$WS/context-$TYPE-$NUM.json" ] && \
  [ $(( $(date +%s) - $(stat -f %m "$WS/context-$TYPE-$NUM.json" 2>/dev/null || stat -c %Y "$WS/context-$TYPE-$NUM.json") )) -lt 3600 ] && \
  echo "Context fresh (use --force to bypass). Delete $WS/*$TYPE*$NUM* to refresh." && exit 0
```

## Fetch PR

```bash
if [ "$TYPE" = "pr" ]; then
  # Metadata
  gh pr view $NUM --json title,body,author,state,headRefName,baseRefName,additions,deletions,changedFiles,labels,reviews,mergeable,createdAt,updatedAt > "$WS/context-pr-$NUM.json"

  # Files
  gh pr view $NUM --json files --jq '.files[] | "\(.additions)+/\(.deletions)- \(.path)"' > "$WS/files-pr-$NUM.txt"

  # Commits (paginated)
  gh api repos/$OWNER/$REPO/pulls/$NUM/commits --paginate | jq '[.[] | {sha: .sha[0:7], message: .commit.message | split("\n")[0]}]' > "$WS/commits-pr-$NUM.json"

  # PR comments (paginated)
  gh api repos/$OWNER/$REPO/issues/$NUM/comments --paginate | jq '[.[] | {author: .user.login, date: .created_at[0:10], body: .body}]' > "$WS/comments-pr-$NUM.json"

  # Check runs / CI status
  gh pr checks $NUM --json name,state,conclusion 2>/dev/null | jq '[.[] | {name, state, conclusion}]' > "$WS/checks-pr-$NUM.json" || echo "[]" > "$WS/checks-pr-$NUM.json"

  # Review threads (paginated GraphQL)
  CURSOR=""
  > "$WS/threads-pr-$NUM.jsonl"
  while true; do
    RESP=$(gh api graphql -f query='{
      repository(owner: "'"$OWNER"'", name: "'"$REPO"'") {
        pullRequest(number: '"$NUM"') {
          reviewThreads(first: 100'"${CURSOR:+, after: \"$CURSOR\"}"') {
            pageInfo { hasNextPage endCursor }
            nodes { isResolved path line comments(first: 100) { nodes { body author { login } } } }
          }
        }
      }
    }')
    echo "$RESP" | jq -c '.data.repository.pullRequest.reviewThreads.nodes[]' >> "$WS/threads-pr-$NUM.jsonl"
    [ "$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.hasNextPage')" != "true" ] && break
    CURSOR=$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.endCursor')
  done

  # Extract unresolved
  jq -s '[.[] | select(.isResolved==false) | {path, line, reviewer: .comments.nodes[0].author.login, comment: .comments.nodes[0].body[0:200]}]' "$WS/threads-pr-$NUM.jsonl" > "$WS/unresolved-pr-$NUM.json"

  # Extract resolved
  jq -s '[.[] | select(.isResolved==true) | {path, line, reviewer: .comments.nodes[0].author.login, comment: .comments.nodes[0].body[0:200]}]' "$WS/threads-pr-$NUM.jsonl" > "$WS/resolved-pr-$NUM.json"

  # Linked issues (parse from body)
  jq -r '.body // ""' "$WS/context-pr-$NUM.json" | grep -oE '#[0-9]+|issues/[0-9]+|ProvableHQ/snarkVM/issues/[0-9]+' | grep -oE '[0-9]+' | sort -u > "$WS/linked-issues-pr-$NUM.txt"

  # Compute review counts
  TOTAL_THREADS=$(jq -s 'length' "$WS/threads-pr-$NUM.jsonl")
  UNRESOLVED=$(jq 'length' "$WS/unresolved-pr-$NUM.json")
  RESOLVED=$(jq 'length' "$WS/resolved-pr-$NUM.json")

  # State file
  cat > "$WS/state-pr-$NUM.md" << EOF
# PR $NUM — $(jq -r .title "$WS/context-pr-$NUM.json")
**Author:** $(jq -r .author.login "$WS/context-pr-$NUM.json") | **Branch:** $(jq -r .headRefName "$WS/context-pr-$NUM.json")
**Stats:** $(jq -r '"\(.additions)+/\(.deletions)-/\(.changedFiles) files"' "$WS/context-pr-$NUM.json")
**Review:** $UNRESOLVED unresolved / $TOTAL_THREADS total ($RESOLVED resolved)
**CI:** $(jq -r 'if length == 0 then "none" else [.[] | "\(.name):\(.conclusion // .state)"] | join(", ") end' "$WS/checks-pr-$NUM.json")

## Findings
| Sev | Location | Issue | Fix |
|-----|----------|-------|-----|

## Log
| Action | Result |
|--------|--------|
EOF
fi
```

## Fetch Issue

```bash
if [ "$TYPE" = "issue" ]; then
  # Metadata + comments (paginated GraphQL)
  CURSOR=""
  > "$WS/comments-issue-$NUM.jsonl"

  RESP=$(gh api graphql -f query='{
    repository(owner: "'"$OWNER"'", name: "'"$REPO"'") {
      issue(number: '"$NUM"') {
        title body state author { login } createdAt labels(first: 10) { nodes { name } }
        comments(first: 100) { pageInfo { hasNextPage endCursor } nodes { body author { login } createdAt } }
      }
    }
  }')

  echo "$RESP" | jq '{title: .data.repository.issue.title, body: .data.repository.issue.body, state: .data.repository.issue.state, author: .data.repository.issue.author.login, labels: [.data.repository.issue.labels.nodes[].name]}' > "$WS/context-issue-$NUM.json"
  echo "$RESP" | jq -c '.data.repository.issue.comments.nodes[]' >> "$WS/comments-issue-$NUM.jsonl"
  HAS_NEXT=$(echo "$RESP" | jq -r '.data.repository.issue.comments.pageInfo.hasNextPage')
  CURSOR=$(echo "$RESP" | jq -r '.data.repository.issue.comments.pageInfo.endCursor')

  while [ "$HAS_NEXT" = "true" ]; do
    RESP=$(gh api graphql -f query='{
      repository(owner: "'"$OWNER"'", name: "'"$REPO"'") {
        issue(number: '"$NUM"') {
          comments(first: 100, after: "'"$CURSOR"'") { pageInfo { hasNextPage endCursor } nodes { body author { login } createdAt } }
        }
      }
    }')
    echo "$RESP" | jq -c '.data.repository.issue.comments.nodes[]' >> "$WS/comments-issue-$NUM.jsonl"
    HAS_NEXT=$(echo "$RESP" | jq -r '.data.repository.issue.comments.pageInfo.hasNextPage')
    CURSOR=$(echo "$RESP" | jq -r '.data.repository.issue.comments.pageInfo.endCursor')
  done

  # Timeline events (linked PRs, references)
  gh api repos/$OWNER/$REPO/issues/$NUM/timeline --paginate 2>/dev/null | jq '[.[] | select(.event == "cross-referenced" or .event == "referenced") | {event, source: .source.issue.number, actor: .actor.login}]' > "$WS/timeline-issue-$NUM.json" || echo "[]" > "$WS/timeline-issue-$NUM.json"

  # Extract linked PRs
  jq -r '[.[] | select(.source != null) | .source] | unique | .[]' "$WS/timeline-issue-$NUM.json" > "$WS/linked-prs-issue-$NUM.txt" 2>/dev/null || true

  # State file
  cat > "$WS/state-issue-$NUM.md" << EOF
# Issue $NUM — $(jq -r .title "$WS/context-issue-$NUM.json")
**Author:** $(jq -r .author "$WS/context-issue-$NUM.json") | **State:** $(jq -r .state "$WS/context-issue-$NUM.json")

## Problem
$(jq -r '.body[0:500]' "$WS/context-issue-$NUM.json")

## Investigation
- **Crate:**
- **Files:**
- **Root cause:**

## Plan
- **Fix:**
- **Tests:**
- **Risk:**

## Log
| Action | Result |
|--------|--------|
EOF
fi
```

## Done

```bash
echo "=== $TYPE $NUM ready ==="
ls -la "$WS"/*$TYPE*$NUM* 2>/dev/null
echo ""
echo "Files fetched:"
[ "$TYPE" = "pr" ] && echo "  - context, files, commits, comments, checks, threads, unresolved, resolved, linked-issues, state"
[ "$TYPE" = "issue" ] && echo "  - context, comments, timeline, linked-prs, state"
```
