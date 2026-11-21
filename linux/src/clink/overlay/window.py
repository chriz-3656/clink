"""GTK3-based transparent overlay window with click-through support."""
from __future__ import annotations
import logging
from typing import Optional, Callable
import cairo

try:
    import gi
    gi.require_version("Gtk", "3.0")
    gi.require_version("Gdk", "3.0")
    from gi.repository import Gtk, Gdk, GLib
    HAS_GTK = True
except Exception as e:
    HAS_GTK = False

from clink.overlay.renderer import OverlayRenderer

logger = logging.getLogger(__name__)

if HAS_GTK:
    class OverlayWindow(Gtk.Window):
        def __init__(self, renderer: Optional[OverlayRenderer] = None, click_through: bool = True):
            super().__init__(type=Gtk.WindowType.TOPLEVEL)
            self.click_through_enabled = click_through
            
            # Screen geometry
            screen = Gdk.Screen.get_default()
            if screen:
                self.screen_w = screen.get_width()
                self.screen_h = screen.get_height()
                # RGBA visual for transparency
                visual = screen.get_rgba_visual()
                if visual:
                    self.set_visual(visual)
            else:
                self.screen_w, self.screen_h = 1920, 1080

            self.renderer = renderer or OverlayRenderer(self.screen_w, self.screen_h)

            # Configure transparent window properties
            self.set_title("Clink Desktop Overlay")
            self.set_decorated(False)
            self.set_keep_above(True)
            self.set_skip_taskbar_hint(True)
            self.set_skip_pager_hint(True)
            self.set_app_paintable(True)
            self.set_default_size(self.screen_w, self.screen_h)
            self.move(0, 0)

            # Drawing area widget
            self.drawing_area = Gtk.DrawingArea()
            self.drawing_area.connect("draw", self.on_draw)
            self.add(self.drawing_area)

            # Signals
            self.connect("realize", self.on_realize)
            self.connect("size-allocate", self.on_size_allocate)

        def on_realize(self, widget: Gtk.Widget) -> None:
            self.apply_click_through(self.click_through_enabled)

        def on_size_allocate(self, widget: Gtk.Widget, allocation: Gdk.Rectangle) -> None:
            if allocation.width != self.renderer.width or allocation.height != self.renderer.height:
                self.renderer.resize(allocation.width, allocation.height)

        def apply_click_through(self, enable: bool) -> None:
            self.click_through_enabled = enable
            gdk_win = self.get_window()
            if not gdk_win:
                return
            if enable:
                empty_region = cairo.Region()
                gdk_win.input_shape_combine_region(empty_region, 0, 0)
            else:
                rect = cairo.RectangleInt(0, 0, self.get_allocated_width(), self.get_allocated_height())
                full_region = cairo.Region(rect)
                gdk_win.input_shape_combine_region(full_region, 0, 0)

        def set_click_through(self, enable: bool) -> None:
            GLib.idle_add(self.apply_click_through, enable)

        def on_draw(self, widget: Gtk.Widget, ctx: cairo.Context) -> bool:
            ctx.set_operator(cairo.OPERATOR_CLEAR)
            ctx.paint()

            ctx.set_operator(cairo.OPERATOR_OVER)
            self.renderer.draw_to_context(ctx)
            return False

        def request_redraw(self) -> None:
            """Thread-safe request to redraw the window."""
            GLib.idle_add(self.drawing_area.queue_draw)
else:
    class OverlayWindow:  # type: ignore
        """Headless fallback when GTK3 introspection is unavailable."""
        def __init__(self, renderer: Optional[OverlayRenderer] = None, click_through: bool = True):
            self.renderer = renderer or OverlayRenderer(1920, 1080)
            self.click_through_enabled = click_through

        def set_click_through(self, enable: bool) -> None:
            self.click_through_enabled = enable

        def request_redraw(self) -> None:
            pass

        def show_all(self) -> None:
            pass

        def hide(self) -> None:
            pass
