"""Session management for saving, restoring, and exporting drawings."""
from __future__ import annotations
import json
import logging
from pathlib import Path
import time
from typing import Dict, List, Optional
import cairo

from clink.protocol.messages import Stroke
from clink.overlay.renderer import OverlayRenderer

logger = logging.getLogger(__name__)

SESSIONS_DIR = Path.home() / ".config" / "clink" / "sessions"

class SessionManager:
    def __init__(self, sessions_dir: Path = SESSIONS_DIR):
        self.sessions_dir = sessions_dir
        self.sessions_dir.mkdir(parents=True, exist_ok=True)

    def save_session(self, strokes: List[Stroke], session_name: Optional[str] = None) -> Path:
        name = session_name or f"session_{int(time.time())}"
        file_path = self.sessions_dir / f"{name}.json"
        data = {
            "version": "1.0",
            "created_at": int(time.time()),
            "stroke_count": len(strokes),
            "strokes": [s.to_dict() for s in strokes]
        }
        with open(file_path, "w", encoding="utf-8") as f:
            json.dump(data, f, indent=2)
        logger.info("Session saved: %s", file_path)
        return file_path

    def load_session(self, file_path: Path) -> List[Stroke]:
        with open(file_path, "r", encoding="utf-8") as f:
            data = json.load(f)
        strokes = [Stroke.from_dict(s) for s in data.get("strokes", [])]
        logger.info("Loaded %d strokes from session %s", len(strokes), file_path)
        return strokes

    def export_png(self, strokes: List[Stroke], output_path: Path, width: int = 1920, height: int = 1080) -> Path:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        renderer = OverlayRenderer(width, height)
        renderer.strokes = strokes
        renderer.render_all_strokes()

        if renderer.surface:
            renderer.surface.write_to_png(str(output_path))
            logger.info("Exported drawing to PNG: %s", output_path)
        return output_path

    def list_sessions(self) -> List[Dict[str, str]]:
        sessions = []
        for p in sorted(self.sessions_dir.glob("*.json"), key=lambda x: x.stat().st_mtime, reverse=True):
            sessions.append({
                "name": p.stem,
                "path": str(p),
                "modified": str(time.ctime(p.stat().st_mtime))
            })
        return sessions
