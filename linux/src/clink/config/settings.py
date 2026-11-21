"""Configuration management for Clink."""
from __future__ import annotations
import json
import os
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any, Dict

DEFAULT_CONFIG_DIR = Path.home() / ".config" / "clink"
DEFAULT_CONFIG_PATH = DEFAULT_CONFIG_DIR / "config.json"

@dataclass
class ClinkConfig:
    host: str = "0.0.0.0"
    port: int = 8765
    require_auth: bool = True
    overlay_enabled: bool = True
    click_through: bool = True
    default_tool: str = "pen"
    default_color: str = "#FF3366"
    default_stroke_width: float = 3.5
    smoothing_enabled: bool = True
    pressure_sensitivity: float = 1.0
    relative_sensitivity: float = 1.5
    relative_acceleration: float = 1.2
    laser_fade_ms: int = 1200
    log_level: str = "INFO"

    @classmethod
    def load(cls, path: Path = DEFAULT_CONFIG_PATH) -> ClinkConfig:
        if path.exists():
            try:
                with open(path, "r", encoding="utf-8") as f:
                    data = json.load(f)
                    return cls(**{k: v for k, v in data.items() if k in cls.__dataclass_fields__})
            except Exception:
                return cls()
        return cls()

    def save(self, path: Path = DEFAULT_CONFIG_PATH) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            json.dump(asdict(self), f, indent=2)
