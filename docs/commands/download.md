# download

Download a completed 3D model.

```bash
keentools-cloud download --avatar-id <ID> --output-path <PATH> [OPTIONS]
```

## Options

| Flag                       | Default  | Description                                       |
| -------------------------- | -------- | ------------------------------------------------- |
| `--avatar-id <ID>`         | —        | Avatar ID (**required**)                          |
| `--output-path <PATH>`     | —        | Where to save the file (**required**)             |
| `--format <FMT>`           | `obj`    | Mesh format: `obj` \| `fbx` \| `glb`             |
| `--blendshapes <LIST>`     | —        | Comma-separated blendshape names                  |
| `--texture`                | false    | Include texture map                               |
| `--edges`                  | false    | Include edge data                                 |
| `--poll`                   | false    | Poll until model is ready (if not yet complete)   |

## Examples

```bash
# Basic download
keentools-cloud download --avatar-id abc123 --output-path head.obj

# FBX with texture
keentools-cloud download --avatar-id abc123 --output-path head.fbx \
  --format fbx --texture

# With ARKit blendshapes
keentools-cloud download --avatar-id abc123 --output-path head.obj \
  --blendshapes arkit,expression

# Poll until ready, then download
keentools-cloud download --avatar-id abc123 --output-path head.obj --poll
```

## Notes

- Blendshapes are passed as a **comma-separated** string, not repeated flags.
- The command follows redirect/retry-after events from the API automatically.
