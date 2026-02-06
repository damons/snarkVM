@../AGENTS.md

# Workspace

Use `.claude/workspace/` for scratch files (gitignored).

```
.claude/workspace/
├── context-{type}-{id}.json    # Structured data (PR metadata, files list)
├── state-{type}-{id}.md        # Human-readable state + progress log
└── handoff-{type}-{id}.md      # Cross-skill communication
```

## Staleness

Workspace files have a **Fetched:** timestamp in state files. On session resume:
1. Check `ls -la .claude/workspace/` for existing data
2. If state file exists, check the **Fetched:** timestamp
3. If data is >1 hour old or you've made code changes since, re-fetch: `/fetch <type> <num> --force`
4. Never trust workspace data without checking the timestamp

## Session Recovery

If resuming after a break:
1. Check workspace: `ls -la .claude/workspace/`
2. Read state file for progress and last timestamp
3. Re-fetch if stale, then continue from last completed step


<claude-mem-context>

</claude-mem-context>
