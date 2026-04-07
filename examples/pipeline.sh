#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# KeenTools Cloud 3D Head Reconstruction — Full Pipeline Example (Bash)
#
# Usage:
#   chmod +x pipeline.sh
#   ./pipeline.sh photo1.jpg photo2.jpg photo3.jpg
#
# Prerequisites:
#   curl, jq  (brew install jq  /  apt install jq)
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# ─────────────────────────────────────────────
# Configuration — replace these values or export as env vars before running
# ─────────────────────────────────────────────
BASE_URL="${KEENTOOLS_API_URL:-https://your-api-endpoint.example.com}"  # <-- replace
TOKEN="${KEENTOOLS_API_TOKEN:-your_token_here}"                         # <-- replace

OUTPUT_FILE="output_head.glb"
BLENDSHAPES="arkit,nose"              # comma-separated; set to "" to skip
TEXTURE_FORMAT="jpg"                  # jpg or png
FOCAL_LENGTH_TYPE="estimate-per-image"
POLL_INTERVAL=5                       # seconds between status polls

# ─────────────────────────────────────────────
# Validation
# ─────────────────────────────────────────────
if [[ "$TOKEN" == "your_token_here" || -z "$TOKEN" ]]; then
  echo "ERROR: Set KEENTOOLS_API_TOKEN (env var) or edit TOKEN in this script." >&2
  exit 1
fi
if [[ "$BASE_URL" == "https://your-api-endpoint.example.com" || -z "$BASE_URL" ]]; then
  echo "ERROR: Set KEENTOOLS_API_URL (env var) or edit BASE_URL in this script." >&2
  exit 1
fi
if [[ $# -eq 0 ]]; then
  echo "Usage: $0 <photo1.jpg> [photo2.jpg …]" >&2
  exit 1
fi

# Collect photos and check they exist
PHOTOS=("$@")
for f in "${PHOTOS[@]}"; do
  [[ -f "$f" ]] || { echo "File not found: $f" >&2; exit 1; }
done

PHOTO_COUNT=${#PHOTOS[@]}

# ─────────────────────────────────────────────
# Helper: authenticated curl wrapper
# ─────────────────────────────────────────────
api() {
  local method="$1"; shift
  local path="$1"; shift
  curl --silent --fail --show-error \
    -X "$method" \
    -H "Authorization: Bearer ${TOKEN}" \
    -H "Content-Type: application/json" \
    "${BASE_URL%/}${path}" \
    "$@"
}

# ─────────────────────────────────────────────
# Helper: poll a status endpoint until Completed
# ─────────────────────────────────────────────
poll_status() {
  local path="$1"
  local target_status="${2:-Completed}"
  printf "  Polling%s" ""
  while true; do
    local status
    status=$(api GET "$path" | jq -r '.status // "unknown"')
    printf " %s" "$status"
    if [[ "$status" == "$target_status" ]]; then
      echo
      return 0
    fi
    if [[ "$status" == "Failed" || "$status" == "Error" ]]; then
      echo
      echo "ERROR: Job failed with status: $status" >&2
      exit 1
    fi
    sleep "$POLL_INTERVAL"
  done
}

# ─────────────────────────────────────────────
# Step 1: Init — obtain avatar_id + upload URLs
# ─────────────────────────────────────────────
echo
echo "[1/5] Initialising session (${PHOTO_COUNT} photo(s)) …"

INIT_RESP=$(api POST "/api/avatars/init" --data-raw "{\"photos_count\": ${PHOTO_COUNT}}")
AVATAR_ID=$(echo "$INIT_RESP" | jq -r '.avatar_id')
echo "  avatar_id = ${AVATAR_ID}"

# Extract upload URLs into a bash array
mapfile -t UPLOAD_URLS < <(echo "$INIT_RESP" | jq -r '.upload_urls[]')

if [[ ${#UPLOAD_URLS[@]} -ne $PHOTO_COUNT ]]; then
  echo "ERROR: Expected ${PHOTO_COUNT} upload URLs, got ${#UPLOAD_URLS[@]}" >&2
  exit 1
fi

# ─────────────────────────────────────────────
# Step 2: Upload photos to pre-signed S3 URLs
# ─────────────────────────────────────────────
echo
echo "[2/5] Uploading ${PHOTO_COUNT} photo(s) …"

for i in "${!PHOTOS[@]}"; do
  photo="${PHOTOS[$i]}"
  url="${UPLOAD_URLS[$i]}"
  idx=$((i + 1))
  echo "  [${idx}/${PHOTO_COUNT}] $(basename "$photo")"
  curl --silent --fail --show-error \
    -X PUT \
    -H "Content-Type: image/jpeg" \
    --data-binary "@${photo}" \
    "$url"
done
echo "  Upload complete."

# ─────────────────────────────────────────────
# Step 3: Trigger reconstruction
# ─────────────────────────────────────────────
echo
echo "[3/5] Starting reconstruction …"
api POST "/api/avatars/${AVATAR_ID}/process" \
  --data-raw "{\"focal_length_type\": \"${FOCAL_LENGTH_TYPE}\"}" > /dev/null
echo "  Reconstruction job submitted."

# ─────────────────────────────────────────────
# Step 4: Wait for completion
# ─────────────────────────────────────────────
echo
echo "[4/5] Waiting for reconstruction …"
poll_status "/api/avatars/${AVATAR_ID}/status" "Completed"

# ─────────────────────────────────────────────
# Step 5: Download the 3D model
# ─────────────────────────────────────────────
echo
echo "[5/5] Downloading model → ${OUTPUT_FILE} …"

MODEL_PARAMS="format=glb"
[[ -n "$BLENDSHAPES" ]]     && MODEL_PARAMS+="&blendshapes=${BLENDSHAPES}"
[[ -n "$TEXTURE_FORMAT" ]]  && MODEL_PARAMS+="&texture=${TEXTURE_FORMAT}"

# Poll until model is ready (202 = packaging in progress)
printf "  Waiting for model to be ready …"
while true; do
  HTTP_CODE=$(curl --silent --output /dev/null --write-out "%{http_code}" \
    -H "Authorization: Bearer ${TOKEN}" \
    "${BASE_URL%/}/api/avatars/${AVATAR_ID}/model?${MODEL_PARAMS}")
  if [[ "$HTTP_CODE" == "200" ]]; then
    echo " ready."
    break
  elif [[ "$HTTP_CODE" == "202" || "$HTTP_CODE" == "404" ]]; then
    printf " …"
    sleep "$POLL_INTERVAL"
  else
    echo
    echo "ERROR: Unexpected HTTP ${HTTP_CODE} from model endpoint." >&2
    exit 1
  fi
done

curl --silent --fail --show-error \
  -H "Authorization: Bearer ${TOKEN}" \
  -o "$OUTPUT_FILE" \
  --progress-bar \
  "${BASE_URL%/}/api/avatars/${AVATAR_ID}/model?${MODEL_PARAMS}"

SIZE_MB=$(du -m "$OUTPUT_FILE" 2>/dev/null | cut -f1 || echo "?")
echo "  Saved ${SIZE_MB} MB → ${OUTPUT_FILE}"

echo
echo "Done! Model saved to: ${OUTPUT_FILE}"
