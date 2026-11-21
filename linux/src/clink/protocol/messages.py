"""Protocol message definitions and validation."""
from __future__ import annotations
import json
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Tuple

PROTOCOL_VERSION = "1.0"

class ProtocolError(Exception):
    """Protocol validation error."""
    pass

@dataclass
class Point:
    x: float
    y: float
    pressure: float = 1.0
    timestamp: int = 0

    def to_dict(self) -> Dict[str, Any]:
        return {"x": self.x, "y": self.y, "pressure": self.pressure, "timestamp": self.timestamp}

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> Point:
        return cls(
            x=float(data.get("x", 0.0)),
            y=float(data.get("y", 0.0)),
            pressure=float(data.get("pressure", 1.0)),
            timestamp=int(data.get("timestamp", 0))
        )

@dataclass
class Stroke:
    stroke_id: str
    tool: str
    color: str
    width: float
    points: List[Point] = field(default_factory=list)
    completed: bool = False

    def to_dict(self) -> Dict[str, Any]:
        return {
            "stroke_id": self.stroke_id,
            "tool": self.tool,
            "color": self.color,
            "width": self.width,
            "points": [p.to_dict() for p in self.points],
            "completed": self.completed
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> Stroke:
        points = [Point.from_dict(p) for p in data.get("points", [])]
        return cls(
            stroke_id=str(data.get("stroke_id", "")),
            tool=str(data.get("tool", "pen")),
            color=str(data.get("color", "#FF3366")),
            width=float(data.get("width", 3.0)),
            points=points,
            completed=bool(data.get("completed", False))
        )

def serialize_message(msg: Dict[str, Any]) -> str:
    return json.dumps(msg, separators=(",", ":"))

def parse_message(raw_data: str) -> Dict[str, Any]:
    try:
        msg = json.loads(raw_data)
        if not isinstance(msg, dict):
            raise ProtocolError("Message payload must be a JSON object")
        if "type" not in msg:
            raise ProtocolError("Missing 'type' field in message")
        return msg
    except json.JSONDecodeError as e:
        raise ProtocolError(f"Malformed JSON: {e}") from e
