"""Tests for Linux transparent overlay window and click-through."""
import pytest
from clink.overlay.renderer import OverlayRenderer
from clink.overlay.window import OverlayWindow, set_x11_click_through

def test_overlay_window_instantiation():
    renderer = OverlayRenderer(1920, 1080)
    win = OverlayWindow(renderer=renderer, click_through=True)
    assert win.renderer == renderer
    assert win.click_through_enabled is True

def test_set_x11_click_through_without_display():
    # Calling with invalid / non-existent display or window handle handles exceptions gracefully
    res = set_x11_click_through(99999999, enable=True)
    assert isinstance(res, bool)
