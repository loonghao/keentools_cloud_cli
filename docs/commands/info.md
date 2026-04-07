# info

Get camera metadata for a completed avatar reconstruction.

```bash
keentools-cloud info --avatar-id <ID>
```

## Options

| Flag               | Description              |
| ------------------ | ------------------------ |
| `--avatar-id <ID>` | Avatar ID (**required**) |

## Output

Returns camera matrices, focal lengths, and other reconstruction metadata.

```json
{
  "avatar_id": "abc123",
  "cameras": [
    {
      "focal_length": 50.0,
      "rotation": [[...], [...], [...]],
      "translation": [...]
    }
  ]
}
```
