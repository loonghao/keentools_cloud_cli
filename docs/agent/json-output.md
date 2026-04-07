# JSON Output

## Auto-detection

`keentools-cloud` detects whether stdout is a TTY at startup:

- **TTY** → human-readable, colored output with progress bars
- **Non-TTY** (pipe, redirect, CI) → JSON output automatically

```bash
# Interactive: human output
keentools-cloud init --count 3

# Piped: JSON output
keentools-cloud init --count 3 | jq '.avatar_id'
```

## Force JSON

```bash
keentools-cloud --output json init --count 3
```

## Output Structure

Every successful command outputs a single JSON object to **stdout**:

```json
{ "field": "value", ... }
```

## Error Structure

Errors are written to **stderr**:

```json
{ "error": "description of the problem" }
```

Exit code is `1` on error, `0` on success.

## Polling / Streaming

When `--poll` is used with `--output json`, status updates are emitted as
**NDJSON** (newline-delimited JSON) — one JSON object per line:

```
{"status":"running"}
{"status":"running","progress":45}
{"status":"completed"}
```

Agents can stream-parse this with any NDJSON parser or `jq -R 'fromjson?'`.
