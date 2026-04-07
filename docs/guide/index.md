# Introduction

`keentools-cloud` is an **unofficial** command-line interface for the
[KeenTools Cloud 3D Head Reconstruction API](https://keentools.io).

It is designed with two goals:

1. **Human usability** — colored output, progress bars, and clear error messages
2. **Agent DX** — JSON output, `--dry-run`, schema introspection, and `--poll` flags
   following Google Cloud's [Agent DX best practices](https://jpoehnelt.com/posts/cli-for-ai-agents/)

> This project is not affiliated with or endorsed by KeenTools.

## Features

- Full reconstruction pipeline: `init → upload → process → status → download`
- One-shot `run` command for the complete pipeline
- Zero-retention `ephemeral` pipeline
- `schema` command for runtime CLI/API introspection by agents
- `self-update` to stay current
- JSON output mode for machine consumption
- Cross-platform: Linux, macOS, Windows
