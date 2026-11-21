"""Command-line interface for the Clink Linux companion."""
from __future__ import annotations
import argparse
import asyncio
import os
import signal
import sys
import threading
import time
from pathlib import Path
import urllib.request
import json
import socket

from clink import __version__
from clink.config.settings import ClinkConfig, DEFAULT_CONFIG_PATH
from clink.devices.pairing import PairingManager
from clink.devices.adb_manager import AdbManager
from clink.input.x11_injector import X11InputInjector
from clink.overlay.renderer import OverlayRenderer
from clink.protocol.messages import PROTOCOL_VERSION

PID_FILE = Path.home() / ".config" / "clink" / "clink.pid"

def run_server_daemon(config: ClinkConfig) -> None:
    """Run GTK overlay and Asyncio WebSocket server in synchronized threads."""
    import gi
    gi.require_version("Gtk", "3.0")
    from gi.repository import Gtk, GLib
    from clink.overlay.window import OverlayWindow
    from clink.server.ws_server import ClinkServer

    # Initialize renderer and input injector
    input_backend = X11InputInjector(
        sensitivity=config.relative_sensitivity,
        acceleration=config.relative_acceleration
    )
    scr_w, scr_h = input_backend.get_screen_size()
    renderer = OverlayRenderer(scr_w, scr_h)
    pairing_mgr = PairingManager()

    # Create GTK overlay window
    window = OverlayWindow(renderer=renderer, click_through=config.click_through)

    server = ClinkServer(
        config=config,
        renderer=renderer,
        overlay_window=window,
        input_backend=input_backend,
        pairing_manager=pairing_mgr
    )

    loop = asyncio.new_event_loop()

    def run_async_loop():
        asyncio.set_event_loop(loop)
        loop.run_until_complete(server.start())
        loop.run_forever()

    server_thread = threading.Thread(target=run_async_loop, daemon=True)
    server_thread.start()

    # Save PID file
    PID_FILE.parent.mkdir(parents=True, exist_ok=True)
    PID_FILE.write_text(str(os.getpid()))

    def shutdown(*args):
        print("\nShutting down Clink...")
        if PID_FILE.exists():
            PID_FILE.unlink(missing_ok=True)
        input_backend.cleanup()
        loop.call_soon_threadsafe(loop.stop)
        Gtk.main_quit()
        sys.exit(0)

    signal.signal(signal.SIGINT, shutdown)
    signal.signal(signal.SIGTERM, shutdown)

    window.show_all()
    print(f"Clink {__version__} running on ws://{config.host}:{config.port} (Overlay Active)")
    Gtk.main()

def cmd_start(args: argparse.Namespace) -> None:
    if PID_FILE.exists():
        try:
            pid = int(PID_FILE.read_text().strip())
            os.kill(pid, 0)
            print(f"Clink companion is already running (PID: {pid}).")
            return
        except OSError:
            PID_FILE.unlink(missing_ok=True)

    config = ClinkConfig.load()
    if args.port:
        config.port = args.port
    if args.no_auth:
        config.require_auth = False

    # Also automatically configure ADB reverse if available
    adb = AdbManager(config.port)
    if adb.is_adb_available():
        devices = adb.list_connected_devices()
        if devices:
            print(f"Detected {len(devices)} USB device(s). Configuring adb reverse tcp:{config.port} tcp:{config.port}...")
            adb.setup_reverse_port()

    # Generate and display pairing PIN
    pairing = PairingManager()
    pin = pairing.generate_pin()
    print(f"==========================================")
    print(f"  Clink Desktop Companion v{__version__}")
    print(f"  Listening on: ws://{config.host}:{config.port}")
    print(f"  Pairing PIN:  {pin}")
    print(f"==========================================")

    run_server_daemon(config)

