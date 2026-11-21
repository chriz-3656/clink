"""Cairo-based drawing renderer for the desktop annotation overlay."""
from __future__ import annotations
import math
import time
from typing import Dict, List, Optional, Tuple
import cairo

from clink.protocol.messages import Point, Stroke

def parse_color(hex_str: str) -> Tuple[float, float, float, float]:
    """Parse #RGB, #RRGGBB, or #RRGGBBAA into (r, g, b, a) in range [0, 1]."""
    s = hex_str.strip().lstrip("#")
    if len(s) == 3:
        r = int(s[0] * 2, 16) / 255.0
        g = int(s[1] * 2, 16) / 255.0
        b = int(s[2] * 2, 16) / 255.0
        return (r, g, b, 1.0)
    elif len(s) == 6:
        r = int(s[0:2], 16) / 255.0
        g = int(s[2:4], 16) / 255.0
        b = int(s[4:6], 16) / 255.0
        return (r, g, b, 1.0)
    elif len(s) == 8:
        r = int(s[0:2], 16) / 255.0
        g = int(s[2:4], 16) / 255.0
        b = int(s[4:6], 16) / 255.0
        a = int(s[6:8], 16) / 255.0
        return (r, g, b, a)
    return (1.0, 0.2, 0.4, 1.0)

