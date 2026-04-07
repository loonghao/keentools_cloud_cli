---
name: keentools-cloud
version: "0.1"
description: Unofficial CLI for KeenTools Cloud 3D Head Reconstruction API
disclaimer: Not affiliated with or endorsed by KeenTools.
---

# keentools-cloud Agent Skill

This file encodes invariants and usage patterns for AI agents.
Always read this file before using `keentools-cloud` in an automated pipeline.

## Authentication

```bash
export KEENTOOLS_API_TOKEN=your_token_here
export KEENTOOLS_API_URL=https://your-api-endpoint.example.com
```

Or store the token once:
```bash
keentools-cloud auth login <token>
```

## Invariants (ALWAYS follow these)

1. **Use `--dry-run` before any mutating operation** (`init`, `process`, `ephemeral`).
2. **Use `--output json`** in all automated pipelines. Non-TTY stdout defaults to JSON automatically.
3. **Use `status --poll`** to wait for completion — never manually loop `status` without `--poll`.
4. **Blendshapes are comma-separated, NOT repeated flags:**
   - Correct: `--blendshapes arkit,nose`
   - Wrong: `--blendshapes arkit --blendshapes nose` ← causes API error
5. **OBJ format always returns a ZIP archive** — save with `.zip` extension, then extract.
6. **Ephemeral mode**: after processing completes, `/info` and `/download` return 404. Results are ONLY in the URLs you provided.
7. **Photos and upload URLs are matched by position** — order matters.
8. **Photo count must be 2–15** for both standard and ephemeral pipelines.

| `KEENTOOLS_API_TOKEN` | API authentication token |
| `KEENTOOLS_API_URL`   | API base URL (required) |

## Simple Pipeline (use `run` for single-command convenience)

```bash
keentools-cloud run photo1.jpg photo2.jpg photo3.jpg \
  --output-path head.glb \
  --blendshapes arkit,nose \
  --texture jpg
```

## Step-by-Step Pipeline (for more control)

```bash
# 1. Initialize — save the avatar_id
INIT=$(keentools-cloud init --count 3 --output json)
AVATAR_ID=$(echo "$INIT" | jq -r .avatar_id)
URLS=$(echo "$INIT" | jq -r '.upload_urls[]')

# 2. Upload photos (order must match)
keentools-cloud upload \
  --avatar-id "$AVATAR_ID" \
  --urls "$(echo "$INIT" | jq -r '.upload_urls | join(",")')" \
  photo1.jpg photo2.jpg photo3.jpg

# 3. Start reconstruction
keentools-cloud process \
  --avatar-id "$AVATAR_ID" \
  --focal-length-type estimate-per-image

# 4. Wait for completion
keentools-cloud status --avatar-id "$AVATAR_ID" --poll

# 5. Download result
keentools-cloud download \
  --avatar-id "$AVATAR_ID" \
  --output-path head.glb \
  --format glb \
  --blendshapes arkit,nose \
  --texture jpg \
  --poll
```

## Ephemeral Pipeline (zero data retention)

```bash
keentools-cloud ephemeral \
  --image-url https://your-bucket.s3.amazonaws.com/photo1.jpg \
  --image-url https://your-bucket.s3.amazonaws.com/photo2.jpg \
  --result-url glb:https://your-bucket.s3.amazonaws.com/result.glb?<presigned-put-params> \
  --focal-length-type estimate-per-image \
  --callback-url https://your-server.com/webhook

# Then poll status (download endpoint is unavailable in ephemeral mode)
keentools-cloud status --avatar-id <ID> --poll --output json
```

## Schema Introspection (for agents)

```bash
# List all commands
keentools-cloud schema

# Describe a specific command
keentools-cloud schema download
keentools-cloud schema run
```

## Output Formats

| Scenario | Format |
|----------|--------|
| TTY (terminal) | Human-readable with colors |
| Non-TTY (pipe/redirect) | JSON (auto-detected) |
| Forced JSON | `--output json` |

JSON output is NDJSON (one object per line) when streaming multiple events (e.g., during `run`).

## Focal Length Reference

| Mode | When to use |
|------|-------------|
| `estimate-per-image` | Mixed cameras, different zoom levels (safe default) |
| `estimate-common` | Same camera/lens, all same resolution |
| `manual` | Known exact values; provide `--focal-lengths 24,28,35` |

Typical smartphone 35mm-equivalent: 24–28mm.

## Error Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | API or runtime error |
| 2 | Input validation error (bad avatar ID, path traversal, etc.) |
| 3 | Authentication error |