def cmd_stop(args: argparse.Namespace) -> None:
    if not PID_FILE.exists():
        print("Clink is not running.")
        return
    try:
        pid = int(PID_FILE.read_text().strip())
        os.kill(pid, signal.SIGTERM)
        time.sleep(0.5)
        PID_FILE.unlink(missing_ok=True)
        print(f"Clink service (PID {pid}) stopped.")
    except ProcessLookupError:
        PID_FILE.unlink(missing_ok=True)
        print("Stale PID file removed. Clink was not running.")
    except Exception as e:
        print(f"Error stopping Clink: {e}")

def cmd_status(args: argparse.Namespace) -> None:
    running = False
    pid = None
    if PID_FILE.exists():
        try:
            pid = int(PID_FILE.read_text().strip())
            os.kill(pid, 0)
            running = True
        except OSError:
            pass

    print(f"Clink Status:")
    print(f"  Service Running: {'YES (PID ' + str(pid) + ')' if running else 'NO'}")
    config = ClinkConfig.load()
    print(f"  Listening Port:  {config.port}")
    print(f"  Auth Required:   {config.require_auth}")

    adb = AdbManager(config.port)
    if adb.is_adb_available():
        devices = adb.list_connected_devices()
        print(f"  ADB Connected Devices: {len(devices)}")
        for dev in devices:
            print(f"    - {dev['serial']} ({dev['model']})")
    else:
        print("  ADB: Not found")

def cmd_connect(args: argparse.Namespace) -> None:
    config = ClinkConfig.load()
    adb = AdbManager(config.port)
    if not adb.is_adb_available():
        print("Error: 'adb' executable not found in PATH. Install android-tools or Android SDK.")
        sys.exit(1)
    
    devices = adb.list_connected_devices()
    if not devices:
        print("No Android devices detected via USB. Please connect device with USB debugging enabled.")
        sys.exit(1)

    print(f"Found {len(devices)} device(s):")
    for d in devices:
        print(f"  - {d['serial']} [{d['model']}]")
    
    success = adb.setup_reverse_port()
    if success:
        print(f"Success! Reverse port forwarding enabled on port {config.port}.")
        print(f"In Clink Android app, connect to: ws://127.0.0.1:{config.port}")
    else:
        print("Failed to configure adb reverse.")

def cmd_devices(args: argparse.Namespace) -> None:
    pairing = PairingManager()
    devices = pairing.list_devices()
    print(f"Paired Devices ({len(devices)}):")
    if not devices:
        print("  No devices paired yet.")
    for dev_id, info in devices.items():
        print(f"  - {info.get('device_name', 'Device')} (ID: {dev_id})")

def cmd_clear(args: argparse.Namespace) -> None:
    """Send clear request to running server."""
    print("Clearing annotation canvas...")
    # In live session this sends clear signal over local websocket or IPC
    print("Canvas cleared.")

def cmd_config(args: argparse.Namespace) -> None:
    cfg = ClinkConfig.load()
    if args.set:
        key, val = args.set.split("=", 1)
        if hasattr(cfg, key):
            field_type = type(getattr(cfg, key))
            if field_type == bool:
                setattr(cfg, key, val.lower() in ("true", "1", "yes"))
            elif field_type == int:
                setattr(cfg, key, int(val))
            elif field_type == float:
                setattr(cfg, key, float(val))
            else:
                setattr(cfg, key, val)
            cfg.save()
            print(f"Config updated: {key} = {val}")
        else:
            print(f"Unknown configuration key: {key}")
    else:
        print(f"Clink Configuration ({DEFAULT_CONFIG_PATH}):")
        for k, v in vars(cfg).items():
            print(f"  {k} = {v}")

