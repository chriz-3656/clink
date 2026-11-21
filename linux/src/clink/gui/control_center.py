"""GTK3-based desktop GUI Control Center for Clink."""
from __future__ import annotations
import asyncio
import logging
import os
import signal
import socket
import sys
import threading
import time
from pathlib import Path
from typing import Optional

try:
    import gi
    gi.require_version("Gtk", "3.0")
    gi.require_version("Gdk", "3.0")
    from gi.repository import Gtk, Gdk, GLib
    HAS_GTK = bool(Gtk.init_check()[0])
except Exception:
    HAS_GTK = False
    Gtk = object  # type: ignore
    Gdk = object  # type: ignore
    GLib = object  # type: ignore

from clink import __version__
from clink.config.settings import ClinkConfig
from clink.devices.pairing import PairingManager
from clink.devices.adb_manager import AdbManager
from clink.devices.qr_manager import generate_pairing_qr_image, QR_CACHE_PATH
from clink.input.x11_injector import X11InputInjector
from clink.overlay.renderer import OverlayRenderer
from clink.overlay.window import OverlayWindow
from clink.server.ws_server import ClinkServer
from clink.protocol.messages import PROTOCOL_VERSION

logger = logging.getLogger(__name__)

PID_FILE = Path.home() / ".config" / "clink" / "clink.pid"

def get_lan_ip() -> str:
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    try:
        s.connect(('10.255.255.255', 1))
        ip = s.getsockname()[0]
    except Exception:
        ip = '127.0.0.1'
    finally:
        s.close()
    return ip

