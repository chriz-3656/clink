"""Integration tests for Clink WebSocket server."""
import asyncio
import json
import pytest
import websockets

from clink.config.settings import ClinkConfig
from clink.devices.pairing import PairingManager
from clink.overlay.renderer import OverlayRenderer
from clink.server.ws_server import ClinkServer
from clink.protocol.messages import PROTOCOL_VERSION
from test_input import MockInputBackend

def test_server_handshake_and_drawing(tmp_path):
    async def _run():
        config = ClinkConfig(port=9876, require_auth=False)
        renderer = OverlayRenderer(1920, 1080)
        input_backend = MockInputBackend()
        pairing_mgr = PairingManager(devices_file=tmp_path / "devices.json")

        server = ClinkServer(
            config=config,
            renderer=renderer,
            overlay_window=None,
            input_backend=input_backend,
            pairing_manager=pairing_mgr
        )

        await server.start()

        try:
            uri = f"ws://127.0.0.1:{config.port}"
            async with websockets.connect(uri) as ws:
                # 1. Hello
                await ws.send(json.dumps({"type": "hello", "version": PROTOCOL_VERSION}))
                ack = json.loads(await ws.recv())
                assert ack["type"] == "hello_ack"
                assert ack["authenticated"] is True
                assert ack["display"]["width"] == 1920

                # 2. Stroke Start
                await ws.send(json.dumps({
                    "type": "stroke_start",
                    "stroke_id": "test_s1",
                    "tool": "pen",
                    "color": "#FF0000",
                    "width": 3.0,
                    "x": 0.1,
                    "y": 0.1,
                    "pressure": 0.8
                }))

                # 3. Stroke Move
                await ws.send(json.dumps({
                    "type": "stroke_move",
                    "stroke_id": "test_s1",
                    "points": [{"x": 0.2, "y": 0.2, "pressure": 0.9}]
                }))

                # 4. Stroke End
                await ws.send(json.dumps({
                    "type": "stroke_end",
                    "stroke_id": "test_s1",
                    "x": 0.3,
                    "y": 0.3
                }))

                # 5. Undo & Redo
                await ws.send(json.dumps({"type": "undo"}))
                undo_ack = json.loads(await ws.recv())
                assert undo_ack["type"] == "undo_ack"

                await ws.send(json.dumps({"type": "redo"}))
                redo_ack = json.loads(await ws.recv())
                assert redo_ack["type"] == "redo_ack"

                # 6. Mode Change to Tablet
                await ws.send(json.dumps({"type": "mode_change", "mode": "relative_tablet"}))
                mode_ack = json.loads(await ws.recv())
                assert mode_ack["type"] == "mode_ack"
                assert mode_ack["mode"] == "relative_tablet"

                # 7. Pointer events
                await ws.send(json.dumps({"type": "cursor_move", "dx": 0.05, "dy": -0.02}))
                await ws.send(json.dumps({"type": "pointer_down", "button": 1})); await asyncio.sleep(0.02)
                assert 1 in input_backend.pressed
                await ws.send(json.dumps({"type": "pointer_up", "button": 1})); await asyncio.sleep(0.02)
                assert 1 not in input_backend.pressed

                # 8. Ping / Pong
                await ws.send(json.dumps({"type": "ping", "timestamp": 12345678}))
                pong = json.loads(await ws.recv())
                assert pong["type"] == "pong"
                assert pong["timestamp"] == 12345678

                # 9. Disconnect cleanup
                await ws.send(json.dumps({"type": "pointer_down", "button": 2})); await asyncio.sleep(0.02)
                assert 2 in input_backend.pressed
                await ws.send(json.dumps({"type": "disconnect"}))
                disc_ack = json.loads(await ws.recv())
                assert disc_ack["type"] == "disconnect_ack"
                assert 2 not in input_backend.pressed

        finally:
            await server.stop()

    asyncio.run(_run())

def test_server_auth_enforcement(tmp_path):
    async def _run():
        config = ClinkConfig(port=9877, require_auth=True)
        renderer = OverlayRenderer(800, 600)
        input_backend = MockInputBackend()
        pairing_mgr = PairingManager(devices_file=tmp_path / "devices.json")

        server = ClinkServer(
            config=config,
            renderer=renderer,
            overlay_window=None,
            input_backend=input_backend,
            pairing_manager=pairing_mgr
        )
        await server.start()

        try:
            uri = f"ws://127.0.0.1:{config.port}"
            async with websockets.connect(uri) as ws:
                # Operational command without auth should fail
                await ws.send(json.dumps({"type": "clear"}))
                err = json.loads(await ws.recv())
                assert err["type"] == "error"
                assert err["code"] == 401

                # Pair with PIN
                pin = pairing_mgr.generate_pin()
                await ws.send(json.dumps({
                    "type": "pair_request",
                    "device_id": "test_dev",
                    "device_name": "Test Device",
                    "pin": pin
                }))
                pair_ack = json.loads(await ws.recv())
                assert pair_ack["type"] == "pair_ack"
                assert pair_ack["success"] is True
                token = pair_ack["token"]

                # Authenticate
                await ws.send(json.dumps({"type": "auth", "token": token}))
                auth_ack = json.loads(await ws.recv())
                assert auth_ack["type"] == "auth_ack"
                assert auth_ack["success"] is True

                # Now clear succeeds
                await ws.send(json.dumps({"type": "clear"}))
                clear_ack = json.loads(await ws.recv())
                assert clear_ack["type"] == "clear_ack"

        finally:
            await server.stop()

    asyncio.run(_run())
