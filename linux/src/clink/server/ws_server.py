"""WebSocket server for handling real-time Android communication."""
from __future__ import annotations
import asyncio
import json
import logging
import traceback
from typing import Any, Dict, Optional, Set
import websockets

from clink.config.settings import ClinkConfig
from clink.devices.pairing import PairingManager
from clink.input.backend import InputBackend
from clink.overlay.renderer import OverlayRenderer
from clink.overlay.window import OverlayWindow
from clink.protocol.messages import parse_message, serialize_message, Point, Stroke, PROTOCOL_VERSION

logger = logging.getLogger(__name__)

class ClinkServer:
    def __init__(
        self,
        config: ClinkConfig,
        renderer: OverlayRenderer,
        overlay_window: Optional[OverlayWindow],
        input_backend: InputBackend,
        pairing_manager: PairingManager
    ):
        self.config = config
        self.renderer = renderer
        self.overlay_window = overlay_window
        self.input_backend = input_backend
        self.pairing_manager = pairing_manager
        
        self.active_mode = "absolute_annotation"
        self.connected_clients: Set[websockets.WebSocketServerProtocol] = set()
        self.server: Optional[websockets.WebSocketServer] = None
        self._stop_event: asyncio.Event = asyncio.Event()

    async def start(self) -> None:
        logger.info("Starting Clink WebSocket server on %s:%d", self.config.host, self.config.port)
        self.server = await websockets.serve(
            self.handle_connection,
            self.config.host,
            self.config.port,
            ping_interval=15,
            ping_timeout=20,
            max_size=2 * 1024 * 1024  # 2MB max frame size
        )
        logger.info("Clink server listening on port %d", self.config.port)

    async def stop(self) -> None:
        logger.info("Stopping Clink WebSocket server...")
        self._stop_event.set()
        if self.server:
            self.server.close()
            await self.server.wait_closed()
        self.input_backend.cleanup()

    async def handle_connection(self, websocket: websockets.WebSocketServerProtocol) -> None:
        client_address = websocket.remote_address
        logger.info("New client connected from %s", client_address)
        self.connected_clients.add(websocket)
        authenticated = not self.config.require_auth

        try:
            async for raw_message in websocket:
                try:
                    msg = parse_message(raw_message)
                    response = await self.process_message(msg, websocket, authenticated)
                    if response:
                        await websocket.send(serialize_message(response))
                        if response.get("type") == "auth_ack" and response.get("success"):
                            authenticated = True
                        elif response.get("type") == "pair_ack" and response.get("success"):
                            authenticated = True
                        elif response.get("type") == "hello_ack" and response.get("authenticated"):
                            authenticated = True
                except Exception as e:
                    logger.error("Error processing message: %s\n%s", e, traceback.format_exc())
                    await websocket.send(serialize_message({
                        "type": "error",
                        "message": str(e)
                    }))
        except websockets.ConnectionClosed:
            logger.info("Client disconnected from %s", client_address)
        finally:
            self.connected_clients.discard(websocket)
            # Safe cleanup of any pressed buttons upon disconnect
            self.input_backend.cleanup()

    async def process_message(
        self, msg: Dict[str, Any], websocket: websockets.WebSocketServerProtocol, authenticated: bool
    ) -> Optional[Dict[str, Any]]:
        msg_type = msg.get("type")

        # 1. Handshake
        if msg_type == "hello":
            screen_w, screen_h = self.input_backend.get_screen_size()
            token = msg.get("token")
            is_valid = self.pairing_manager.authenticate_token(token) if token else False
            return {
                "type": "hello_ack",
                "version": PROTOCOL_VERSION,
                "server_name": "clink-linux",
                "authenticated": is_valid or not self.config.require_auth,
                "pairing_required": self.config.require_auth and not is_valid,
                "display": {
                    "width": screen_w,
                    "height": screen_h,
                    "aspect_ratio": round(screen_w / max(1, screen_h), 4)
                }
            }

        # 2. Pairing Request
        if msg_type == "pair_request":
            dev_id = msg.get("device_id", "unknown")
            dev_name = msg.get("device_name", "Android Device")
            pin = msg.get("pin", "")
            token = self.pairing_manager.pair_device(dev_id, dev_name, pin)
            if token:
                return {"type": "pair_ack", "success": True, "token": token}
            return {"type": "pair_ack", "success": False, "message": "Invalid PIN"}

        # 3. Auth Token
        if msg_type == "auth":
            token = msg.get("token", "")
            if not self.config.require_auth or self.pairing_manager.authenticate_token(token):
                return {"type": "auth_ack", "success": True, "message": "Authenticated successfully"}
            return {"type": "auth_ack", "success": False, "message": "Authentication failed"}

        # Enforce authentication for operational messages
        if self.config.require_auth and not authenticated:
            return {"type": "error", "code": 401, "message": "Unauthorized. Please authenticate first."}

        # 4. Mode Change
        if msg_type == "mode_change":
            self.active_mode = msg.get("mode", "absolute_annotation")
            # Always ensure release of buttons during mode switch
            self.input_backend.cleanup()
            return {"type": "mode_ack", "mode": self.active_mode}

        # 5. Drawing Events (Mode A)
        if msg_type == "stroke_start":
            p = Point(x=msg.get("x", 0.0), y=msg.get("y", 0.0), pressure=msg.get("pressure", 1.0), timestamp=msg.get("timestamp", 0))
            stroke = Stroke(
                stroke_id=msg.get("stroke_id", ""),
                tool=msg.get("tool", "pen"),
                color=msg.get("color", "#FF3366"),
                width=float(msg.get("width", 3.0)),
                points=[p]
            )
            self.renderer.start_stroke(stroke)
            if self.overlay_window:
                self.overlay_window.request_redraw()
            return None

        if msg_type == "stroke_move":
            stroke_id = msg.get("stroke_id", "")
            raw_points = msg.get("points", [])
            points = [Point.from_dict(p) for p in raw_points]
            self.renderer.append_points(stroke_id, points)
            if self.overlay_window:
                self.overlay_window.request_redraw()
            return None

        if msg_type == "stroke_end":
            stroke_id = msg.get("stroke_id", "")
            final_p = Point(x=msg.get("x", 0.0), y=msg.get("y", 0.0), pressure=msg.get("pressure", 0.0), timestamp=msg.get("timestamp", 0))
            self.renderer.end_stroke(stroke_id, final_p)
            if self.overlay_window:
                self.overlay_window.request_redraw()
            return None

        if msg_type == "undo":
            self.renderer.undo()
            if self.overlay_window:
                self.overlay_window.request_redraw()
            return {"type": "undo_ack"}

        if msg_type == "redo":
            self.renderer.redo()
            if self.overlay_window:
                self.overlay_window.request_redraw()
            return {"type": "redo_ack"}

        if msg_type == "clear":
            self.renderer.clear()
            if self.overlay_window:
                self.overlay_window.request_redraw()
            return {"type": "clear_ack"}

        if msg_type == "laser_pointer":
            self.renderer.set_laser(
                x=float(msg.get("x", 0.0)),
                y=float(msg.get("y", 0.0)),
                active=bool(msg.get("active", False))
            )
            if self.overlay_window:
                self.overlay_window.request_redraw()
            return None

        if msg_type == "overlay_toggle":
            visible = bool(msg.get("visible", True))
            if self.overlay_window:
                import gi
                gi.require_version("Gtk", "3.0")
                from gi.repository import GLib
                if visible:
                    GLib.idle_add(self.overlay_window.show_all)
                else:
                    GLib.idle_add(self.overlay_window.hide)
            return {"type": "overlay_status", "visible": visible}

        # 6. Tablet Input Injection Events (Mode B)
        if msg_type == "cursor_move":
            dx = float(msg.get("dx", 0.0))
            dy = float(msg.get("dy", 0.0))
            x = float(msg.get("x", 0.0))
            y = float(msg.get("y", 0.0))
            pressure = float(msg.get("pressure", 1.0))
            if "dx" in msg and "dy" in msg and (dx != 0.0 or dy != 0.0):
                self.input_backend.move_relative(dx, dy, pressure)
            else:
                self.input_backend.move_absolute(x, y, pressure)
            return None

        if msg_type == "pointer_down":
            btn = int(msg.get("button", 1))
            pressure = float(msg.get("pressure", 1.0))
            self.input_backend.button_down(btn, pressure)
            return None

        if msg_type == "pointer_up":
            btn = int(msg.get("button", 1))
            self.input_backend.button_up(btn)
            return None

        # 7. Heartbeat
        if msg_type == "ping":
            return {"type": "pong", "timestamp": msg.get("timestamp", 0)}

        if msg_type == "disconnect":
            self.input_backend.cleanup()
            return {"type": "disconnect_ack"}

        return None