class ControlCenterWindow(Gtk.Window):
    def __init__(self, config: Optional[ClinkConfig] = None):
        super().__init__(title=f"Clink Desktop Companion v{__version__}")
        self.config = config or ClinkConfig.load()
        self.pairing_mgr = PairingManager()
        self.adb_mgr = AdbManager(self.config.port)

        self.input_backend = X11InputInjector(
            sensitivity=self.config.relative_sensitivity,
            acceleration=self.config.relative_acceleration
        )
        scr_w, scr_h = self.input_backend.get_screen_size()
        self.renderer = OverlayRenderer(scr_w, scr_h)
        self.overlay_window = OverlayWindow(renderer=self.renderer, click_through=self.config.click_through)

        self.server = ClinkServer(
            config=self.config,
            renderer=self.renderer,
            overlay_window=self.overlay_window,
            input_backend=self.input_backend,
            pairing_manager=self.pairing_mgr
        )

        self.loop: Optional[asyncio.AbstractEventLoop] = None
        self.server_thread: Optional[threading.Thread] = None
        self.is_running = False
        self._adb_configured = False

        self.set_default_size(540, 740)
        self.set_position(Gtk.WindowPosition.CENTER)
        self.connect("destroy", self.on_close)

        self._build_ui()
        self.start_server()

        if HAS_GTK:
            GLib.timeout_add_seconds(3, self._auto_check_usb)

    def _build_ui(self) -> None:
        header = Gtk.HeaderBar()
        header.set_show_close_button(True)
        header.set_title("Clink")
        header.set_subtitle(f"v{__version__} • One Canvas")
        self.set_titlebar(header)

        scroller = Gtk.ScrolledWindow()
        scroller.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        self.add(scroller)

        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=16)
        main_box.set_margin_top(16)
        main_box.set_margin_bottom(16)
        main_box.set_margin_start(18)
        main_box.set_margin_end(18)
        scroller.add(main_box)

        # 1. Status & Service Toggle Card
        status_frame = Gtk.Frame(label=" Companion Service ")
        status_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        status_box.set_margin_top(12)
        status_box.set_margin_bottom(12)
        status_box.set_margin_start(14)
        status_box.set_margin_end(14)
        status_frame.add(status_box)
        main_box.pack_start(status_frame, False, False, 0)

        status_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=12)
        self.status_label = Gtk.Label(label="Service Status: Initializing...")
        self.status_label.set_xalign(0)
        status_row.pack_start(self.status_label, True, True, 0)

        self.service_toggle = Gtk.Switch()
        self.service_toggle.set_active(True)
        self.service_toggle.connect("notify::active", self.on_service_toggled)
        status_row.pack_end(self.service_toggle, False, False, 0)
        status_box.pack_start(status_row, False, False, 0)

        self.lan_ip = get_lan_ip()
        self.address_label = Gtk.Label(
            label=f"Local (USB): ws://127.0.0.1:{self.config.port}\nWi-Fi LAN:   ws://{self.lan_ip}:{self.config.port}"
        )
        self.address_label.set_xalign(0)
        status_box.pack_start(self.address_label, False, False, 0)

        # 2. Pairing PIN & QR Code Card
        pin_frame = Gtk.Frame(label=" Instant Device Pairing (PIN & QR Code) ")
        pin_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=12)
        pin_box.set_margin_top(12)
        pin_box.set_margin_bottom(12)
        pin_box.set_margin_start(14)
        pin_box.set_margin_end(14)
        pin_frame.add(pin_box)
        main_box.pack_start(pin_frame, False, False, 0)

        pairing_content_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=16)

        # Left column: PIN & New PIN button
        pin_info_col = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        current_pin = self.pairing_mgr.get_current_pin()
        formatted_pin = f"{current_pin[:3]} {current_pin[3:]}" if len(current_pin) == 6 else current_pin
        
        self.pin_display = Gtk.Label()
        self.pin_display.set_markup(f"<span size='xx-large' weight='bold' foreground='#FF3366'>{formatted_pin}</span>")
        self.pin_display.set_xalign(0)
        pin_info_col.pack_start(self.pin_display, False, False, 0)

        pin_btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=8)
        regen_btn = Gtk.Button(label="Generate New PIN")
        regen_btn.connect("clicked", self.on_regenerate_pin)
        pin_btn_row.pack_start(regen_btn, True, True, 0)

        refresh_ip_btn = Gtk.Button(label="Refresh Network IP")
        refresh_ip_btn.connect("clicked", lambda b: self.refresh_network_ip(force=True))
        pin_btn_row.pack_end(refresh_ip_btn, False, False, 0)
        pin_info_col.pack_start(pin_btn_row, False, False, 0)

        pin_hint = Gtk.Label(
            label="Scan the QR code with Clink on Android or enter the 6-digit PIN to pair securely over Wi-Fi / Hotspot."
        )
        pin_hint.set_line_wrap(True)
        pin_hint.set_xalign(0)
        pin_info_col.pack_start(pin_hint, False, False, 0)
        pairing_content_row.pack_start(pin_info_col, True, True, 0)

        # Right column: QR Code Image
        qr_path = generate_pairing_qr_image(self.lan_ip, self.config.port, current_pin)
        self.qr_image = Gtk.Image.new_from_file(str(qr_path))
        pairing_content_row.pack_end(self.qr_image, False, False, 0)

        pin_box.pack_start(pairing_content_row, False, False, 0)

        # 3. Quick USB Setup Card
        usb_frame = Gtk.Frame(label=" USB Connection (ADB) ")
        usb_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        usb_box.set_margin_top(12)
        usb_box.set_margin_bottom(12)
        usb_box.set_margin_start(14)
        usb_box.set_margin_end(14)
        usb_frame.add(usb_box)
        main_box.pack_start(usb_frame, False, False, 0)

        self.usb_status_label = Gtk.Label(label="Checking for connected USB devices...")
        self.usb_status_label.set_xalign(0)
        usb_box.pack_start(self.usb_status_label, False, False, 0)

        usb_btn_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        self.adb_reverse_btn = Gtk.Button(label="Configure USB (adb reverse)")
        self.adb_reverse_btn.connect("clicked", self.on_configure_adb_reverse)
        usb_btn_row.pack_start(self.adb_reverse_btn, True, True, 0)

        refresh_adb_btn = Gtk.Button(label="Refresh USB")
        refresh_adb_btn.connect("clicked", self.on_refresh_usb)
        usb_btn_row.pack_end(refresh_adb_btn, False, False, 0)
        usb_box.pack_start(usb_btn_row, False, False, 0)

        # 4. Desktop Overlay Controls Card
        overlay_frame = Gtk.Frame(label=" Desktop Annotation Overlay ")
        overlay_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        overlay_box.set_margin_top(12)
        overlay_box.set_margin_bottom(12)
        overlay_box.set_margin_start(14)
        overlay_box.set_margin_end(14)
        overlay_frame.add(overlay_box)
        main_box.pack_start(overlay_frame, False, False, 0)

        overlay_row1 = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        overlay_vis_label = Gtk.Label(label="Overlay Visible:")
        overlay_vis_label.set_xalign(0)
        overlay_row1.pack_start(overlay_vis_label, True, True, 0)
        self.overlay_switch = Gtk.Switch()
        self.overlay_switch.set_active(True)
        self.overlay_switch.connect("notify::active", self.on_overlay_visibility_toggled)
        overlay_row1.pack_end(self.overlay_switch, False, False, 0)
        overlay_box.pack_start(overlay_row1, False, False, 0)

        overlay_row2 = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        clickthrough_label = Gtk.Label(label="Click-Through Mode (pass clicks to desktop):")
        clickthrough_label.set_xalign(0)
        overlay_row2.pack_start(clickthrough_label, True, True, 0)
        self.clickthrough_switch = Gtk.Switch()
        self.clickthrough_switch.set_active(self.config.click_through)
        self.clickthrough_switch.connect("notify::active", self.on_clickthrough_toggled)
        overlay_row2.pack_end(self.clickthrough_switch, False, False, 0)
        overlay_box.pack_start(overlay_row2, False, False, 0)

        # Annotation auto-fade timer row
        timer_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        timer_label = Gtk.Label(label="Annotation Duration / Auto-Clear:")
        timer_label.set_xalign(0)
        timer_row.pack_start(timer_label, True, True, 0)

        self.timer_combo = Gtk.ComboBoxText()
        self.timer_combo.append("0", "Never (Persistent)")
        self.timer_combo.append("3", "3 seconds")
        self.timer_combo.append("5", "5 seconds")
        self.timer_combo.append("10", "10 seconds")
        self.timer_combo.append("30", "30 seconds")
        self.timer_combo.append("60", "60 seconds")
        self.timer_combo.set_active_id("0")
        self.timer_combo.connect("changed", self.on_timer_changed)
        timer_row.pack_end(self.timer_combo, False, False, 0)
        overlay_box.pack_start(timer_row, False, False, 0)

        action_row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
        clear_btn = Gtk.Button(label="Clear All Annotations")
        clear_btn.connect("clicked", self.on_clear_canvas)
        action_row.pack_start(clear_btn, True, True, 0)

        undo_btn = Gtk.Button(label="Undo")
        undo_btn.connect("clicked", lambda b: self.renderer.undo() or self.overlay_window.request_redraw())
        action_row.pack_start(undo_btn, True, True, 0)

        redo_btn = Gtk.Button(label="Redo")
        redo_btn.connect("clicked", lambda b: self.renderer.redo() or self.overlay_window.request_redraw())
        action_row.pack_start(redo_btn, True, True, 0)

        overlay_box.pack_start(action_row, False, False, 0)

        # 5. Paired Devices List Card
        devices_frame = Gtk.Frame(label=" Paired Android Devices ")
        self.devices_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=6)
        self.devices_box.set_margin_top(12)
        self.devices_box.set_margin_bottom(12)
        self.devices_box.set_margin_start(14)
        self.devices_box.set_margin_end(14)
        devices_frame.add(self.devices_box)
        main_box.pack_start(devices_frame, False, False, 0)
        self.refresh_devices_list()

        # 6. Developer Information Card
        dev_frame = Gtk.Frame(label=" Developer & Project Information ")
        dev_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=8)
        dev_box.set_margin_top(12)
        dev_box.set_margin_bottom(12)
        dev_box.set_margin_start(14)
        dev_box.set_margin_end(14)

        dev_lbl = Gtk.Label()
        dev_lbl.set_markup("<b>Developer:</b> chriz-3656\n<b>GitHub:</b> <a href='https://github.com/chriz-3656/clink'>https://github.com/chriz-3656/clink</a>\n<b>Contact:</b> <a href='mailto:chrizmonsaji@proton.me'>chrizmonsaji@proton.me</a>")
        dev_lbl.set_xalign(0)
        dev_lbl.set_line_wrap(True)
        dev_box.pack_start(dev_lbl, False, False, 0)
        dev_frame.add(dev_box)
        main_box.pack_start(dev_frame, False, False, 0)

        # Update USB status on launch
        self.on_refresh_usb()

    def start_server(self) -> None:
        if self.is_running:
            return
        self.loop = asyncio.new_event_loop()

        def _run():
            asyncio.set_event_loop(self.loop)
            self.loop.run_until_complete(self.server.start())
            self.loop.run_forever()

        self.server_thread = threading.Thread(target=_run, daemon=True)
        self.server_thread.start()
        self.is_running = True

        if self.config.overlay_enabled:
            self.overlay_window.show_all()
            self.overlay_window.apply_click_through(self.config.click_through)

        # Auto-configure ADB reverse on server start if device is plugged in
        if self.adb_mgr.is_adb_available():
            devs = self.adb_mgr.list_connected_devices()
            if devs:
                if self.adb_mgr.setup_reverse_port():
                    self._adb_configured = True
                    self.usb_status_label.set_markup(
                        f"<span foreground='#00E676'><b>USB Active:</b> {devs[0]['model']} (Port {self.config.port} forwarded)</span>"
                    )

        self.status_label.set_markup("Service Status: <span foreground='#00E676' weight='bold'>Active &amp; Listening</span>")
        PID_FILE.parent.mkdir(parents=True, exist_ok=True)
        PID_FILE.write_text(str(os.getpid()))

    def refresh_network_ip(self, force: bool = False) -> None:
        new_ip = get_lan_ip()
        if new_ip != self.lan_ip or force:
            self.lan_ip = new_ip
            self.address_label.set_text(
                f"Local (USB): ws://127.0.0.1:{self.config.port}\nWi-Fi / Hotspot: ws://{self.lan_ip}:{self.config.port}"
            )
            current_pin = self.pairing_mgr.get_current_pin()
            qr_path = generate_pairing_qr_image(self.lan_ip, self.config.port, current_pin)
            self.qr_image.set_from_file(str(qr_path))

    def _auto_check_usb(self) -> bool:
        if self.is_running:
            self.refresh_network_ip(force=False)
            if self.adb_mgr.is_adb_available():
                devices = self.adb_mgr.list_connected_devices()
                if devices and not self._adb_configured:
                    if self.adb_mgr.setup_reverse_port():
                        self._adb_configured = True
                        info = f"{devices[0]['model']} ({devices[0]['serial']})"
                        self.usb_status_label.set_markup(
                            f"<span foreground='#00E676'><b>USB Active:</b> {info} (Port {self.config.port} forwarded)</span>"
                        )
                elif not devices:
                    self._adb_configured = False
        return True

    def stop_server(self) -> None:
        if not self.is_running or not self.loop:
            return
        self.input_backend.cleanup()
        self.loop.call_soon_threadsafe(self.loop.stop)
        self.overlay_window.hide()
        self.is_running = False
        self.status_label.set_markup("Service Status: <span foreground='#FF3366' weight='bold'>Stopped</span>")
        if PID_FILE.exists():
            PID_FILE.unlink(missing_ok=True)

    def on_service_toggled(self, switch: Gtk.Switch, param) -> None:
        if switch.get_active():
            self.start_server()
        else:
            self.stop_server()

    def on_regenerate_pin(self, button: Gtk.Button) -> None:
        self.refresh_network_ip(force=False)
        new_pin = self.pairing_mgr.generate_pin()
        formatted = f"{new_pin[:3]} {new_pin[3:]}" if len(new_pin) == 6 else new_pin
        self.pin_display.set_markup(f"<span size='xx-large' weight='bold' foreground='#FF3366'>{formatted}</span>")
        qr_path = generate_pairing_qr_image(self.lan_ip, self.config.port, new_pin)
        self.qr_image.set_from_file(str(qr_path))

    def on_refresh_usb(self, *args) -> None:
        if not self.adb_mgr.is_adb_available():
            self.usb_status_label.set_text("ADB not found in PATH.")
            return
        devices = self.adb_mgr.list_connected_devices()
        if devices:
            info = ", ".join(f"{d['model']} ({d['serial']})" for d in devices)
            self.usb_status_label.set_markup(f"USB Device Detected: <b>{info}</b>")
        else:
            self.usb_status_label.set_text("No USB devices detected. Connect phone with USB Debugging enabled.")

    def on_configure_adb_reverse(self, button: Gtk.Button) -> None:
        success = self.adb_mgr.setup_reverse_port()
        if success:
            self.usb_status_label.set_markup(
                f"<span foreground='#00E676'><b>Success!</b> Port {self.config.port} forwarded via USB ADB reverse.</span>"
            )
        else:
            self.usb_status_label.set_markup("<span foreground='#FF3366'>Failed to configure ADB reverse. Check USB connection.</span>")

    def on_overlay_visibility_toggled(self, switch: Gtk.Switch, param) -> None:
        if switch.get_active():
            self.overlay_window.show_all()
            self.overlay_window.apply_click_through(self.clickthrough_switch.get_active())
        else:
            self.overlay_window.hide()

    def on_clickthrough_toggled(self, switch: Gtk.Switch, param) -> None:
        self.overlay_window.set_click_through(switch.get_active())

    def on_timer_changed(self, combo: Gtk.ComboBoxText) -> None:
        val = float(combo.get_active_id() or "0")
        self.renderer.fade_timeout = val

    def on_clear_canvas(self, button: Gtk.Button) -> None:
        self.renderer.clear()
        self.overlay_window.request_redraw()

    def refresh_devices_list(self) -> None:
        for child in self.devices_box.get_children():
            self.devices_box.remove(child)

        devices = self.pairing_mgr.list_devices()
        if not devices:
            lbl = Gtk.Label(label="No paired Android devices yet.")
            lbl.set_xalign(0)
            self.devices_box.pack_start(lbl, False, False, 0)
        else:
            for dev_id, info in devices.items():
                row = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL, spacing=10)
                lbl = Gtk.Label(label=f"• {info.get('device_name', 'Android')} ({dev_id})")
                lbl.set_xalign(0)
                row.pack_start(lbl, True, True, 0)

                unpair_btn = Gtk.Button(label="Unpair")
                unpair_btn.connect("clicked", lambda b, d=dev_id: self.on_unpair_device(d))
                row.pack_end(unpair_btn, False, False, 0)
                self.devices_box.pack_start(row, False, False, 0)
        self.devices_box.show_all()

    def on_unpair_device(self, dev_id: str) -> None:
        self.pairing_mgr.remove_device(dev_id)
        self.refresh_devices_list()

    def on_close(self, *args) -> None:
        self.stop_server()
        if hasattr(self.overlay_window, "destroy"):
            self.overlay_window.destroy()
        Gtk.main_quit()

def launch_control_center() -> None:
    if not HAS_GTK:
        print("Error: GTK 3 is not available. Please install gir1.2-gtk-3.0 and python3-gi.", file=sys.stderr)
        sys.exit(1)
    window = ControlCenterWindow()
    window.show_all()
    Gtk.main()
