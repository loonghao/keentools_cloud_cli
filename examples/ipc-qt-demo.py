#!/usr/bin/env python3
"""
KeenTools Cloud — Qt Desktop Integration Demo

Demonstrates wrapping keentools-cloud CLI with `--ipc` mode in a PySide6/PyQt6
desktop application. The CLI emits NDJSON progress events on stdout; this demo
parses them in real-time and updates a progress bar + log panel.

Requirements:
    pip install PySide6  # or: pip install PyQt6

Usage:
    python ipc-qt-demo.py

Then use the UI to:
    1. Select 2-15 photos
    2. Choose output path
    3. Click "Start Reconstruction"
    4. Watch real-time NDJSON-driven progress updates

The key insight: `keentools-cloud run ... --ipc` outputs one JSON object per line
(NDJSON), which we read asynchronously from the subprocess stdout. Each line is a
progress event that maps directly to UI state.
"""

import json
import subprocess
import sys
from pathlib import Path

try:
    from PySide6.QtWidgets import (
        QApplication,
        QMainWindow,
        QWidget,
        QVBoxLayout,
        QHBoxLayout,
        QLabel,
        QPushButton,
        QProgressBar,
        QTextEdit,
        QFileDialog,
        QListWidget,
        QGroupBox,
        QSpinBox,
        QLineEdit,
        QMessageBox,
    )
    from PySide6.QtCore import QProcess, Qt, Signal, QObject
except ImportError:
    from PyQt6.QtWidgets import (
        QApplication,
        QMainWindow,
        QWidget,
        QVBoxLayout,
        QHBoxLayout,
        QLabel,
        QPushButton,
        QProgressBar,
        QTextEdit,
        QFileDialog,
        QListWidget,
        QGroupBox,
        QSpinBox,
        QLineEdit,
        QMessageBox,
    )
    from PyQt6.QtCore import QProcess, Qt, pyqtSignal as Signal, QObject


# ---------------------------------------------------------------------------
# NDJSON event parser — reads line-by-line from CLI stdout
# ---------------------------------------------------------------------------

class IPCEventEmitter(QObject):
    """Emits parsed signals for each NDJSON line from keentools-cloud --ipc."""

    progress = Signal(str, int, str)   # stage, percent, message
    error = Signal(str, str)           # stage, message
    complete = Signal(str, int, str)   # stage, percent, saved_to
    raw_line = Signal(str)             # raw text for debug log


def _parse_ndjson_line(line: str, emitter: IPCEventEmitter) -> None:
    """Parse a single NDJSON line and emit the appropriate signal."""
    line = line.strip()
    if not line:
        return

    emitter.raw_line.emit(line)

    try:
        event = json.loads(line)
    except json.JSONDecodeError:
        return  # skip malformed lines

    etype = event.get("type", "")
    stage = event.get("stage", "")
    percent = event.get("percent", 0)
    message = event.get("message", "")

    if etype == "progress":
        emitter.progress.emit(stage, percent, message)
    elif etype == "error":
        emitter.error.emit(stage, message)
    elif etype == "complete":
        saved_to = event.get("saved_to", "")
        emitter.complete.emit(stage, percent, saved_to)


# ---------------------------------------------------------------------------
# Main Window
# ---------------------------------------------------------------------------

class MainWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("KeenTools Cloud — 3D Head Reconstruction")
        self.setMinimumSize(720, 560)
        self._photos: list[Path] = []
        self._process: QProcess | None = None
        self._ipc_emitter = IPCEventEmitter()

        # Connect signals
        self._ipc_emitter.progress.connect(self._on_progress)
        self._ipc_emitter.error.connect(self._on_error)
        self._ipc_emitter.complete.connect(self._on_complete)
        self._ipc_emitter.raw_line.connect(self._on_raw_line)

        self._setup_ui()

    # ---- UI layout ----------------------------------------------------------

    def _setup_ui(self):
        central = QWidget()
        self.setCentralWidget(central)
        root = QVBoxLayout(central)

        # --- Config group ---
        config_group = QGroupBox("Configuration")
        config_layout = QVBoxLayout(config_group)

        # API URL
        url_row = QHBoxLayout()
        url_row.addWidget(QLabel("API URL:"))
        self._api_url = QLineEdit()
        self._api_url.setPlaceholderText("https://your-api-endpoint.example.com")
        self._api_url.setText(
            __import__("os").environ.get("KEENTOOLS_API_URL", "")
        )
        url_row.addWidget(self._api_url)
        config_layout.addLayout(url_row)

        # Token
        token_row = QHBoxLayout()
        token_row.addWidget(QLabel("Token:"))
        self._token = QLineEdit()
        self._token.setEchoMode(QLineEdit.EchoMode.Password)
        self._token.setPlaceholderText("KEENTOOLS_API_TOKEN or leave empty for config")
        self._token.setText(
            __import__("os").environ.get("KEENTOOLS_API_TOKEN", "")
        )
        token_row.addWidget(self._token)
        config_layout.addLayout(token_row)

        root.addWidget(config_group)

        # --- Photo selection ---
        photo_group = QGroupBox("Input Photos (2–15)")
        photo_layout = QVBoxLayout(photo_group)

        btn_row = QHBoxLayout()
        self._btn_add = QPushButton("Add Photos...")
        self._btn_add.clicked.connect(self._add_photos)
        self._btn_clear = QPushButton("Clear")
        self._btn_clear.clicked.connect(self._clear_photos)
        btn_row.addWidget(self._btn_add)
        btn_row.addWidget(self._btn_clear)
        btn_row.addStretch()
        photo_layout.addLayout(btn_row)

        self._photo_list = QListWidget()
        self._photo_list.setMinimumHeight(100)
        photo_layout.addWidget(self._photo_list)

        self._count_label = QLabel("0 photos selected")
        photo_layout.addWidget(self._count_label)

        root.addWidget(photo_group)

        # --- Options ---
        opts_group = QGroupBox("Options")
        opts_layout = QHBoxLayout(opts_group)

        opts_layout.addWidget(QLabel("Output:"))
        self._output_path = QLineEdit("output_head.glb")
        opts_layout.addWidget(self._output_path)

        self._btn_browse = QPushButton("Browse...")
        self._btn_browse.clicked.connect(self._browse_output)
        opts_layout.addWidget(self._btn_browse)

        opts_layout.addWidget(QLabel("Blendshapes:"))
        self._blendshapes = QLineEdit("arkit,nose")
        self._blendshapes.setToolTip("Comma-separated: arkit, expression, nose")
        opts_layout.addWidget(self._blendshapes)

        root.addWidget(opts_group)

        # --- Progress ---
        prog_group = QGroupBox("Progress")
        prog_layout = QVBoxLayout(prog_group)

        self._stage_label = QLabel("Ready")
        self._stage_label.setStyleSheet("font-weight: bold;")
        prog_layout.addWidget(self._stage_label)

        self._progress_bar = QProgressBar()
        self._progress_bar.setRange(0, 100)
        self._progress_bar.setValue(0)
        prog_layout.addWidget(self._progress_bar)

        self._status_label = QLabel("")
        prog_layout.addWidget(self._status_label)

        root.addWidget(prog_group)

        # --- Action buttons ---
        action_row = QHBoxLayout()
        self._btn_run = QPushButton("▶ Start Reconstruction")
        self._btn_run.setStyleSheet("font-size: 14px; padding: 8px; font-weight: bold;")
        self._btn_run.clicked.connect(self._start_reconstruction)
        self._btn_stop = QPushButton("⏹ Stop")
        self._btn_stop.setEnabled(False)
        self._btn_stop.clicked.connect(self._stop_reconstruction)
        action_row.addWidget(self._btn_run)
        action_row.addWidget(self._btn_stop)
        root.addLayout(action_row)

        # --- Log viewer ---
        log_group = QGroupBox("NDJSON Event Log")
        log_layout = QVBoxLayout(log_group)
        self._log = QTextEdit()
        self._log.setReadOnly(True)
        self._log.setMaximumHeight(150)
        self._log.setFont(__import__("QtWidgets" if "PySide6" in sys.modules else "QtWidgets").QApplication.font().__class__("Monospace") if False else self._log.font())  # keep default
        log_layout.addWidget(self._log)
        root.addWidget(log_group)

    # ---- Slots ---------------------------------------------------------------

    def _add_photos(self):
        paths, _ = QFileDialog.getOpenFileNames(
            self, "Select Photos", "", "Images (*.jpg *.jpeg *.png *.heic *.heif);;All Files (*)"
        )
        for p in paths:
            pp = Path(p)
            if pp not in self._photos and len(self._photos) < 15:
                self._photos.append(pp)
        self._refresh_photo_list()

    def _clear_photos(self):
        self._photos.clear()
        self._refresh_photo_list()

    def _refresh_photo_list(self):
        self._photo_list.clear()
        for p in self._photos:
            self._photo_list.addItem(p.name)
        self._count_label.setText(f"{len(self._photos)} photos selected")

    def _browse_output(self):
        path, _ = QFileDialog.getSaveFileName(
            self, "Save Model As", self._output_path.text(),
            "GLB (*.glb);;OBJ (*.obj)"
        )
        if path:
            self._output_path.setText(path)

    def _start_reconstruction(self):
        """Build and launch `keentools-cloud run ... --ipc` as a subprocess."""
        if len(self._photos) < 2:
            QMessageBox.warning(self, "Error", "Select at least 2 photos.")
            return
        if len(self._photos) > 15:
            QMessageBox.warning(self, "Error", "Maximum 15 photos allowed.")
            return

        output = self._output_path.text().strip()
        if not output:
            QMessageBox.warning(self, "Error", "Set an output file path.")
            return

        # Build CLI command
        cmd = ["keentools-cloud", "run"]
        cmd += [str(p) for p in self._photos]
        cmd += ["-o", output]
        cmd += ["--ipc"]  # Enable NDJSON progress events

        api_url = self._api_url.text().strip()
        if api_url:
            cmd += ["--api-url", api_url]

        token = self._token.text().strip()
        if token:
            cmd += ["--token", token]

        bs = self._blendshapes.text().strip()
        if bs:
            cmd += ["--blendshapes", bs]

        # Launch subprocess
        self._process = QProcess()
        self._process.setProcessChannelMode(
            QProcess.ProcessChannelMode.MergedChannelsOutput
        )
        self._process.readyReadStandardOutput.connect(self._read_stdout)
        self._process.finished.connect(self._on_finished)

        self._log.clear()
        self._progress_bar.setValue(0)
        self._stage_label.setText("Starting...")
        self._btn_run.setEnabled(False)
        self._btn_stop.setEnabled(True)

        self._log.append(f"$ {' '.join(cmd)}\n{'─' * 60}")
        self._process.start(cmd[0], cmd[1:])

    def _stop_reconstruction(self):
        if self._process and self._process.state() != QProcess.ProcessState.NotRunning:
            self._process.kill()
            self._log.append("\n--- STOPPED by user ---")

    def _read_stdout(self):
        """Read available stdout data, split by lines, parse each as NDJSON."""
        if not self._process:
            return
        data = bytes(self._process.readAllStandardOutput()).decode("utf-8", errors="replace")
        for line in data.splitlines():
            _parse_ndjson_line(line, self._ipc_emitter)

    def _on_progress(self, stage: str, percent: int, message: str):
        self._stage_label.setText(f"[{stage}] {percent}%")
        self._progress_bar.setValue(percent)
        self._status_label.setText(message)

    def _on_error(self, stage: str, message: str):
        self._stage_label.setText(f"[{stage}] ERROR")
        self._status_label.setText(f"Error: {message}")
        self._status_label.setStyleSheet("color: red;")
        self._log.append(f"\n✗ ERROR [{stage}]: {message}")

    def _on_complete(self, stage: str, percent: int, saved_to: str):
        self._stage_label.setText("[done] Complete!")
        self._progress_bar.setValue(100)
        self._status_label.setText(f"Saved to {saved_to}")
        self._status_label.setStyleSheet("color: green;")
        self._log.append(f"\n✓ Done! Saved to {saved_to}")

    def _on_raw_line(self, line: str):
        self._log.append(f"  ← {line[:200]}")

    def _on_finished(self, exit_code: int, exit_status: QProcess.ExitStatus):
        self._btn_run.setEnabled(True)
        self._btn_stop.setEnabled(False)
        if exit_code != 0:
            stderr = bytes(self._process.readAllStandardError()).decode(errors="replace") if self._process else ""
            self._log.append(f"\n✗ Process exited with code {exit_code}")
            if stderr:
                self._log.append(f"  stderr: {stderr[:500]}")


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

def main():
    app = QApplication(sys.argv)
    app.setApplicationName("KeenTools Cloud Demo")
    window = MainWindow()
    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
