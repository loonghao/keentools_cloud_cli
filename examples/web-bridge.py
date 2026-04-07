#!/usr/bin/env python3
"""
KeenTools Cloud — Web Browser Integration Demo

Bridges keentools-cloud CLI `--ipc` NDJSON events to a browser frontend
via Server-Sent Events (SSE). The browser receives real-time progress
updates and displays them in a progress bar.

Architecture:
    Browser ←─ SSE ─── Flask server ←─ stdout (NDJSON) ─── keentools-cloud run --ipc

Requirements:
    pip install flask

Usage:
    python web-bridge.py
    # Open http://localhost:5000 in your browser

Environment:
    KEENTOOLS_API_URL   — required
    KEENTOOLS_API_TOKEN — required (or pass via form)
"""

import json
import os
import subprocess
import threading
import queue
import tempfile
from pathlib import Path

from flask import Flask, Response, request, stream_with_context

app = Flask(__name__)

# ---------------------------------------------------------------------------
# HTML frontend (single-file, no build step)
# ---------------------------------------------------------------------------

_HTML = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>KeenTools Cloud — Web Demo</title>
<style>
  body { font-family: system-ui, sans-serif; max-width: 700px; margin: 40px auto; padding: 0 16px; }
  h1   { margin-bottom: 4px; }
  .sub { color: #666; margin-bottom: 24px; font-size: 14px; }
  label { display: block; margin-bottom: 4px; font-weight: 600; font-size: 14px; }
  input, select { width: 100%; padding: 8px; box-sizing: border-box; border: 1px solid #ccc;
                  border-radius: 4px; margin-bottom: 12px; font-size: 14px; }
  button { padding: 10px 24px; background: #0057ff; color: #fff; border: none;
           border-radius: 4px; cursor: pointer; font-size: 14px; font-weight: 600; }
  button:disabled { background: #aaa; cursor: not-allowed; }
  .progress-wrap { margin: 20px 0; }
  progress { width: 100%; height: 20px; }
  #stage    { font-weight: 700; margin-bottom: 4px; }
  #msg      { font-size: 13px; color: #444; min-height: 20px; }
  #log      { background: #1e1e1e; color: #d4d4d4; padding: 12px; border-radius: 4px;
              font-family: monospace; font-size: 12px; height: 220px; overflow-y: auto;
              white-space: pre-wrap; margin-top: 16px; }
  .ok   { color: #4ec9b0; }
  .err  { color: #f44747; }
  .done { color: #b5cea8; }
</style>
</head>
<body>
<h1>KeenTools Cloud</h1>
<p class="sub">3D Head Reconstruction via <code>keentools-cloud run --ipc</code></p>

<label>API URL</label>
<input id="apiUrl" value="" placeholder="https://your-api-endpoint.example.com">

<label>API Token</label>
<input id="token" type="password" placeholder="or set KEENTOOLS_API_TOKEN env var">

<label>Photos (2-15 JPEG/PNG files)</label>
<input id="photos" type="file" accept=".jpg,.jpeg,.png,.heic" multiple>

<label>Blendshapes</label>
<input id="blendshapes" value="arkit,nose" placeholder="arkit,nose,expression">

<button id="btn" onclick="run()">▶ Start Reconstruction</button>

<div class="progress-wrap">
  <div id="stage">Ready</div>
  <progress id="bar" value="0" max="100"></progress>
  <div id="msg"></div>
</div>

<div id="log">Press "Start Reconstruction" to begin...\n</div>

<script>
let evtSource = null;

function log(text, cls) {
  const el = document.getElementById('log');
  const line = document.createElement('span');
  if (cls) line.className = cls;
  line.textContent = text + '\n';
  el.appendChild(line);
  el.scrollTop = el.scrollHeight;
}

async function run() {
  const photos = document.getElementById('photos').files;
  if (photos.length < 2) { alert('Select at least 2 photos'); return; }
  if (photos.length > 15) { alert('Maximum 15 photos'); return; }

  const btn = document.getElementById('btn');
  btn.disabled = true;

  // Upload photos to the Flask server
  const form = new FormData();
  for (const f of photos) form.append('photos', f);
  form.append('api_url', document.getElementById('apiUrl').value);
  form.append('token',   document.getElementById('token').value);
  form.append('blendshapes', document.getElementById('blendshapes').value);

  document.getElementById('log').textContent = '';
  log('Uploading photos to server...');

  const resp = await fetch('/start', { method: 'POST', body: form });
  if (!resp.ok) {
    log('✗ Server error: ' + await resp.text(), 'err');
    btn.disabled = false;
    return;
  }
  const { job_id } = await resp.json();
  log('Job started: ' + job_id);

  // Subscribe to SSE stream
  if (evtSource) evtSource.close();
  evtSource = new EventSource('/events/' + job_id);

  evtSource.onmessage = function(e) {
    try {
      const ev = JSON.parse(e.data);
      const type = ev.type || '';

      if (type === 'progress') {
        document.getElementById('stage').textContent = '[' + ev.stage + '] ' + ev.percent + '%';
        document.getElementById('bar').value = ev.percent;
        document.getElementById('msg').textContent = ev.message || '';
        log('  ← ' + e.data, 'ok');

      } else if (type === 'error') {
        document.getElementById('stage').textContent = '✗ ERROR';
        document.getElementById('msg').textContent = ev.message;
        document.getElementById('msg').style.color = 'red';
        log('✗ [' + ev.stage + '] ' + ev.message, 'err');
        evtSource.close();
        btn.disabled = false;

      } else if (type === 'complete') {
        document.getElementById('stage').textContent = '✓ Complete!';
        document.getElementById('bar').value = 100;
        document.getElementById('msg').textContent = 'Saved to ' + ev.saved_to;
        document.getElementById('msg').style.color = 'green';
        log('✓ Done! Saved to: ' + ev.saved_to, 'done');
        evtSource.close();
        btn.disabled = false;
      }
    } catch (_) {
      log('  raw: ' + e.data);
    }
  };

  evtSource.onerror = function() {
    log('SSE connection closed.', 'err');
    btn.disabled = false;
  };
}
</script>
</body>
</html>
"""


# ---------------------------------------------------------------------------
# In-memory job store (production would use Redis / DB)
# ---------------------------------------------------------------------------

_jobs: dict[str, queue.Queue] = {}


# ---------------------------------------------------------------------------
# Routes
# ---------------------------------------------------------------------------

@app.route("/")
def index():
    return _HTML


@app.route("/start", methods=["POST"])
def start():
    """
    Accept uploaded photos, save to a temp dir, launch keentools-cloud run --ipc,
    and stream NDJSON events into a per-job queue read by /events/<job_id>.
    """
    photos = request.files.getlist("photos")
    api_url = request.form.get("api_url", "").strip() or os.environ.get("KEENTOOLS_API_URL", "")
    token = request.form.get("token", "").strip() or os.environ.get("KEENTOOLS_API_TOKEN", "")
    blendshapes = request.form.get("blendshapes", "").strip()

    if not api_url:
        return {"error": "api_url is required"}, 400
    if len(photos) < 2 or len(photos) > 15:
        return {"error": "need 2-15 photos"}, 400

    # Save photos to temp dir
    tmpdir = tempfile.mkdtemp(prefix="keentools_")
    photo_paths = []
    for f in photos:
        dest = Path(tmpdir) / f.filename
        f.save(dest)
        photo_paths.append(str(dest))

    output_path = str(Path(tmpdir) / "head.glb")

    # Build CLI command
    cmd = ["keentools-cloud", "run"] + photo_paths + [
        "-o", output_path,
        "--ipc",
        "--api-url", api_url,
    ]
    if token:
        cmd += ["--token", token]
    if blendshapes:
        cmd += ["--blendshapes", blendshapes]

    job_id = f"job_{len(_jobs) + 1:04d}"
    event_queue: queue.Queue = queue.Queue()
    _jobs[job_id] = event_queue

    def _run():
        try:
            proc = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1,
            )
            for line in proc.stdout:
                line = line.strip()
                if line:
                    event_queue.put(line)
            proc.wait()
        except Exception as exc:
            event_queue.put(json.dumps({"type": "error", "stage": "server", "message": str(exc)}))
        finally:
            event_queue.put(None)  # sentinel — stream done

    threading.Thread(target=_run, daemon=True).start()
    return {"job_id": job_id}


@app.route("/events/<job_id>")
def events(job_id: str):
    """
    Server-Sent Events stream. Reads NDJSON lines from the job queue and
    forwards them to the browser as SSE data frames.
    """
    if job_id not in _jobs:
        return {"error": "unknown job"}, 404

    event_queue = _jobs[job_id]

    @stream_with_context
    def _generate():
        while True:
            line = event_queue.get()
            if line is None:
                break  # subprocess finished
            # SSE format: "data: <payload>\n\n"
            yield f"data: {line}\n\n"

    return Response(
        _generate(),
        mimetype="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "X-Accel-Buffering": "no",
        },
    )


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    print("KeenTools Cloud Web Demo")
    print("Open: http://localhost:5000")
    print()
    print("Set credentials via env vars or the browser form:")
    print("  export KEENTOOLS_API_URL=https://your-api-endpoint.example.com")
    print("  export KEENTOOLS_API_TOKEN=your-token-here")
    print()
    app.run(debug=False, port=5000, threaded=True)
