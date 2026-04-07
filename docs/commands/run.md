# run

Run the complete reconstruction pipeline in a single command:
`init → upload → process → poll status → download`.

```bash
keentools-cloud run --photos <FILE>... --output-path <PATH> [OPTIONS]
```

## Options

| Flag                         | Description                                              |
| ---------------------------- | -------------------------------------------------------- |
| `--photos <FILE>...`         | Photo files to upload (**required**)                     |
| `--output-path <PATH>`       | Output file path (**required**)                          |
| `--format <FMT>`             | Mesh format: `obj` \| `fbx` \| `glb` (default: `obj`)   |
| `--blendshapes <LIST>`       | Comma-separated blendshape names                         |
| `--texture`                  | Include texture map                                      |
| `--focal-length-type <TYPE>` | `auto` \| `fixed` \| `per-photo`                         |
| `--focal-lengths <F>...`     | Focal lengths in mm                                      |
| `--expressions`              | Enable facial expressions                                |
| `--poll-interval <SECS>`     | Seconds between status polls (default: `5`)              |
| `--dry-run`                  | Validate inputs without calling API                      |

## Examples

```bash
# Simplest usage
keentools-cloud run \
  --photos front.jpg left.jpg right.jpg \
  --output-path head.obj

# FBX with texture and ARKit blendshapes
keentools-cloud run \
  --photos *.jpg \
  --output-path head.fbx \
  --format fbx --texture \
  --blendshapes arkit,expression

# Dry run (validate inputs, no API calls)
keentools-cloud run \
  --photos front.jpg left.jpg \
  --output-path head.obj \
  --dry-run
```

## Notes

- Equivalent to running `init`, `upload`, `process`, `status --poll`, and `download` in sequence.
- Progress is shown at each stage in human mode.
