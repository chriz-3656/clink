"""Tests for drawing session management and PNG export."""
from pathlib import Path
from clink.protocol.messages import Point, Stroke
from clink.sessions.session_manager import SessionManager

def test_session_lifecycle(tmp_path: Path):
    mgr = SessionManager(sessions_dir=tmp_path / "sessions")
    strokes = [
        Stroke("s1", "pen", "#FF0000", 3.0, [Point(0.1, 0.1), Point(0.2, 0.2)], completed=True),
        Stroke("s2", "highlighter", "#00FF00", 8.0, [Point(0.3, 0.3), Point(0.4, 0.4)], completed=True)
    ]

    saved_file = mgr.save_session(strokes, session_name="my_drawing")
    assert saved_file.exists()

    loaded_strokes = mgr.load_session(saved_file)
    assert len(loaded_strokes) == 2
    assert loaded_strokes[0].stroke_id == "s1"
    assert loaded_strokes[1].tool == "highlighter"

    png_path = tmp_path / "export.png"
    mgr.export_png(loaded_strokes, png_path, width=400, height=300)
    assert png_path.exists()
    assert png_path.stat().st_size > 0
