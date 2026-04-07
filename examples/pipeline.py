#!/usr/bin/env python3
"""
KeenTools Cloud 3D Head Reconstruction — Full Pipeline Example (Python)

Replace BASE_URL and TOKEN with your actual values, then run:
    python pipeline.py photo1.jpg photo2.jpg photo3.jpg

Requirements:
    pip install requests
"""

import os
import sys
import time
import json
from pathlib import Path
import requests

# ─────────────────────────────────────────────
# Configuration — replace these values
# ─────────────────────────────────────────────
BASE_URL = os.environ.get(
    "KEENTOOLS_API_URL",
    "https://your-api-endpoint.example.com",  # <-- replace with your base URL
)
TOKEN = os.environ.get(
    "KEENTOOLS_API_TOKEN",
    "your_token_here",  # <-- replace with your token
)

OUTPUT_PATH = "output_head.glb"
BLENDSHAPES = "arkit,nose"          # comma-separated; omit to skip
TEXTURE_FORMAT = "jpg"              # jpg or png
FOCAL_LENGTH_TYPE = "estimate-per-image"  # or a numeric mm value

# ─────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────

HEADERS = {
    "Authorization": f"Bearer {TOKEN}",
    "Content-Type": "application/json",
}


def api(method: str, path: str, **kwargs) -> dict:
    """Make an authenticated API call and return the parsed JSON body."""
    url = f"{BASE_URL.rstrip('/')}{path}"
    resp = requests.request(method, url, headers=HEADERS, timeout=60, **kwargs)
    resp.raise_for_status()
    return resp.json()


def poll(path: str, done_status: str = "Completed", interval: int = 5) -> dict:
    """Poll a status endpoint until the job reaches *done_status*."""
    print(f"  Polling {path} …", end="", flush=True)
    while True:
        data = api("GET", path)
        status = data.get("status", "")
        print(f" {status}", end="", flush=True)
        if status == done_status:
            print()
            return data
        if status in ("Failed", "Error"):
            print()
            raise RuntimeError(f"Job failed: {json.dumps(data, indent=2)}")
        time.sleep(interval)


# ─────────────────────────────────────────────
# Pipeline steps
# ─────────────────────────────────────────────

def step_init(photo_count: int) -> tuple[str, list[str]]:
    """Initialize a session and obtain pre-signed S3 upload URLs."""
    print(f"\n[1/5] Initialising session for {photo_count} photo(s) …")
    data = api("POST", "/api/avatars/init", json={"photos_count": photo_count})
    avatar_id: str = data["avatar_id"]
    upload_urls: list[str] = data["upload_urls"]
    print(f"  avatar_id = {avatar_id}")
    return avatar_id, upload_urls


def step_upload(photo_paths: list[Path], upload_urls: list[str]) -> None:
    """Upload each photo to its corresponding pre-signed S3 URL."""
    print(f"\n[2/5] Uploading {len(photo_paths)} photo(s) …")
    for i, (path, url) in enumerate(zip(photo_paths, upload_urls), 1):
        print(f"  [{i}/{len(photo_paths)}] {path.name}")
        with path.open("rb") as fh:
            resp = requests.put(url, data=fh, timeout=120)
            resp.raise_for_status()
    print("  Upload complete.")


def step_process(avatar_id: str) -> None:
    """Trigger 3D reconstruction."""
    print("\n[3/5] Starting reconstruction …")
    payload: dict = {
        "focal_length_type": FOCAL_LENGTH_TYPE,
    }
    api("POST", f"/api/avatars/{avatar_id}/process", json=payload)
    print("  Reconstruction job submitted.")


def step_wait_status(avatar_id: str) -> None:
    """Wait until reconstruction is complete."""
    print("\n[4/5] Waiting for reconstruction …")
    poll(f"/api/avatars/{avatar_id}/status")


def step_download(avatar_id: str, output_path: str) -> None:
    """Download the finished 3D model."""
    print(f"\n[5/5] Downloading model → {output_path} …")
    params: dict = {"format": "glb"}
    if BLENDSHAPES:
        params["blendshapes"] = BLENDSHAPES
    if TEXTURE_FORMAT:
        params["texture"] = TEXTURE_FORMAT

    url = f"{BASE_URL.rstrip('/')}/api/avatars/{avatar_id}/model"

    # Poll until the model endpoint returns 200 (may still be packaging)
    print("  Waiting for model to be ready …", end="", flush=True)
    while True:
        resp = requests.get(url, headers=HEADERS, params=params, timeout=60, stream=True)
        if resp.status_code == 200:
            print(" ready.")
            break
        if resp.status_code in (202, 404):
            print(" …", end="", flush=True)
            time.sleep(5)
            continue
        resp.raise_for_status()

    dest = Path(output_path)
    total = int(resp.headers.get("Content-Length", 0))
    written = 0
    with dest.open("wb") as fh:
        for chunk in resp.iter_content(chunk_size=65536):
            fh.write(chunk)
            written += len(chunk)
            if total:
                pct = written * 100 // total
                print(f"\r  Downloading … {pct:3d}%", end="", flush=True)
    print(f"\r  Saved {written / 1_048_576:.1f} MB → {dest}")


# ─────────────────────────────────────────────
# Ephemeral pipeline (zero-retention alternative)
# ─────────────────────────────────────────────

def run_ephemeral(image_urls: list[str], result_put_url: str) -> str:
    """
    Ephemeral pipeline: images are never stored on KeenTools servers.

    Args:
        image_urls:     Publicly accessible URLs of the input photos.
        result_put_url: A pre-signed PUT URL where the GLB will be delivered.

    Returns:
        The avatar_id for subsequent status polling.
    """
    print("\n[Ephemeral] Submitting ephemeral job …")
    payload = {
        "image_urls": image_urls,
        "result_mesh_urls": [f"glb:{result_put_url}"],
        "focal_length_type": FOCAL_LENGTH_TYPE,
    }
    data = api("POST", "/api/avatars/ephemeral/create", json=payload)
    avatar_id: str = data["avatar_id"]
    print(f"  avatar_id = {avatar_id}")

    print("[Ephemeral] Waiting for completion …")
    poll(f"/api/avatars/{avatar_id}/status")
    print("[Ephemeral] Done — model delivered to your result URL.")
    return avatar_id


# ─────────────────────────────────────────────
# Entry point
# ─────────────────────────────────────────────

def main() -> None:
    photos = [Path(p) for p in sys.argv[1:]]
    if not photos:
        print("Usage: python pipeline.py <photo1.jpg> [photo2.jpg …]", file=sys.stderr)
        sys.exit(1)

    missing = [p for p in photos if not p.exists()]
    if missing:
        print(f"File(s) not found: {', '.join(str(p) for p in missing)}", file=sys.stderr)
        sys.exit(1)

    if TOKEN in ("your_token_here", "") or BASE_URL in ("https://your-api-endpoint.example.com", ""):
        print(
            "ERROR: Set KEENTOOLS_API_TOKEN and KEENTOOLS_API_URL environment variables\n"
            "  or edit the BASE_URL / TOKEN constants at the top of this file.",
            file=sys.stderr,
        )
        sys.exit(1)

    # ── Standard pipeline ──
    avatar_id, upload_urls = step_init(len(photos))
    step_upload(photos, upload_urls)
    step_process(avatar_id)
    step_wait_status(avatar_id)
    step_download(avatar_id, OUTPUT_PATH)

    print(f"\nDone! Model saved to: {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
