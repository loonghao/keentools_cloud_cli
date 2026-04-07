# self-update

Update `keentools-cloud` to the latest (or a specific) version.

```bash
keentools-cloud self-update [OPTIONS]
```

## Options

| Flag                | Description                                         |
| ------------------- | --------------------------------------------------- |
| `--check`           | Only check for updates; do not install              |
| `--version <VER>`   | Install a specific version (e.g. `v0.2.0`)          |
| `--force`           | Reinstall even if already on the latest version     |

## Examples

```bash
# Update to latest
keentools-cloud self-update

# Check without installing
keentools-cloud self-update --check

# Install a specific version
keentools-cloud self-update --version v0.2.0

# Force reinstall current version
keentools-cloud self-update --force
```

## How It Works

1. Fetches release info from the GitHub Releases API
2. Detects the current platform/architecture (embedded at compile time)
3. Downloads the matching `.tar.gz` or `.zip` asset
4. Verifies the downloaded archive
5. Replaces the current executable (atomic swap on Unix, rename on Windows)

## Notes

- Does **not** require `--api-url` or `--token`.
- On Windows, the old binary is renamed to `keentools-cloud.old.exe` before replacement.
- The installed binary must be writable by the current user.
