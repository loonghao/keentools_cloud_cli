# auth

Manage API token authentication.

## Subcommands

### `auth login`

Save an API token to the config file.

```bash
keentools-cloud auth login
# Prompts: Enter your API token:
```

### `auth logout`

Remove the saved token from the config file.

```bash
keentools-cloud auth logout
```

### `auth status`

Show the current authentication status (token source and masked value).

```bash
keentools-cloud auth status
```

**Example output (human):**

```
Token source : config file
Token        : abcd...wxyz
```

**Example output (JSON):**

```json
{
  "status": "authenticated",
  "source": "config_file",
  "token_preview": "abcd...wxyz"
}
```

## Notes

- The `auth` command does not require `--api-url` or `--token` flags.
- Token priority: `--token` flag / `KEENTOOLS_API_TOKEN` env var → config file.
