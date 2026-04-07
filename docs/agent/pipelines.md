# Pipelines

## Standard Pipeline

```
init → upload → process → status (poll) → download
```

### As a single command

```bash
keentools-cloud run \
  --photos front.jpg left.jpg right.jpg \
  --output-path head.obj \
  --output json
```

### Step-by-step (agent-controlled)

```bash
# Step 1: init
INIT=$(keentools-cloud --output json init --count 3)
AVATAR_ID=$(echo $INIT | jq -r '.avatar_id')
URLS=$(echo $INIT | jq -r '.upload_urls[]')

# Step 2: upload (pass URLs explicitly to avoid second API call)
keentools-cloud upload \
  --avatar-id "$AVATAR_ID" \
  --photos front.jpg left.jpg right.jpg

# Step 3: process
keentools-cloud --output json process --avatar-id "$AVATAR_ID"

# Step 4: poll status
keentools-cloud --output json status --avatar-id "$AVATAR_ID" --poll

# Step 5: download
keentools-cloud --output json download \
  --avatar-id "$AVATAR_ID" \
  --output-path head.obj
```

## Ephemeral Pipeline

Zero-retention — no data stored server-side:

```bash
keentools-cloud --output json ephemeral \
  --image-url https://cdn.example.com/front.jpg \
  --image-url https://cdn.example.com/left.jpg \
  --result-url obj:https://s3.example.com/result.obj?... \
  --poll
```

## Error Handling

Check exit code after each step:

```bash
keentools-cloud --output json process --avatar-id "$AVATAR_ID"
if [ $? -ne 0 ]; then
  echo "process failed" >&2
  exit 1
fi
```

Or capture stderr for structured error info:

```bash
ERR=$(keentools-cloud --output json process --avatar-id "$AVATAR_ID" 2>&1 >/dev/null)
```
