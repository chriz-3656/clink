"""X11 pointer and stylus input injection using XTest extension."""
from __future__ import annotations
import logging
import math
from typing import Optional, Set, Tuple

from clink.input.backend import InputBackend

logger = logging.getLogger(__name__)

class X11InputInjector(InputBackend):
    def __init__(self, sensitivity: float = 1.5, acceleration: float = 1.2):
        self.sensitivity = max(0.1, sensitivity)
        self.acceleration = max(1.0, acceleration)
        self.pressed_buttons: Set[int] = set()
        self.display = None
        self.root = None
        self.has_xtest = False
        self.width = 1920
        self.height = 1080
        self._init_x11()

    def _init_x11(self) -> None:
        try:
            from Xlib import X, display
            from Xlib.ext import xtest
            self.display = display.Display()
            self.root = self.display.screen().root
            self.has_xtest = self.display.has_extension("XTEST")
            self.width = self.display.screen().width_in_pixels
            self.height = self.display.screen().height_in_pixels
            logger.info("X11 input injector initialized (%dx%d, XTest=%s)", self.width, self.height, self.has_xtest)
        except Exception as e:
            logger.warning("Could not initialize X11 display: %s. Using mock fallback.", e)
            self.display = None

    def get_screen_size(self) -> Tuple[int, int]:
        return (self.width, self.height)

    def move_relative(self, dx: float, dy: float, pressure: float = 1.0) -> None:
        if not self.display or not self.has_xtest:
            return
        from Xlib import X
        from Xlib.ext import xtest

        try:
            geom = self.root.query_pointer()
            cur_x, cur_y = geom.root_x, geom.root_y

            # Apply acceleration curve and sensitivity
            dist = math.hypot(dx, dy)
            factor = self.sensitivity * (1.0 + (dist * self.acceleration * 10.0))
            delta_x = int(dx * self.width * factor)
            delta_y = int(dy * self.height * factor)

            target_x = max(0, min(self.width - 1, cur_x + delta_x))
            target_y = max(0, min(self.height - 1, cur_y + delta_y))

            xtest.fake_input(self.display, X.MotionNotify, x=target_x, y=target_y)
            self.display.sync()
        except Exception as e:
            logger.error("Error moving relative cursor: %s", e)

    def move_absolute(self, x: float, y: float, pressure: float = 1.0) -> None:
        if not self.display or not self.has_xtest:
            return
        from Xlib import X
        from Xlib.ext import xtest

        try:
            target_x = max(0, min(self.width - 1, int(x * self.width)))
            target_y = max(0, min(self.height - 1, int(y * self.height)))

            xtest.fake_input(self.display, X.MotionNotify, x=target_x, y=target_y)
            self.display.sync()
        except Exception as e:
            logger.error("Error moving absolute cursor: %s", e)

    def button_down(self, button: int = 1, pressure: float = 1.0) -> None:
        if not self.display or not self.has_xtest:
            return
        from Xlib import X
        from Xlib.ext import xtest

        try:
            xtest.fake_input(self.display, X.ButtonPress, detail=button)
            self.pressed_buttons.add(button)
            self.display.sync()
        except Exception as e:
            logger.error("Error pressing button %d: %s", button, e)

    def button_up(self, button: int = 1) -> None:
        if not self.display or not self.has_xtest:
            return
        from Xlib import X
        from Xlib.ext import xtest

        try:
            xtest.fake_input(self.display, X.ButtonRelease, detail=button)
            self.pressed_buttons.discard(button)
            self.display.sync()
        except Exception as e:
            logger.error("Error releasing button %d: %s", button, e)

    def cleanup(self) -> None:
        """Release all held buttons to prevent sticky button states on disconnect."""
        if not self.display or not self.has_xtest or not self.pressed_buttons:
            return
        from Xlib import X
        from Xlib.ext import xtest

        try:
            for button in list(self.pressed_buttons):
                xtest.fake_input(self.display, X.ButtonRelease, detail=button)
            self.pressed_buttons.clear()
            self.display.sync()
            logger.info("X11 input injector cleaned up all active buttons.")
        except Exception as e:
            logger.error("Error during cleanup: %s", e)
