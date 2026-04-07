# Schema Introspection

The `schema` command lets agents discover the CLI's capabilities at runtime
without relying on static documentation.

## Usage

```bash
# Full schema
keentools-cloud schema

# Schema for one command
keentools-cloud schema --command init
keentools-cloud schema --command run
keentools-cloud schema --command download
```

## Schema Structure

```json
{
  "version": "0.1.0",
  "base_url": "Set via KEENTOOLS_API_URL env var or --api-url flag (required)",
  "commands": {
    "init": {
      "description": "Initialize a reconstruction job",
      "endpoint": "POST /v1/avatar/init",
      "args": { ... }
    },
    "run": {
      "description": "Full pipeline: init → upload → process → download",
      "args": { ... }
    }
  }
}
```

## Agent Pattern

```bash
# At agent startup: load schema
SCHEMA=$(keentools-cloud schema)

# Discover available commands
echo $SCHEMA | jq 'keys'

# Get args for a specific command
echo $SCHEMA | jq '.commands.download.args'

# Get API endpoint
echo $SCHEMA | jq '.commands.process.endpoint'
```

## Notes

- Does not require `--api-url` or `--token`.
- Always outputs JSON (ignores `--output human`).
- Stable contract — agents can rely on this structure across versions.
