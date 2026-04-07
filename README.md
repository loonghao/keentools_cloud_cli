# keentools-cloud CLI

> **Unofficial CLI for the [KeenTools Cloud](https://keentools.io) 3D Head Reconstruction API.**
> This project is not affiliated with or endorsed by KeenTools.

Generate photorealistic 3D head models from 2–15 ordinary photos directly from your terminal or any automated workflow. Built in Rust, designed with [Agent DX best practices](https://justin.poehnelt.com/posts/rewrite-your-cli-for-ai-agents/) for reliable use by AI agents like Claude, OpenClaw, and others.

## Features

- **Full pipeline in one command** — `run` handles init → upload → reconstruct → download
- **Agent-first design** — JSON output, `--dry-run`, `schema` introspection, input hardening
- **Standard and ephemeral pipelines** — cloud-based or zero-retention (photos never stored)
- **Flexible auth** — env var, `--token` flag, or config file
- **Progress bars** on TTY, NDJSON streaming in pipelines

## Installation

```bash
cargo install --git https://github.com/loonghao/keentools_cloud_cli
```

Or build from source:

```bash
git clone https://github.com/loonghao/keentools_cloud_cli
cd keentools_cloud_cli
cargo build --release
# Binary: ./target/release/keentools-cloud
```

## Quick Start

```bash
# Set your API token and endpoint
export KEENTOOLS_API_TOKEN=your_token_here
export KEENTOOLS_API_URL=https://your-api-endpoint.example.com

# Full pipeline in one command
keentools-cloud run photo1.jpg photo2.jpg photo3.jpg \
  --output-path head.glb \
  --blendshapes arkit,nose \
  --texture jpg
```

## Commands

| Command | Description |
|---------|-------------|
| `run` | Full pipeline shortcut (recommended) |
| `init` | Initialize a session, get upload URLs |
| `upload` | Upload photos to pre-signed S3 URLs |
| `process` | Start reconstruction |
| `status` | Check status (use `--poll` to wait) |
| `download` | Download the 3D model (use `--poll`) |
| `info` | Get camera matrices and reconstruction metadata |
| `ephemeral` | Zero-retention pipeline |
| `schema` | Dump full CLI schema as JSON |
| `auth` | Manage stored API token |

## Step-by-Step Pipeline

```bash
# 1. Initialize
INIT=$(keentools-cloud init --count 3 --output json)
AVATAR_ID=$(echo "$INIT" | jq -r .avatar_id)

# 2. Upload photos
keentools-cloud upload \
  --avatar-id "$AVATAR_ID" \
  --urls "$(echo "$INIT" | jq -r '.upload_urls | join(",")')" \
  photo1.jpg photo2.jpg photo3.jpg

# 3. Start reconstruction
keentools-cloud process \
  --avatar-id "$AVATAR_ID" \
  --focal-length-type estimate-per-image

# 4. Poll until done
keentools-cloud status --avatar-id "$AVATAR_ID" --poll

# 5. Download
keentools-cloud download \
  --avatar-id "$AVATAR_ID" \
  --output-path head.glb \
  --format glb \
  --blendshapes arkit,nose \
  --poll
```

## Ephemeral Pipeline (zero data retention)

Photos are processed in-memory and results are pushed directly to your pre-signed URLs. No data is retained on KeenTools servers.

```bash
keentools-cloud ephemeral \
  --image-url https://your-storage.com/photo1.jpg \
  --image-url https://your-storage.com/photo2.jpg \
  --result-url glb:https://your-storage.com/result.glb?<presigned-put> \
  --focal-length-type estimate-per-image \
  --callback-url https://your-server.com/webhook
```

> **Note:** In ephemeral mode, `download` and `info` endpoints are unavailable after processing. Results are only in the URLs you provided.

## Output Formats

- **TTY** (terminal): Colorized, human-readable output
- **Non-TTY** (pipe/redirect): JSON, automatically detected
- **Forced JSON**: `--output json`

```bash
# In automated pipelines
keentools-cloud status --avatar-id abc123 --output json
# {"status":"completed"}
```

## Authentication

Priority order:
1. `--token <TOKEN>` flag
2. `KEENTOOLS_API_TOKEN` environment variable
3. `~/.config/keentools-cloud/config.toml`

The API base URL must be set via `KEENTOOLS_API_URL` or `--api-url`.

```bash
# Store token permanently
keentools-cloud auth login <token>

# Check current auth
keentools-cloud auth status

# Remove stored token
keentools-cloud auth logout
```

## Agent Integration

This CLI is designed for reliable use in AI agent workflows:

```bash
# Discover all commands and parameters at runtime
keentools-cloud schema

# Describe a specific command
keentools-cloud schema run

# Dry-run before mutating operations
keentools-cloud init --count 3 --dry-run
keentools-cloud process --avatar-id abc --focal-length-type estimate-per-image --dry-run
```

See [SKILL.md](SKILL.md) for a complete agent skill file with invariants and usage patterns.

## Output Formats

### GLB (recommended)
Single binary file (~40 MB) with embedded JPEG textures, wireframe edges, and morph targets. Use with `--format glb`.

### OBJ
Always returned as a **ZIP archive** containing `.obj`, `.mtl`, and texture files. Use with `--format obj`.

## Blendshapes (GLB only)

| Group | Description |
|-------|-------------|
| `arkit` | 51 ARKit-compatible morph targets |
| `expression` | Numbered expressions (requires `--expressions` during `process`) |
| `nose` | Nose shape controls |

```bash
--blendshapes arkit,nose          # comma-separated, not repeated flags
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | API or runtime error |
| 2 | Input validation error |
| 3 | Authentication error |

## Disclaimer

This is an **unofficial** tool and is not affiliated with, endorsed by, or supported by KeenTools. For official support, visit [keentools.io](https://keentools.io).
