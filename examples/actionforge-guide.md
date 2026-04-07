# Actionforge Agentic Coding Integration

Integrate **keentools-cloud** into your AI agent workflows using
[Actionforge](https://docs.actionforge.dev/agentic-coding/).
The CLI's built-in `schema` command and `--ipc` mode make it a
first-class citizen in agentic pipelines.

---

## Why This Works Well

| CLI feature | Agentic benefit |
|-------------|-----------------|
| `keentools-cloud schema` | Agent discovers every command and parameter at runtime — no docs needed |
| `--output json` | All responses are machine-readable JSON |
| `--dry-run` | Agent can validate inputs safely before committing |
| `--ipc` (NDJSON stream) | Agent monitors long-running jobs without blocking |
| Exit codes 0/1/2/3 | Unambiguous success / error signalling |

---

## 1. Let an Agent Discover the CLI

Before writing any `.act` graph, an agent can query the CLI's own schema:

```bash
keentools-cloud schema | jq .
```

```json
{
  "tool": "keentools-cloud",
  "version": "0.1.2",
  "commands": {
    "run":       { "description": "Full pipeline ...", "params": { ... } },
    "init":      { "description": "Init session ...", "params": { ... } },
    "ephemeral": { "description": "Zero-retention ...", "params": { ... } },
    ...
  }
}
```

An Actionforge agent can call this once to auto-generate the correct
node inputs for any command, eliminating manual schema maintenance.

---

## 2. MCP Server Configuration

### Claude Code

```bash
# Add the keentools-cloud schema as a context source
keentools-cloud schema > .keentools-schema.json

# Register a local MCP server that wraps the CLI
claude mcp add keentools-cloud -- keentools-cloud schema
```

### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "keentools-cloud": {
      "command": "keentools-cloud",
      "args": ["schema"]
    }
  }
}
```

### GitHub Copilot / VS Code (`.vscode/mcp.json`)

```json
{
  "servers": {
    "keentools-cloud": {
      "type": "local",
      "command": "keentools-cloud",
      "args": ["schema"]
    }
  }
}
```

### OpenCode (`opencode.json`)

```json
{
  "mcp": {
    "keentools-cloud": {
      "type": "local",
      "command": ["keentools-cloud", "schema"]
    }
  }
}
```

---

## 3. Actionforge Graph Patterns

### Pattern A: Standard Pipeline `.act` graph

An agent-generated `.act` graph for a full reconstruction run:

```json
{
  "nodes": [
    {
      "id": "set-env",
      "type": "core/env-set",
      "inputs": {
        "KEENTOOLS_API_URL":   "${{ vars.API_URL }}",
        "KEENTOOLS_API_TOKEN": "${{ secrets.API_TOKEN }}"
      }
    },
    {
      "id": "validate",
      "type": "core/shell",
      "needs": ["set-env"],
      "inputs": {
        "run": "keentools-cloud run ${{ inputs.photos }} -o ${{ inputs.output }} --dry-run"
      }
    },
    {
      "id": "reconstruct",
      "type": "core/shell",
      "needs": ["validate"],
      "inputs": {
        "run": "keentools-cloud run ${{ inputs.photos }} -o ${{ inputs.output }} --output json"
      }
    },
    {
      "id": "notify",
      "type": "core/shell",
      "needs": ["reconstruct"],
      "inputs": {
        "run": "echo 'Model saved to ${{ steps.reconstruct.outputs.saved_to }}'"
      }
    }
  ]
}
```

### Pattern B: Ephemeral pipeline for privacy-sensitive data

```json
{
  "nodes": [
    {
      "id": "ephemeral-run",
      "type": "core/shell",
      "inputs": {
        "run": [
          "keentools-cloud ephemeral",
          "--image-url ${{ inputs.photo_url_1 }}",
          "--image-url ${{ inputs.photo_url_2 }}",
          "--result-url 'glb:${{ inputs.result_presigned_url }}'",
          "--callback-url ${{ inputs.webhook_url }}",
          "--output json"
        ]
      }
    }
  ]
}
```

### Pattern C: IPC-driven progress monitoring

Pipe `--ipc` NDJSON output to an agent that can react to progress:

```json
{
  "nodes": [
    {
      "id": "run-with-ipc",
      "type": "core/shell",
      "inputs": {
        "run": "keentools-cloud run photo1.jpg photo2.jpg -o head.glb --ipc 2>&1 | tee reconstruction.ndjson"
      }
    },
    {
      "id": "check-result",
      "type": "core/shell",
      "needs": ["run-with-ipc"],
      "inputs": {
        "run": "tail -1 reconstruction.ndjson | jq -r 'if .type == \"complete\" then .saved_to else error(.message) end'"
      }
    }
  ]
}
```

---

## 4. Agent Workflow Example

A complete agentic coding session using Claude Code + Actionforge:

```bash
# 1. Claude discovers the CLI capabilities
keentools-cloud schema

# 2. Claude creates a graph for the task
#    (e.g., "create a graph that reconstructs 3 photos and notifies a webhook")

# 3. Agent validates the graph before running
actrun validate reconstruction.act

# 4. Agent runs the graph step by step for inspection
actrun mcp  # starts local MCP debug server
# → debug_run: loads reconstruction.act
# → debug_step: init node runs, avatar_id returned
# → debug_inspect: check avatar_id value
# → debug_resume: upload + process + download complete

# 5. Agent sees results and iterates
```

---

## 5. Environment Variables Reference

All settings can be injected via Actionforge `vars` / `secrets`:

| Variable | Required | Description |
|----------|----------|-------------|
| `KEENTOOLS_API_URL` | Yes | API base URL |
| `KEENTOOLS_API_TOKEN` | Yes | Bearer token |
| `KEENTOOLS_INSTALL_VERSION` | No | CLI version to install (default: latest) |
| `KEENTOOLS_INSTALL_DIR` | No | Binary install directory |
| `KEENTOOLS_INSTALL_REPOSITORY` | No | GitHub repo override |

---

## 6. CI/CD Integration

Install and run in GitHub Actions:

```yaml
- name: Install keentools-cloud
  run: curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash
  
- name: Run reconstruction
  env:
    KEENTOOLS_API_URL:   ${{ vars.KEENTOOLS_API_URL }}
    KEENTOOLS_API_TOKEN: ${{ secrets.KEENTOOLS_API_TOKEN }}
  run: |
    keentools-cloud run photo1.jpg photo2.jpg -o result.glb --output json \
      | jq -r '"Model saved to \(.data.saved_to)"'

- name: Upload artifact
  uses: actions/upload-artifact@v4
  with:
    name: head-model
    path: result.glb
```

---

## Resources

- [Actionforge Agentic Coding Docs](https://docs.actionforge.dev/agentic-coding/)
- [keentools-cloud CLI Reference](./cli-quickstart.md)
- [IPC Qt Desktop Demo](./ipc-qt-demo.py)
- [IPC Web Browser Demo](./web-bridge.py)
