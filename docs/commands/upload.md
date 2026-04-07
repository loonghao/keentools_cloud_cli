# upload

Upload photos to the presigned S3 URLs returned by `init`.

```bash
keentools-cloud upload --avatar-id <ID> --photos <FILE>... [OPTIONS]
```

## Options

| Flag                  | Description                                            |
| --------------------- | ------------------------------------------------------ |
| `--avatar-id <ID>`    | Avatar ID returned by `init` (**required**)            |
| `--photos <FILE>...`  | One or more photo file paths (**required**)            |
| `--urls <URL>...`     | Override presigned URLs (skips API lookup if provided) |

## Examples

```bash
# Upload 3 photos
keentools-cloud upload \
  --avatar-id abc123 \
  --photos front.jpg left.jpg right.jpg
```

## Notes

- Photos are uploaded directly to S3 (no API token sent to S3).
- A progress bar is shown in human mode (TTY).
- Photo count must match the `--count` used in `init`.
