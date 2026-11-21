"""Abstract base class for pointer and tablet input injection backends."""
from __future__ import annotations
from abc import ABC, abstractmethod
from typing import Tuple

class InputBackend(ABC):
    @abstractmethod
    def move_relative(self, dx: float, dy: float, pressure: float = 1.0) -> None:
        """Inject relative cursor motion with normalized deltas."""
        pass

    @abstractmethod
    def move_absolute(self, x: float, y: float, pressure: float = 1.0) -> None:
        """Inject absolute cursor motion using normalized coordinates [0, 1]."""
        pass

    @abstractmethod
    def button_down(self, button: int = 1, pressure: float = 1.0) -> None:
        """Inject mouse / stylus tip button down."""
        pass

    @abstractmethod
    def button_up(self, button: int = 1) -> None:
        """Inject mouse / stylus tip button up."""
        pass

    @abstractmethod
    def cleanup(self) -> None:
        """Release all pressed buttons and reset state."""
        pass

    @abstractmethod
    def get_screen_size(self) -> Tuple[int, int]:
        """Return screen width and height in pixels."""
        pass