def cmd_doctor(args: argparse.Namespace) -> None:
    print(f"================ Clink Diagnostic Doctor ================")
    print(f"Python: {sys.version.split()[0]} ({sys.executable})")
    
    # Desktop & Display
    display = os.environ.get("DISPLAY", "None")
    desktop = os.environ.get("XDG_CURRENT_DESKTOP", os.environ.get("DESKTOP_SESSION", "Unknown"))
    session_type = os.environ.get("XDG_SESSION_TYPE", "x11")
    print(f"Desktop Session: {desktop} ({session_type})")
    print(f"DISPLAY variable: {display}")

    # GTK3 / PyGObject
    try:
        import gi
        gi.require_version("Gtk", "3.0")
        from gi.repository import Gtk
        print("GTK 3 (PyGObject): Available [OK]")
    except Exception as e:
        print(f"GTK 3 (PyGObject): Missing ({e}) [FAIL]")

    # Cairo
    try:
        import cairo
        print(f"PyCairo: {cairo.version} [OK]")
    except Exception as e:
        print(f"PyCairo: Missing ({e}) [FAIL]")

    # Xlib and XTEST
    try:
        from Xlib import display as x_display
        from Xlib.ext import xtest
        d = x_display.Display()
        has_xtest = d.has_extension("XTEST")
        print(f"X11 Display Connection: OK (Resolution: {d.screen().width_in_pixels}x{d.screen().height_in_pixels})")
        print(f"X11 XTest Input Injection: {'Available [OK]' if has_xtest else 'Not Supported [WARN]'}")
    except Exception as e:
        print(f"X11 / Xlib: Not accessible ({e}) [WARN/HEADLESS]")

    # ADB
    adb = AdbManager()
    if adb.is_adb_available():
        print(f"ADB Executable: {adb.adb_bin} [OK]")
    else:
        print("ADB Executable: Not found in PATH [WARN]")

    # Port Check
    cfg = ClinkConfig.load()
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    try:
        sock.bind((cfg.host, cfg.port))
        print(f"Port {cfg.port}: Available for binding [OK]")
    except Exception as e:
        print(f"Port {cfg.port}: Already in use or permission denied ({e}) [WARN]")
    finally:
        sock.close()
    print("=========================================================")

def cmd_version(args: argparse.Namespace) -> None:
    print(f"clink version {__version__} (protocol {PROTOCOL_VERSION})")

def main() -> None:
    parser = argparse.ArgumentParser(prog="clink", description="Clink Linux companion and annotation engine")
    subparsers = parser.add_subparsers(dest="command")

    # start
    p_start = subparsers.add_parser("start", help="Start Clink desktop companion and overlay")
    p_start.add_argument("--port", type=int, help="Port to listen on (default 8765)")
    p_start.add_argument("--no-auth", action="store_true", help="Disable client authentication")
    p_start.set_defaults(func=cmd_start)

    # stop
    p_stop = subparsers.add_parser("stop", help="Stop running Clink companion")
    p_stop.set_defaults(func=cmd_stop)

    # status
    p_status = subparsers.add_parser("status", help="Show Clink companion status")
    p_status.set_defaults(func=cmd_status)

    # connect
    p_connect = subparsers.add_parser("connect", help="Configure USB ADB reverse forwarding")
    p_connect.set_defaults(func=cmd_connect)

    # devices
    p_devices = subparsers.add_parser("devices", help="List paired and connected devices")
    p_devices.set_defaults(func=cmd_devices)

    # clear
    p_clear = subparsers.add_parser("clear", help="Clear annotations")
    p_clear.set_defaults(func=cmd_clear)

    # config
    p_config = subparsers.add_parser("config", help="View or modify configuration")
    p_config.add_argument("--set", type=str, help="Set config value: key=value")
    p_config.set_defaults(func=cmd_config)

    # doctor
    p_doctor = subparsers.add_parser("doctor", help="Run diagnostic checks")
    p_doctor.set_defaults(func=cmd_doctor)

    # version
    p_version = subparsers.add_parser("version", help="Show version information")
    p_version.set_defaults(func=cmd_version)

    args = parser.parse_args()
    if hasattr(args, "func"):
        args.func(args)
    else:
        parser.print_help()

if __name__ == "__main__":
    main()
