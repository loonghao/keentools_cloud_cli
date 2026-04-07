# process

Start 3D head reconstruction for an uploaded avatar.

```bash
keentools-cloud process --avatar-id <ID> [OPTIONS]
```

## Options

| Flag                         | Description                                         |
| ---------------------------- | --------------------------------------------------- |
| `--avatar-id <ID>`           | Avatar ID (**required**)                            |
| `--focal-length-type <TYPE>` | `auto` \| `fixed` \| `per-photo` (default: `auto`) |
| `--focal-lengths <F>...`     | Focal lengths in mm (for `fixed`/`per-photo` modes) |
| `--expressions`              | Enable facial expressions in the output mesh        |
| `--dry-run`                  | Print payload without calling API                   |

## Examples

```bash
# Auto focal length
keentools-cloud process --avatar-id abc123

# Fixed focal length (same for all photos)
keentools-cloud process --avatar-id abc123 \
  --focal-length-type fixed --focal-lengths 50

# Per-photo focal lengths
keentools-cloud process --avatar-id abc123 \
  --focal-length-type per-photo --focal-lengths 50 48 52

# With blendshapes/expressions
keentools-cloud process --avatar-id abc123 --expressions
```
