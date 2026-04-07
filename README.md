# keentools-cloud CLI

[中文](README_zh.md) | **English**

[![CI](https://github.com/loonghao/keentools_cloud_cli/actions/workflows/ci.yml/badge.svg)](https://github.com/loonghao/keentools_cloud_cli/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/loonghao/keentools_cloud_cli?label=release)](https://github.com/loonghao/keentools_cloud_cli/releases/latest)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)](#installation)
[![Downloads](https://img.shields.io/github/downloads/loonghao/keentools_cloud_cli/total)](https://github.com/loonghao/keentools_cloud_cli/releases)

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

### Option 1 — One-liner (Linux / macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash
```

Install a specific version:

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash -s -- v0.1.1
```

### Option 2 — PowerShell (Windows)

```powershell
irm https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.ps1 | iex
```

### Option 3 — Download pre-built binary

Download the binary for your platform from the [Releases page](https://github.com/loonghao/keentools_cloud_cli/releases/latest) and place it on your `PATH`.

| Platform | Archive |
|----------|---------|
| Linux x86_64 | `keentools-cloud-*-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `keentools-cloud-*-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x86_64 | `keentools-cloud-*-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (M1+) | `keentools-cloud-*-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `keentools-cloud-*-x86_64-pc-windows-msvc.zip` |

### Option 4 — Build from source

Requires [Rust](https://rustup.rs/) stable.

```bash
git clone https://github.com/loonghao/keentools_cloud_cli
cd keentools_cloud_cli
cargo build --release
# Binary: ./target/release/keentools-cloud
```

Or install directly from Git:

```bash
cargo install --git https://github.com/loonghao/keentools_cloud_cli
```

### Self-update

```bash
keentools-cloud self-update
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
| `self-update` | Update the CLI to the latest release |

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

## Examples

Ready-to-run examples are in the [`examples/`](examples/) directory:

| File | Description |
|------|-------------|
| [`examples/cli-quickstart.md`](examples/cli-quickstart.md) | Complete guide for every CLI subcommand with copy-paste examples |
| [`examples/pipeline.py`](examples/pipeline.py) | Full Python pipeline using the REST API directly (requires `pip install requests`) |
| [`examples/pipeline.sh`](examples/pipeline.sh) | Full Bash pipeline (requires `curl` + `jq`) |
| [`examples/ipc-qt-demo.py`](examples/ipc-qt-demo.py) | PySide6/PyQt6 desktop app with real-time progress via `--ipc` NDJSON stream |
| [`examples/web-bridge.py`](examples/web-bridge.py) | Flask + Server-Sent Events web frontend driven by `--ipc` mode (requires `pip install flask`) |
| [`examples/actionforge-guide.md`](examples/actionforge-guide.md) | Agentic coding integration with [Actionforge](https://docs.actionforge.dev/agentic-coding/), MCP config, and `.act` graph patterns |

```bash
export KEENTOOLS_API_URL=https://your-api-endpoint.example.com
export KEENTOOLS_API_TOKEN=your_token_here

# Python REST pipeline
python examples/pipeline.py photo1.jpg photo2.jpg photo3.jpg

# Bash pipeline
bash examples/pipeline.sh photo1.jpg photo2.jpg photo3.jpg

# Qt desktop app (real-time progress)
pip install PySide6
python examples/ipc-qt-demo.py

# Web browser frontend
pip install flask
python examples/web-bridge.py   # open http://localhost:5000
```

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
