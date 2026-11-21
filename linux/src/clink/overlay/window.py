"""GTK3-based transparent overlay window with click-through support."""
from __future__ import annotations
import logging
from typing import Optional, Callable
import cairo
import ctypes
import ctypes.util

logger = logging.getLogger(__name__)

# Native X11 library bindings for reliable click-through input shapes
_libx11 = None
_libxfixes = None
_libxext = None

try:
    x11_name = ctypes.util.find_library("X11") or "libX11.so.6"
    _libx11 = ctypes.CDLL(x11_name)
    _libx11.XOpenDisplay.argtypes = [ctypes.c_char_p]
    _libx11.XOpenDisplay.restype = ctypes.c_void_p
    _libx11.XFlush.argtypes = [ctypes.c_void_p]
except Exception as e:
    logger.debug("libX11 not loaded: %s", e)

try:
    xfixes_name = ctypes.util.find_library("Xfixes") or "libXfixes.so.3"
    _libxfixes = ctypes.CDLL(xfixes_name)
    _libxfixes.XFixesSetWindowShapeRegion.argtypes = [
        ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_ulong
    ]
    _libxfixes.XFixesSetWindowShapeRegion.restype = None
except Exception as e:
    logger.debug("libXfixes not loaded: %s", e)

try:
    xext_name = ctypes.util.find_library("Xext") or "libXext.so.6"
    _libxext = ctypes.CDLL(xext_name)
    _libxext.XShapeCombineRectangles.argtypes = [
        ctypes.c_void_p, ctypes.c_ulong, ctypes.c_int, ctypes.c_int, ctypes.c_int, ctypes.c_void_p, ctypes.c_int, ctypes.c_int, ctypes.c_int
    ]
    _libxext.XShapeCombineRectangles.restype = ctypes.c_int
except Exception as e:
    logger.debug("libXext not loaded: %s", e)

class _XRectangle(ctypes.Structure):
    _fields_ = [("x", ctypes.c_short), ("y", ctypes.c_short),
                ("width", ctypes.c_ushort), ("height", ctypes.c_ushort)]

def set_x11_click_through(xid: int, enable: bool, width: int = 1920, height: int = 1080) -> bool:
    """Set X11 window input shape to empty (click-through) or full rectangle."""
    if not _libx11:
        return False
    try:
        dpy = _libx11.XOpenDisplay(None)
        if not dpy:
            return False
        if enable:
            if _libxfixes and hasattr(_libxfixes, "XFixesSetWindowShapeRegion"):
                # ShapeInput = 2, region 0 = empty (clicks pass through completely)
                _libxfixes.XFixesSetWindowShapeRegion(dpy, xid, 2, 0, 0, 0)
            elif _libxext and hasattr(_libxext, "XShapeCombineRectangles"):
                _libxext.XShapeCombineRectangles(dpy, xid, 2, 0, 0, None, 0, 0, 0)
        else:
            if _libxext and hasattr(_libxext, "XShapeCombineRectangles"):
                rect = _XRectangle(0, 0, width, height)
                _libxext.XShapeCombineRectangles(dpy, xid, 2, 0, 0, ctypes.byref(rect), 1, 0, 0)
        _libx11.XFlush(dpy)
        return True
    except Exception as e:
        logger.warning("Error applying X11 click-through: %s", e)
        return False

try:
    import gi
    gi.require_version("Gtk", "3.0")
    gi.require_version("Gdk", "3.0")
    from gi.repository import Gtk, Gdk, GLib
    HAS_GTK = True
except Exception as e:
    HAS_GTK = False

from clink.overlay.renderer import OverlayRenderer

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
                visual = screen.get_rgba_visual()
                if visual:
                    self.set_visual(visual)
            else:
                self.screen_w, self.screen_h = 1920, 1080

            self.renderer = renderer or OverlayRenderer(self.screen_w, self.screen_h)

            # Configure non-intrusive transparent window properties
            self.set_title("Clink Desktop Overlay")
            self.set_decorated(False)
            self.set_keep_above(True)
            self.set_accept_focus(False)
            self.set_focus_on_map(False)
            self.set_skip_taskbar_hint(True)
            self.set_skip_pager_hint(True)
            self.set_type_hint(Gdk.WindowTypeHint.DOCK)
            self.set_app_paintable(True)
            self.set_default_size(self.screen_w, self.screen_h)
            self.move(0, 0)

            # Drawing area widget
            self.drawing_area = Gtk.DrawingArea()
            self.drawing_area.connect("draw", self.on_draw)
            self.add(self.drawing_area)

            # Signals
            self.connect("realize", self.on_realize)
            self.connect("map", self.on_map)
            self.connect("size-allocate", self.on_size_allocate)

        def on_realize(self, widget: Gtk.Widget) -> None:
            self.apply_click_through(self.click_through_enabled)

        def on_map(self, widget: Gtk.Widget) -> None:
            self.apply_click_through(self.click_through_enabled)

        def on_size_allocate(self, widget: Gtk.Widget, allocation: Gdk.Rectangle) -> None:
            if allocation.width != self.renderer.width or allocation.height != self.renderer.height:
                self.renderer.resize(allocation.width, allocation.height)

        def apply_click_through(self, enable: bool) -> None:
            self.click_through_enabled = enable
            gdk_win = self.get_window()
            if not gdk_win:
                return

            # Primary: Native X11 XFixes / XShape input region combining
            try:
                xid = gdk_win.get_xid()
                if xid and set_x11_click_through(xid, enable, self.renderer.width, self.renderer.height):
                    return
            except Exception as e:
                logger.debug("Native X11 shape helper exception: %s", e)

            # Secondary fallback: GDK input_shape_combine_region
            try:
                if enable:
                    empty_region = cairo.Region()
                    gdk_win.input_shape_combine_region(empty_region, 0, 0)
                else:
                    rect = cairo.RectangleInt(0, 0, self.get_allocated_width(), self.get_allocated_height())
                    full_region = cairo.Region(rect)
                    gdk_win.input_shape_combine_region(full_region, 0, 0)
            except Exception as e:
                logger.debug("GDK input_shape_combine_region fallback: %s", e)

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

