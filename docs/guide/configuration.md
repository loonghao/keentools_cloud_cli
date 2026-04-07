# Configuration

## Required Environment Variables

| Variable              | Description                                    |
| --------------------- | ---------------------------------------------- |
| `KEENTOOLS_API_URL`   | Base URL of the KeenTools Cloud API (**required**) |
| `KEENTOOLS_API_TOKEN` | API authentication token                       |

```bash
export KEENTOOLS_API_URL=https://your-api-endpoint.example.com
export KEENTOOLS_API_TOKEN=your-token-here
```

## CLI Flags (Override Env Vars)

```bash
keentools-cloud --api-url https://... --token <TOKEN> <command>
```

## Config File

The token can also be persisted in a config file via the `auth` command:

```bash
keentools-cloud auth login
```

Config file location:

- **Linux/macOS**: `~/.config/keentools-cloud/config.toml`
- **Windows**: `%APPDATA%\keentools-cloud\config.toml`

## Token Priority

1. `--token` flag (or `KEENTOOLS_API_TOKEN` env var, both handled by clap)
2. Config file (`~/.config/keentools-cloud/config.toml`)
3. Error if none found

## Output Format

| Method                      | Result                  |
| --------------------------- | ----------------------- |
| TTY detected (interactive)  | Human-readable, colored |
| Non-TTY (pipe / CI)         | JSON (auto)             |
| `--output json`             | JSON (forced)           |
| `--output human`            | Human (forced)          |
