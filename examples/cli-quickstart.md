# KeenTools Cloud CLI — Quick Start Guide

Complete examples demonstrating every feature of the **keentools-cloud** CLI.
Each example is copy-paste ready — just set your API credentials first.

---

## Table of Contents

1. [Setup & Authentication](#1-setup--authentication)
2. [One-Shot Pipeline (Recommended)](#2-one-shot-pipeline-recommended)
3. [Step-by-Step Pipeline](#3-step-by-step-pipeline)
4. [Ephemeral (Zero-Retention) Pipeline](#4-ephemeral-zero-retention-pipeline)
5. [Advanced Options](#5-advanced-options)
6. [Output Formats](#6-output-formats)
7. [Schema Introspection for AI Agents](#7-schema-introspection-for-ai-agents)

---

## 1. Setup & Authentication

### Install

```bash
# macOS / Linux
curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash

# Windows (PowerShell 5.1+ compatible)
irm https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.ps1 | iex
```

### Authenticate (3 methods, pick one)

```bash
# Method A: Environment variable (one-liner, best for CI/CD)
export KEENTOOLS_API_TOKEN="your-token-here"
export KEENTOOLS_API_URL="https://your-api-endpoint.example.com"

# Method B: Command-line flag (ad-hoc)
keentools-cloud run photo1.jpg photo2.jpg -o result.glb \
  --token your-token-here --api-url "https://your-api-endpoint.example.com"

# Method C: Persistent config (best for interactive use)
keentools-cloud auth login your-token-here \
  --api-url "https://your-api-endpoint.example.com"
# Now every subsequent command auto-reads the token from ~/.config/keentools-cloud/config.toml

# Check auth status at any time
keentools-cloud auth status
# Output:
#   Source:    config file (~/.config/keentools-cloud/config.toml)
#   Token:     abcd...wxyz
```

**Auth priority:** `--token` flag > `KEENTOOLS_API_TOKEN` env var > config file

---

## 2. One-Shot Pipeline (Recommended)

The `run` command does **everything** in one go: init → upload → process → wait → download.

### Basic: 2 photos → GLB model

```bash
keentools-cloud run photo_front.jpg photo_side.jpg -o head.glb
```

**Console output:**
```
✓ Initializing session
  avatar_id = avt_abc123def
  Upload URLs: 2
✓ Uploading photos...
  [1/2] photo_front.jpg ██████████████████████ 100%
  [2/2] photo_side.jpg  ██████████████████████ 100%
✓ Starting reconstruction
⏳ Reconstructing... (polling every 5s)
  running 12% → running 35% → running 68% → completed ✓
✓ Downloading model → head.glb
  38.2 MB saved
```

### With all options

```bash
keentools-cloud run \
  photo1.jpg photo2.jpg photo3.jpg photo4.jpg photo5.jpg \
  -o output/head_with_expressions.glb \
  --format glb \
  --blendshapes arkit,nose \
  --texture png \
  --focal-length-type estimate-per-image \
  --expressions \
  --edges
```

### JSON output (for scripting / CI)

```bash
keentools-cloud run photo1.jpg photo2.jpg -o head.glb --output json
```
```json
{"type":"success","data":{"avatar_id":"avt_abc123","saved_to":"/path/to/head.glb"}}
```

---

## 3. Step-by-Step Pipeline

Use individual commands when you need fine-grained control over each stage.

### Step 1: Initialize session

```bash
keentools-cloud init -n 3
# or with explicit API URL
keentools-cloud init -n 3 --api-url "https://your-api.example.com"
```
**JSON response:**
```json
{
  "avatar_id": "avt_abc123def456",
  "upload_urls": [
    "https://s3.amazonaws.com/bucket/photo_0?signature=...",
    "https://s3.amazonaws.com/bucket/photo_1?signature=...",
    "https://s3.amazonaws.com/bucket/photo_2?signature=..."
  ]
}
```

> **Tip:** Pipe the output to a variable in scripts:
> ```bash
> INIT=$(keentools-cloud init -n 3 --output json)
> AVATAR_ID=$(echo "$INIT" | jq -r '.data.avatar_id')
> UPLOAD_URLS=$(echo "$INIT" | jq -r '.data.upload_urls[]')
> ```

### Step 2: Upload photos

Save the avatar_id and URLs from step 1, then:

```bash
keentools-cloud upload \
  --avatar-id avt_abc123def456 \
  --urls "url0,url1,url2" \
  photo1.jpg photo2.jpg photo3.jpg
```

**Supported formats:** JPEG (.jpg), PNG (.png), HEIC/HEIF (.heic)

### Step 3: Start reconstruction

```bash
keentools-cloud process --avatar-id avt_abc123def456

# With manual focal lengths (when you know camera specs)
keentools-cloud process \
  --avatar-id avt_abc123def456 \
  --focal-length-type manual \
  --focal-lengths 50,50,50

# Enable ARKit blendshapes + expression morph targets
keentools-cloud process \
  --avatar-id avt_abc123def456 \
  --expressions
```

**Focal length modes:**

| Mode | Flag | When to Use |
|------|------|-------------|
| Auto per-image | `--focal-length-type estimate-per-image` (default) | Mixed cameras / unknown specs |
| Common estimate | `--focal-length-type estimate-common` | Same camera for all photos |
| Manual | `--focal-length-type manual --focal-lengths V,V,V` | Known focal lengths |

### Step 4: Poll status

```bash
# One-time check
keentools-cloud status --avatar-id avt_abc123def456
# Output: running 42%

# Continuous polling until done
keentools-cloud status --avatar-id avt_abc123def456 --poll
# Output: running 10% → ... → completed ✓

# Custom poll interval
keentools-cloud status --avatar-id avt_abc123def456 --poll --poll-interval 3
```

### Step 5: Download model

```bash
# GLB (recommended — single file, embedded textures)
keentools-cloud download \
  --avatar-id avt_abc123def456 \
  -o head.glb \
  --format glb \
  --blendshapes arkit,nose \
  --texture jpg

# OBJ format (always delivered as ZIP, auto-extracted)
keentools-cloud download \
  --avatar-id avt_abc123def456 \
  -o head.obj \
  --format obj

# Auto-poll while waiting for model to be packaged
keentools-cloud download \
  --avatar-id avt_abc123def456 \
  -o head.glb \
  --poll
```

### Get reconstruction metadata

```bash
keentools-cloud info --avatar-id avt_abc123def456 --output json
```
```json
{
  "camera_positions": [[[...4x4 matrix...]]],
  "camera_projections": [[[...4x4 matrix...]]],
  "focal_length_type": "estimated_per_image",
  "expressions_enabled": true,
  "img_urls": ["https://s3..."]
}
```

---

## 4. Ephemeral (Zero-Retention) Pipeline

Photos are **never stored** on KeenTools servers.
Results are pushed directly to your S3 pre-signed URL.

```bash
keentools-cloud ephemeral \
  --image-url "https://cdn.example.com/photo1.jpg" \
  --image-url "https://cdn.example.com/photo2.jpg" \
  --image-url "https://cdn.example.com/photo3.jpg" \
  --result-url "glb:https://your-bucket.s3.amazonaws.com/result.glb?sig=..." \
  --result-url "obj:https://your-bucket.s3.amazonaws.com/result.zip?sig=..." \
  --callback-url "https://your-app.example.com/webhook/done" \
  --expressions
```

> **Key differences from standard pipeline:**
> - No separate upload/process/download steps
> - Input images must be publicly accessible URLs (not local files)
> - Results are ONLY available at your provided `--result-url`
> - After completion, `/info` and `/download` return 404

---

## 5. Advanced Options

### Dry-run validation

Validate inputs without calling the API:

```bash
keentools-cloud init -n 5 --dry-run
keentools-cloud run photo1.jpg photo2.jpg -o out.glb --dry-run
keentools-cloud ephemeral \
  --image-url "https://example.com/p1.jpg" \
  --result-url "glb:https://example.com/r.glb" \
  --dry-run
```

### IPC Mode (for GUI integration)

Emit NDJSON progress events on stdout — designed for Qt/Web frontends.

```bash
keentools-cloud run photo1.jpg photo2.jpg -o head.glb --ipc
```

**NDJSON event stream:**
```
{"type":"progress","stage":"init","percent":0,"message":"Initializing session"}
{"type":"progress","stage":"upload","percent":15,"message":"Uploading photo 1/2"}
{"type":"progress","stage":"process","percent":40,"message":"Starting reconstruction"}
{"type":"progress","stage":"reconstruct","percent":65,"message":"Reconstructing... 50%"}
{"type":"progress","stage":"reconstruct","percent":80,"message":"Reconstructing... 85%"}
{"type":"progress","stage":"download","percent":90,"message":"Downloading model"}
{"type":"complete","stage":"done","percent":100,"saved_to":"/abs/path/to/head.glb"}
```

See [ipc-qt-demo.py](./ipc-qt-demo.py) and [web-bridge.py](./web-bridge.py) for full frontend integration examples.

### Self-update

```bash
# Check if an update is available
keentools-cloud self-update --check

# Update to latest version
keentools-cloud self-update

# Update to a specific version
keentools-cloud self-update --version v0.2.0

# Force re-install
keentools-cloud self-update --force
```

---

## 6. Output Formats

### Human-readable (default on TTY)

```
✓ Initializing session
  avatar_id = avt_abc123def
```

### Machine-readable (default when piped)

```bash
keentools-cloud status --avatar-id avt_abc123 | jq .
```
```json
{"status":{"value":"completed","avatar_id":"avt_abc123"}}
```

Force JSON output even on terminal:
```bash
keentools-cloud status --avatar-id avt_abc123 --output json
```

### Exit codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | API error |
| 2 | Input validation error |
| 3 | Authentication error |

---

## 7. Schema Introspection for AI Agents

The built-in `schema` command dumps the entire CLI capability as machine-readable JSON.
AI coding assistants can discover commands and parameters at runtime without external docs.

```bash
# Full schema (all commands)
keentools-cloud schema | python3 -m json.tool > cli-schema.json

# Single command schema
keentools-cloud schema run | python3 -m json.tool
```

This enables seamless integration with AI agent frameworks like:
- **[Actionforge Agentic Coding](https://docs.actionforge.dev/agentic-coding/)**
- Claude Code, Cursor, Copilot (via MCP)
- Any LLM tool-use framework

See the [actionforge integration guide](./actionforge-guide.md) for details.
