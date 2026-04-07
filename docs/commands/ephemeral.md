# ephemeral

Zero-retention reconstruction pipeline. Images are processed without being
stored on the server — you provide pre-signed upload URLs for results.

```bash
keentools-cloud ephemeral [OPTIONS]
```

## Options

| Flag                     | Description                                                   |
| ------------------------ | ------------------------------------------------------------- |
| `--image-url <URL>`      | HTTPS URL of a source photo (repeat for multiple, **required at least 2**) |
| `--result-url <FMT:URL>` | `FORMAT:URL` for output (e.g. `obj:https://...`) (**required**) |
| `--poll`                 | Poll until the job completes                                  |
| `--poll-interval <SECS>` | Seconds between polls (default: `5`)                          |

## Format Values for `--result-url`

`obj`, `fbx`, `glb`

## Examples

```bash
keentools-cloud ephemeral \
  --image-url https://cdn.example.com/front.jpg \
  --image-url https://cdn.example.com/left.jpg \
  --image-url https://cdn.example.com/right.jpg \
  --result-url obj:https://s3.example.com/result.obj?X-Amz-Signature=...
```

## Notes

- All URLs must be `https://`.
- The `--result-url` format is `FORMAT:URL` split on the **first** colon.
- Use `--poll` if you want to wait for the async job to complete.
