# init

Initialize a new 3D head reconstruction job.

```bash
keentools-cloud init [OPTIONS]
```

## Options

| Flag            | Default | Description                                |
| --------------- | ------- | ------------------------------------------ |
| `--count <N>`   | `3`     | Number of photos to upload (2–15)          |
| `--dry-run`     | false   | Print what would happen without calling API |

## Output

```json
{
  "avatar_id": "abc123",
  "upload_urls": [
    "https://s3.amazonaws.com/bucket/...",
    "https://s3.amazonaws.com/bucket/...",
    "https://s3.amazonaws.com/bucket/..."
  ]
}
```

## Examples

```bash
# Initialize a job for 5 photos
keentools-cloud init --count 5

# Dry run (no API call)
keentools-cloud init --count 3 --dry-run

# Force JSON output
keentools-cloud init --output json | jq '.avatar_id'
```

## Notes

- The `avatar_id` is needed for all subsequent commands.
- `upload_urls` are presigned S3 PUT URLs; use them with `upload`.
- Photos must be `--count` exactly (matched positionally to URLs).
