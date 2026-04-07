# schema

Dump the CLI and API schema as JSON for agent introspection.

```bash
keentools-cloud schema [--command <CMD>]
```

## Options

| Flag                | Description                                 |
| ------------------- | ------------------------------------------- |
| `--command <CMD>`   | Show schema for a specific command only     |

## Examples

```bash
# Full schema
keentools-cloud schema

# Schema for a specific command
keentools-cloud schema --command init
keentools-cloud schema --command run
```

## Output

Returns a JSON object describing:

- All commands with their arguments, types, defaults, and descriptions
- API endpoint information (method, path, summary)
- Environment variables required

## Agent Use

Agents can call `schema` at startup to discover available commands and
their parameters without needing external documentation:

```bash
SCHEMA=$(keentools-cloud schema --output json)
echo $SCHEMA | jq '.commands.run.args'
```

## Notes

- Does **not** require `--api-url` or `--token`.
- Always outputs JSON regardless of `--output` flag.
