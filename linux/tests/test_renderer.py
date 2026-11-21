"""Tests for Cairo drawing renderer."""
import math
from clink.overlay.renderer import OverlayRenderer, parse_color
from clink.protocol.messages import Point, Stroke

def test_parse_color():
    assert parse_color("#FFFFFF") == (1.0, 1.0, 1.0, 1.0)
    assert parse_color("#000000") == (0.0, 0.0, 0.0, 1.0)
    r, g, b, a = parse_color("#FF000080")
    assert round(r, 2) == 1.0
    assert round(g, 2) == 0.0
    assert round(b, 2) == 0.0
    assert round(a, 2) == 0.5

def test_stroke_lifecycle():
    renderer = OverlayRenderer(800, 600)
    s = Stroke("s1", "pen", "#FF0000", 3.0, [Point(0.1, 0.1, 1.0)])
    
    renderer.start_stroke(s)
    assert "s1" in renderer.active_strokes
    
    renderer.append_points("s1", [Point(0.2, 0.2, 1.0), Point(0.3, 0.3, 1.0)])
    assert len(renderer.active_strokes["s1"].points) == 3
    
    completed = renderer.end_stroke("s1", Point(0.4, 0.4, 1.0))
    assert completed is not None
    assert completed.completed is True
    assert "s1" not in renderer.active_strokes
    assert len(renderer.strokes) == 1

def test_undo_redo():
    renderer = OverlayRenderer(800, 600)
    s1 = Stroke("s1", "pen", "#FF0000", 3.0, [Point(0.1, 0.1), Point(0.2, 0.2)])
    renderer.start_stroke(s1)
    renderer.end_stroke("s1")
    assert len(renderer.strokes) == 1

    s2 = Stroke("s2", "highlighter", "#00FF00", 10.0, [Point(0.3, 0.3), Point(0.4, 0.4)])
    renderer.start_stroke(s2)
    renderer.end_stroke("s2")
    assert len(renderer.strokes) == 2

    # Undo
    assert renderer.undo() is True
    assert len(renderer.strokes) == 1
    assert renderer.strokes[0].stroke_id == "s1"

    # Redo
    assert renderer.redo() is True
    assert len(renderer.strokes) == 2
    assert renderer.strokes[1].stroke_id == "s2"

def test_clear():
    renderer = OverlayRenderer(800, 600)
    s1 = Stroke("s1", "pen", "#FF0000", 3.0, [Point(0.1, 0.1)])
    renderer.start_stroke(s1)
    renderer.end_stroke("s1")
    assert len(renderer.strokes) == 1

    renderer.clear()
    assert len(renderer.strokes) == 0

    # Clear can be undone
    assert renderer.undo() is True
    assert len(renderer.strokes) == 1

def test_shapes():
    renderer = OverlayRenderer(800, 600)
    for tool in ("line", "arrow", "rectangle", "circle"):
        st = Stroke(f"shape_{tool}", tool, "#0000FF", 2.0, [Point(0.1, 0.1)])
        renderer.start_stroke(st)
        renderer.end_stroke(f"shape_{tool}", Point(0.5, 0.5))
    assert len(renderer.strokes) == 4

def test_laser():
    renderer = OverlayRenderer(800, 600)
    renderer.set_laser(0.5, 0.5, True)
    assert renderer.laser_active is True
    assert renderer.laser_pos == (0.5, 0.5)
    renderer.set_laser(0.5, 0.5, False)
    assert renderer.laser_active is False
