# Agent Integration Overview

`keentools-cloud` is designed to be reliably usable by AI agents
(Claude, GPT, OpenClaw, etc.) following Google Cloud's
[Agent DX best practices](https://jpoehnelt.com/posts/cli-for-ai-agents/).

## Key Agent-Friendly Features

### 1. JSON Output Mode

All commands emit structured JSON when:
- stdout is not a TTY (pipe / CI environment)
- `--output json` is passed explicitly

```bash
# Machine-readable output
keentools-cloud --output json init --count 3
# → { "avatar_id": "...", "upload_urls": [...] }
```

### 2. Schema Introspection

```bash
keentools-cloud schema
keentools-cloud schema --command run
```

Agents can call this at startup to discover all commands, arguments, and API
endpoints without needing external documentation.

### 3. `--dry-run` Flag

Available on `init`, `process`, and `run`. Validates inputs and prints what
would be sent to the API — without making any network calls.

```bash
keentools-cloud run --photos *.jpg --output-path head.obj --dry-run
```

### 4. `--poll` Flag

Blocks until the async operation completes, emitting progress events as
NDJSON to stdout. No need for the agent to implement polling loops.

```bash
keentools-cloud status --avatar-id abc123 --poll --output json
# streams: {"status":"running"} {"status":"running"} {"status":"completed"}
```

### 5. Structured Errors

All errors are written to **stderr** as plain text (human) or JSON (`--output json`).
Exit codes: `0` = success, `1` = error.

```json
{ "error": "avatar_id contains invalid characters" }
```

### 6. `SKILL.md` Invariant

The `SKILL.md` file in the repository root describes the tool's contract —
commands, env vars, output formats — as a stable reference for agents.
