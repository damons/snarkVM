#!/bin/bash
# Quick refresh of PR review threads (faster than full fetch)
# Usage: refresh-threads.sh <pr_number>

set -euo pipefail

OWNER="ProvableHQ"
REPO="snarkVM"
WS="${WS:-.claude/workspace}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Validate arguments
if [[ $# -lt 1 ]]; then
  echo "Usage: refresh-threads.sh <pr_number>"
  exit 1
fi

PR="$1"

# Check gh auth
if ! gh auth status &>/dev/null; then
  log_error "gh CLI not authenticated. Run 'gh auth login' first."
  exit 1
fi

mkdir -p "$WS"

log_info "Refreshing review threads for PR #$PR..."

# Fetch review threads with pagination
CURSOR=""
> "$WS/threads-pr-$PR.jsonl"

while true; do
  RESP=$(gh api graphql -f query='{
    repository(owner: "'"$OWNER"'", name: "'"$REPO"'") {
      pullRequest(number: '"$PR"') {
        reviewThreads(first: 100'"${CURSOR:+, after: \"$CURSOR\"}"') {
          pageInfo { hasNextPage endCursor }
          nodes { 
            isResolved 
            path 
            line 
            comments(first: 100) { 
              nodes { 
                body 
                author { login } 
              } 
            } 
          }
        }
      }
    }
  }')
  
  echo "$RESP" | jq -c '.data.repository.pullRequest.reviewThreads.nodes[]' >> "$WS/threads-pr-$PR.jsonl"
  
  HAS_NEXT=$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.hasNextPage')
  [[ "$HAS_NEXT" != "true" ]] && break
  
  CURSOR=$(echo "$RESP" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.endCursor')
done

# Extract unresolved threads
jq -s '[.[] | select(.isResolved==false) | {
  path, 
  line, 
  reviewer: .comments.nodes[0].author.login, 
  comment: .comments.nodes[0].body[0:200]
}]' "$WS/threads-pr-$PR.jsonl" > "$WS/unresolved-pr-$PR.json"

# Extract resolved threads
jq -s '[.[] | select(.isResolved==true) | {
  path, 
  line, 
  reviewer: .comments.nodes[0].author.login, 
  comment: .comments.nodes[0].body[0:200]
}]' "$WS/threads-pr-$PR.jsonl" > "$WS/resolved-pr-$PR.json"

# Compute counts
TOTAL=$(jq -s 'length' "$WS/threads-pr-$PR.jsonl")
UNRESOLVED=$(jq 'length' "$WS/unresolved-pr-$PR.json")
RESOLVED=$(jq 'length' "$WS/resolved-pr-$PR.json")

log_info "Review threads: $UNRESOLVED unresolved / $TOTAL total ($RESOLVED resolved)"

# Output summary for caller
echo "TOTAL=$TOTAL"
echo "UNRESOLVED=$UNRESOLVED"
echo "RESOLVED=$RESOLVED"
