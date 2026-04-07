# status

Check the reconstruction status of an avatar.

```bash
keentools-cloud status --avatar-id <ID> [OPTIONS]
```

## Options

| Flag                      | Default | Description                                |
| ------------------------- | ------- | ------------------------------------------ |
| `--avatar-id <ID>`        | —       | Avatar ID (**required**)                   |
| `--poll`                  | false   | Keep polling until completed or failed     |
| `--poll-interval <SECS>`  | `5`     | Seconds between polls (used with `--poll`) |

## Status Values

| Status        | Meaning                              |
| ------------- | ------------------------------------ |
| `not_started` | Job queued, not yet running          |
| `running`     | Reconstruction in progress           |
| `completed`   | Reconstruction successful            |
| `failed`      | Reconstruction failed                |
| `deleted`     | Avatar has been deleted              |

## Examples

```bash
# Check once
keentools-cloud status --avatar-id abc123

# Poll until done
keentools-cloud status --avatar-id abc123 --poll

# Poll with custom interval
keentools-cloud status --avatar-id abc123 --poll --poll-interval 10
```

## JSON Output

```json
{ "status": "completed" }
```

```json
{ "status": "running", "progress": 42 }
```
