"""Tests for protocol message parsing and serialization."""
import pytest
from clink.protocol.messages import (
    Point, Stroke, serialize_message, parse_message, ProtocolError, PROTOCOL_VERSION
)

def test_point_serialization():
    p = Point(x=0.25, y=0.5, pressure=0.8, timestamp=1000)
    d = p.to_dict()
    assert d == {"x": 0.25, "y": 0.5, "pressure": 0.8, "timestamp": 1000}
    p2 = Point.from_dict(d)
    assert p2.x == 0.25
    assert p2.y == 0.5
    assert p2.pressure == 0.8
    assert p2.timestamp == 1000

def test_stroke_serialization():
    s = Stroke(
        stroke_id="test_1",
        tool="pen",
        color="#00FF00",
        width=4.5,
        points=[Point(0.1, 0.1), Point(0.2, 0.2)],
        completed=True
    )
    d = s.to_dict()
    assert d["stroke_id"] == "test_1"
    assert d["tool"] == "pen"
    assert d["completed"] is True
    assert len(d["points"]) == 2

    s2 = Stroke.from_dict(d)
    assert s2.stroke_id == "test_1"
    assert len(s2.points) == 2
    assert s2.completed is True

def test_parse_message_success():
    raw = '{"type":"stroke_start","stroke_id":"s1","tool":"pen","x":0.5,"y":0.5}'
    msg = parse_message(raw)
    assert msg["type"] == "stroke_start"
    assert msg["stroke_id"] == "s1"

def test_parse_message_invalid_json():
    with pytest.raises(ProtocolError):
        parse_message("not a json string")

def test_parse_message_missing_type():
    with pytest.raises(ProtocolError):
        parse_message('{"id": 123}')

def test_serialize_message():
    data = {"type": "hello", "version": PROTOCOL_VERSION}
    res = serialize_message(data)
    assert '"type":"hello"' in res
