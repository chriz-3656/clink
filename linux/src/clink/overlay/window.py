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
    _libx11.XCloseDisplay.argtypes = [ctypes.c_void_p]
    _libx11.XCloseDisplay.restype = ctypes.c_int
    _libx11.XFlush.argtypes = [ctypes.c_void_p]
    _libx11.XFlush.restype = ctypes.c_int
    _libx11.XRaiseWindow.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
    _libx11.XRaiseWindow.restype = ctypes.c_int
except Exception as e:
    logger.debug("libX11 not loaded: %s", e)

try:
    xfixes_name = ctypes.util.find_library("Xfixes") or "libXfixes.so.3"
    _libxfixes = ctypes.CDLL(xfixes_name)
    _libxfixes.XFixesCreateRegion.argtypes = [ctypes.c_void_p, ctypes.c_void_p, ctypes.c_int]
    _libxfixes.XFixesCreateRegion.restype = ctypes.c_ulong
    _libxfixes.XFixesDestroyRegion.argtypes = [ctypes.c_void_p, ctypes.c_ulong]
    _libxfixes.XFixesDestroyRegion.restype = None
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
    dpy = None
    try:
        dpy = _libx11.XOpenDisplay(None)
        if not dpy:
            return False
        if enable:
            if _libxfixes and hasattr(_libxfixes, "XFixesCreateRegion") and hasattr(_libxfixes, "XFixesSetWindowShapeRegion"):
                empty_reg = _libxfixes.XFixesCreateRegion(dpy, None, 0)
                # ShapeInput = 2
                _libxfixes.XFixesSetWindowShapeRegion(dpy, xid, 2, 0, 0, empty_reg)
                if hasattr(_libxfixes, "XFixesDestroyRegion"):
                    _libxfixes.XFixesDestroyRegion(dpy, empty_reg)
            if _libxext and hasattr(_libxext, "XShapeCombineRectangles"):
                # ShapeInput = 2, ShapeSet = 0, n_rects = 0 (empty input shape)
                _libxext.XShapeCombineRectangles(dpy, xid, 2, 0, 0, None, 0, 0, 0)
        else:
            if _libxext and hasattr(_libxext, "XShapeCombineRectangles"):
                rect = _XRectangle(0, 0, width, height)
                _libxext.XShapeCombineRectangles(dpy, xid, 2, 0, 0, ctypes.byref(rect), 1, 0, 0)
        _libx11.XFlush(dpy)
        _libx11.XCloseDisplay(dpy)
        return True
    except Exception as e:
        logger.warning("Error applying X11 click-through: %s", e)
        if dpy:
            try:
                _libx11.XCloseDisplay(dpy)
            except Exception:
                pass
        return False

try:
    import gi
    gi.require_version("Gtk", "3.0")
    gi.require_version("Gdk", "3.0")
    from gi.repository import Gtk, Gdk, GLib
    HAS_GTK = bool(Gtk.init_check()[0])
except Exception as e:
    HAS_GTK = False

from clink.overlay.renderer import OverlayRenderer

if HAS_GTK:
    class OverlayWindow(Gtk.Window):
        def __init__(self, renderer: Optional[OverlayRenderer] = None, click_through: bool = True):
            # Use POPUP to make it an override-redirect window that stays on top and is never
            # minimized or hidden by the window manager when the main control center is minimized.
            super().__init__(type=Gtk.WindowType.POPUP)
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
            self.set_app_paintable(True)
            self.set_default_size(self.screen_w, self.screen_h)
            self.move(0, 0)

            # Connect draw directly to self - no child sub-windows that intercept pointer clicks
            self.connect("draw", self.on_draw)

            # Signals
            self.connect("realize", self.on_realize)
            self.connect("map", self.on_map)
            self.connect("size-allocate", self.on_size_allocate)

            # Periodic check for expiring annotation strokes (every 250ms)
            GLib.timeout_add(250, self._check_stroke_expiry)

        def _check_stroke_expiry(self) -> bool:
            if self.renderer.purge_expired_strokes():
                self.request_redraw()
            return True

        def on_realize(self, widget: Gtk.Widget) -> None:
            self.apply_click_through(self.click_through_enabled)

        def on_map(self, widget: Gtk.Widget) -> None:
            self.apply_click_through(self.click_through_enabled)

        def on_size_allocate(self, widget: Gtk.Widget, allocation: Gdk.Rectangle) -> None:
            if allocation.width != self.renderer.width or allocation.height != self.renderer.height:
                self.renderer.resize(allocation.width, allocation.height)
            self.apply_click_through(self.click_through_enabled)

        def apply_click_through(self, enable: bool) -> None:
            self.click_through_enabled = enable
            gdk_win = self.get_window()
            if not gdk_win:
                return

            # GDK native pass-through (GTK >= 3.18)
            if hasattr(gdk_win, "set_pass_through"):
                gdk_win.set_pass_through(enable)

            # Primary: Native X11 XFixes / XShape empty input region combining
            try:
                xid = gdk_win.get_xid()
                if xid:
                    set_x11_click_through(xid, enable, self.renderer.width, self.renderer.height)
            except Exception as e:
                logger.debug("Native X11 shape helper exception: %s", e)

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
            GLib.idle_add(self.queue_draw)
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
