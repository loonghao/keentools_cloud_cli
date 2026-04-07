# Quick Start

## 1. Install

```bash
curl -fsSL https://raw.githubusercontent.com/loonghao/keentools_cloud_cli/main/install.sh | bash
```

## 2. Configure

```bash
export KEENTOOLS_API_URL=https://your-api-endpoint.example.com

# Option A: env var
export KEENTOOLS_API_TOKEN=your-token-here

# Option B: saved config
keentools-cloud auth login
```

## 3. Full Pipeline (One Command)

```bash
keentools-cloud run \
  --photos front.jpg side_left.jpg side_right.jpg \
  --output-path head.obj
```

## 4. Step-by-Step Pipeline

```bash
# Initialize reconstruction job
keentools-cloud init --count 3
# Returns: { "avatar_id": "abc123", "upload_urls": [...] }

# Upload photos
keentools-cloud upload --avatar-id abc123 \
  --photos front.jpg side_left.jpg side_right.jpg

# Start reconstruction
keentools-cloud process --avatar-id abc123

# Poll until complete
keentools-cloud status --avatar-id abc123 --poll

# Download the 3D model
keentools-cloud download --avatar-id abc123 --output-path head.obj
```

## 5. Ephemeral Pipeline (Zero-Retention)

Process without storing data on the server:

```bash
keentools-cloud ephemeral \
  --image-url https://example.com/front.jpg \
  --image-url https://example.com/side.jpg \
  --result-url obj:https://s3.example.com/head.obj?presigned=...
```

## 6. Check API Schema (for Agent Use)

```bash
keentools-cloud schema
keentools-cloud schema --command init
```
