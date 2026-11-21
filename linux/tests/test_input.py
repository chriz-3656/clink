"""Tests for input backend and safety cleanup."""
from clink.input.backend import InputBackend
from clink.input.x11_injector import X11InputInjector

class MockInputBackend(InputBackend):
    def __init__(self):
        self.cursor = (0.5, 0.5)
        self.pressed = set()

    def move_relative(self, dx: float, dy: float, pressure: float = 1.0):
        self.cursor = (self.cursor[0] + dx, self.cursor[1] + dy)

    def move_absolute(self, x: float, y: float, pressure: float = 1.0):
        self.cursor = (x, y)

    def button_down(self, button: int = 1, pressure: float = 1.0):
        self.pressed.add(button)

    def button_up(self, button: int = 1):
        self.pressed.discard(button)

    def cleanup(self):
        self.pressed.clear()

    def get_screen_size(self):
        return (1920, 1080)

def test_mock_backend():
    backend = MockInputBackend()
    backend.move_absolute(0.2, 0.4)
    assert backend.cursor == (0.2, 0.4)
    backend.move_relative(0.1, -0.1)
    assert round(backend.cursor[0], 2) == 0.3
    assert round(backend.cursor[1], 2) == 0.3

    backend.button_down(1)
    assert 1 in backend.pressed
    backend.cleanup()
    assert len(backend.pressed) == 0

def test_x11_injector_headless_fallback():
    injector = X11InputInjector()
    # In headless environment without X, methods should not crash
    injector.move_relative(0.01, 0.02)
    injector.move_absolute(0.5, 0.5)
    injector.button_down(1)
    injector.button_up(1)
    injector.cleanup()
    assert isinstance(injector.get_screen_size(), tuple)