class OverlayRenderer:
    def __init__(self, width: int = 1920, height: int = 1080):
        self.width = max(1, width)
        self.height = max(1, height)
        self.strokes: List[Stroke] = []
        self.undo_stack: List[List[Stroke]] = []
        self.redo_stack: List[List[Stroke]] = []
        self.active_strokes: Dict[str, Stroke] = {}
        
        # Laser pointer state
        self.laser_pos: Optional[Tuple[float, float]] = None
        self.laser_active: bool = False
        self.laser_color: str = "#FF0033"

        # Annotation timer / auto-fade duration in seconds (0 = persistent)
        self.fade_timeout: float = 0.0
        self.stroke_timestamps: Dict[str, float] = {}

        # Offscreen surface for persistent ink
        self.surface: Optional[cairo.ImageSurface] = None
        self._init_surface()

    def _init_surface(self) -> None:
        self.surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, self.width, self.height)
        ctx = cairo.Context(self.surface)
        ctx.set_operator(cairo.OPERATOR_CLEAR)
        ctx.paint()

    def resize(self, width: int, height: int) -> None:
        if width == self.width and height == self.height:
            return
        old_surface = self.surface
        self.width = max(1, width)
        self.height = max(1, height)
        self._init_surface()
        self.render_all_strokes()

    def start_stroke(self, stroke: Stroke) -> None:
        self.active_strokes[stroke.stroke_id] = stroke
        # Clear redo stack on new action
        self.redo_stack.clear()
        if stroke.points:
            pt = stroke.points[0]
            # Draw initial dot or small segment
            self._draw_point(stroke, pt)

    def append_points(self, stroke_id: str, new_points: List[Point]) -> None:
        if stroke_id not in self.active_strokes:
            return
        stroke = self.active_strokes[stroke_id]
        if not new_points:
            return
        
        # For shapes, don't draw incrementally to surface; only update points
        if stroke.tool in ("line", "arrow", "rectangle", "circle"):
            stroke.points.extend(new_points)
            return

        # For freehand / pen / highlighter / eraser:
        for pt in new_points:
            if stroke.points:
                prev_pt = stroke.points[-1]
                self._draw_segment(stroke, prev_pt, pt)
            else:
                self._draw_point(stroke, pt)
            stroke.points.append(pt)

    def end_stroke(self, stroke_id: str, final_point: Optional[Point] = None) -> Optional[Stroke]:
        if stroke_id not in self.active_strokes:
            return None
        stroke = self.active_strokes.pop(stroke_id)
        if final_point:
            if stroke.tool in ("line", "arrow", "rectangle", "circle"):
                stroke.points.append(final_point)
            else:
                if stroke.points:
                    self._draw_segment(stroke, stroke.points[-1], final_point)
                stroke.points.append(final_point)
        stroke.completed = True
        self.stroke_timestamps[stroke.stroke_id] = time.time()

        # Save history state for undo
        self.undo_stack.append([s for s in self.strokes])
        self.strokes.append(stroke)

        # If shape tool, render the shape onto the persistent surface
        if stroke.tool in ("line", "arrow", "rectangle", "circle"):
            self._render_shape(stroke)

        return stroke

    def purge_expired_strokes(self) -> bool:
        """Remove strokes that have exceeded fade_timeout. Returns True if any stroke was removed."""
        if self.fade_timeout <= 0:
            return False
        now = time.time()
        initial_count = len(self.strokes)
        valid_strokes = []
        for s in self.strokes:
            t = self.stroke_timestamps.get(s.stroke_id, now)
            if (now - t) <= self.fade_timeout:
                valid_strokes.append(s)
            else:
                self.stroke_timestamps.pop(s.stroke_id, None)

        if len(valid_strokes) != initial_count:
            self.strokes = valid_strokes
            self.render_all_strokes()
            return True
        return False

    def undo(self) -> bool:
        if not self.strokes and not self.undo_stack:
            return False
        # Push current state to redo
        self.redo_stack.append([s for s in self.strokes])
        if self.undo_stack:
            self.strokes = self.undo_stack.pop()
        else:
            self.strokes.pop()
        self.render_all_strokes()
        return True

    def redo(self) -> bool:
        if not self.redo_stack:
            return False
        self.undo_stack.append([s for s in self.strokes])
        self.strokes = self.redo_stack.pop()
        self.render_all_strokes()
        return True

    def clear(self) -> None:
        if not self.strokes and not self.active_strokes:
            return
        self.undo_stack.append([s for s in self.strokes])
        self.redo_stack.clear()
        self.strokes.clear()
        self.active_strokes.clear()
        self.stroke_timestamps.clear()
        self._init_surface()

    def set_laser(self, x: float, y: float, active: bool) -> None:
        self.laser_pos = (x, y)
        self.laser_active = active

    def render_all_strokes(self) -> None:
        self._init_surface()
        for stroke in self.strokes:
            if stroke.tool in ("line", "arrow", "rectangle", "circle"):
                self._render_shape(stroke)
            else:
                for i in range(len(stroke.points)):
                    if i == 0:
                        self._draw_point(stroke, stroke.points[0])
                    else:
                        self._draw_segment(stroke, stroke.points[i - 1], stroke.points[i])

    def _draw_point(self, stroke: Stroke, pt: Point) -> None:
        if not self.surface:
            return
        ctx = cairo.Context(self.surface)
        px = pt.x * self.width
        py = pt.y * self.height
        radius = max(1.0, (stroke.width * pt.pressure) / 2.0)

        if stroke.tool == "eraser":
            ctx.set_operator(cairo.OPERATOR_CLEAR)
            ctx.arc(px, py, radius * 2.5, 0, 2 * math.pi)
            ctx.fill()
        else:
            r, g, b, a = parse_color(stroke.color)
            if stroke.tool == "highlighter":
                ctx.set_operator(cairo.OPERATOR_OVER)
                ctx.set_source_rgba(r, g, b, 0.35)
                ctx.arc(px, py, radius * 2.0, 0, 2 * math.pi)
            elif stroke.tool == "pencil":
                ctx.set_operator(cairo.OPERATOR_OVER)
                ctx.set_source_rgba(r, g, b, 0.7)
                ctx.arc(px, py, radius * 0.8, 0, 2 * math.pi)
            else:
                ctx.set_operator(cairo.OPERATOR_OVER)
                ctx.set_source_rgba(r, g, b, a)
                ctx.arc(px, py, radius, 0, 2 * math.pi)
            ctx.fill()

    def _draw_segment(self, stroke: Stroke, p1: Point, p2: Point) -> None:
        if not self.surface:
            return
        ctx = cairo.Context(self.surface)
        x1, y1 = p1.x * self.width, p1.y * self.height
        x2, y2 = p2.x * self.width, p2.y * self.height
        avg_pressure = (p1.pressure + p2.pressure) / 2.0
        line_w = max(1.0, stroke.width * avg_pressure)

        ctx.set_line_cap(cairo.LINE_CAP_ROUND)
        ctx.set_line_join(cairo.LINE_JOIN_ROUND)

        if stroke.tool == "eraser":
            ctx.set_operator(cairo.OPERATOR_CLEAR)
            ctx.set_line_width(line_w * 2.5)
            ctx.move_to(x1, y1)
            ctx.line_to(x2, y2)
            ctx.stroke()
        else:
            r, g, b, a = parse_color(stroke.color)
            if stroke.tool == "highlighter":
                ctx.set_operator(cairo.OPERATOR_OVER)
                ctx.set_source_rgba(r, g, b, 0.35)
                ctx.set_line_width(line_w * 2.2)
            elif stroke.tool == "pencil":
                ctx.set_operator(cairo.OPERATOR_OVER)
                ctx.set_source_rgba(r, g, b, 0.7)
                ctx.set_line_width(line_w * 0.8)
            else:
                ctx.set_operator(cairo.OPERATOR_OVER)
                ctx.set_source_rgba(r, g, b, a)
                ctx.set_line_width(line_w)

            ctx.move_to(x1, y1)
            ctx.line_to(x2, y2)
            ctx.stroke()

    def _render_shape(self, stroke: Stroke) -> None:
        if not self.surface or len(stroke.points) < 2:
            return
        ctx = cairo.Context(self.surface)
        p1 = stroke.points[0]
        p2 = stroke.points[-1]
        x1, y1 = p1.x * self.width, p1.y * self.height
        x2, y2 = p2.x * self.width, p2.y * self.height

        r, g, b, a = parse_color(stroke.color)
        ctx.set_source_rgba(r, g, b, a)
        ctx.set_line_width(stroke.width)
        ctx.set_line_cap(cairo.LINE_CAP_ROUND)
        ctx.set_line_join(cairo.LINE_JOIN_ROUND)

        if stroke.tool == "line":
            ctx.move_to(x1, y1)
            ctx.line_to(x2, y2)
            ctx.stroke()
        elif stroke.tool == "arrow":
            # Draw shaft
            ctx.move_to(x1, y1)
            ctx.line_to(x2, y2)
            ctx.stroke()
            # Draw arrowhead
            angle = math.atan2(y2 - y1, x2 - x1)
            arrow_len = max(14.0, stroke.width * 3.5)
            arrow_angle = math.pi / 6.0  # 30 deg
            
            x_left = x2 - arrow_len * math.cos(angle - arrow_angle)
            y_left = y2 - arrow_len * math.sin(angle - arrow_angle)
            x_right = x2 - arrow_len * math.cos(angle + arrow_angle)
            y_right = y2 - arrow_len * math.sin(angle + arrow_angle)

            ctx.move_to(x2, y2)
            ctx.line_to(x_left, y_left)
            ctx.move_to(x2, y2)
            ctx.line_to(x_right, y_right)
            ctx.stroke()
        elif stroke.tool == "rectangle":
            rx = min(x1, x2)
            ry = min(y1, y2)
            rw = abs(x2 - x1)
            rh = abs(y2 - y1)
            ctx.rectangle(rx, ry, rw, rh)
            ctx.stroke()
        elif stroke.tool == "circle":
            cx = (x1 + x2) / 2.0
            cy = (y1 + y2) / 2.0
            rx = abs(x2 - x1) / 2.0
            ry = abs(y2 - y1) / 2.0
            if rx > 0 and ry > 0:
                ctx.save()
                ctx.translate(cx, cy)
                ctx.scale(rx, ry)
                ctx.arc(0, 0, 1.0, 0, 2 * math.pi)
                ctx.restore()
                ctx.stroke()

    def draw_to_context(self, ctx: cairo.Context) -> None:
        """Draw persistent surface and transient active elements (shapes in progress, laser) to window context."""
        # 1. Paint offscreen surface
        if self.surface:
            ctx.set_source_surface(self.surface, 0, 0)
            ctx.set_operator(cairo.OPERATOR_OVER)
            ctx.paint()

        # 2. Draw in-progress shapes
        for stroke in self.active_strokes.values():
            if stroke.tool in ("line", "arrow", "rectangle", "circle") and len(stroke.points) >= 2:
                p1 = stroke.points[0]
                p2 = stroke.points[-1]
                x1, y1 = p1.x * self.width, p1.y * self.height
                x2, y2 = p2.x * self.width, p2.y * self.height

                r, g, b, a = parse_color(stroke.color)
                ctx.set_source_rgba(r, g, b, a)
                ctx.set_line_width(stroke.width)
                ctx.set_line_cap(cairo.LINE_CAP_ROUND)
                ctx.set_line_join(cairo.LINE_JOIN_ROUND)

                if stroke.tool == "line":
                    ctx.move_to(x1, y1)
                    ctx.line_to(x2, y2)
                    ctx.stroke()
                elif stroke.tool == "arrow":
                    ctx.move_to(x1, y1)
                    ctx.line_to(x2, y2)
                    ctx.stroke()
                    angle = math.atan2(y2 - y1, x2 - x1)
                    arrow_len = max(14.0, stroke.width * 3.5)
                    arrow_angle = math.pi / 6.0
                    ctx.move_to(x2, y2)
                    ctx.line_to(x2 - arrow_len * math.cos(angle - arrow_angle), y2 - arrow_len * math.sin(angle - arrow_angle))
                    ctx.move_to(x2, y2)
                    ctx.line_to(x2 - arrow_len * math.cos(angle + arrow_angle), y2 - arrow_len * math.sin(angle + arrow_angle))
                    ctx.stroke()
                elif stroke.tool == "rectangle":
                    ctx.rectangle(min(x1, x2), min(y1, y2), abs(x2 - x1), abs(y2 - y1))
                    ctx.stroke()
                elif stroke.tool == "circle":
                    cx, cy = (x1 + x2) / 2.0, (y1 + y2) / 2.0
                    rx, ry = abs(x2 - x1) / 2.0, abs(y2 - y1) / 2.0
                    if rx > 0 and ry > 0:
                        ctx.save()
                        ctx.translate(cx, cy)
                        ctx.scale(rx, ry)
                        ctx.arc(0, 0, 1.0, 0, 2 * math.pi)
                        ctx.restore()
                        ctx.stroke()

        # 3. Draw Laser Pointer if active
        if self.laser_active and self.laser_pos:
            lx = self.laser_pos[0] * self.width
            ly = self.laser_pos[1] * self.height

            # Outer glow
            pat = cairo.RadialGradient(lx, ly, 2.0, lx, ly, 18.0)
            pat.add_color_stop_rgba(0.0, 1.0, 0.1, 0.1, 0.9)
            pat.add_color_stop_rgba(0.4, 1.0, 0.2, 0.2, 0.5)
            pat.add_color_stop_rgba(1.0, 1.0, 0.0, 0.0, 0.0)
            ctx.set_source(pat)
            ctx.arc(lx, ly, 18.0, 0, 2 * math.pi)
            ctx.fill()

            # Solid core
            ctx.set_source_rgba(1.0, 1.0, 1.0, 1.0)
            ctx.arc(lx, ly, 3.5, 0, 2 * math.pi)
            ctx.fill()
