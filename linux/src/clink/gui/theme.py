"""Custom modern dark / cyberpunk theme for Clink Linux Control Center."""
from __future__ import annotations
import logging

logger = logging.getLogger(__name__)

CLINK_GTK_CSS = """
/* Clink Custom Cyberpunk & Dark Theme (Matching Android App) */
window.clink-window {
    background-color: #121218;
    color: #F0F0FA;
}

headerbar.clink-header {
    background: linear-gradient(180deg, #181826 0%, #13131F 100%);
    border-bottom: 1px solid #28283E;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.5);
    padding: 6px 12px;
}

headerbar.clink-header .title {
    color: #FFFFFF;
    font-weight: 800;
    font-size: 13pt;
    letter-spacing: 0.5px;
}

headerbar.clink-header .subtitle {
    color: #FF3366;
    font-weight: 700;
    font-size: 9.5pt;
}

/* Card Containers */
.clink-card {
    background-color: #191926;
    border: 1px solid #28283D;
    border-radius: 14px;
    padding: 14px 16px;
    margin-bottom: 2px;
    box-shadow: 0 4px 14px rgba(0, 0, 0, 0.3);
}

.clink-card-header {
    font-weight: 800;
    font-size: 11pt;
    color: #00E5FF;
    letter-spacing: 0.8px;
    margin-bottom: 8px;
}

.clink-subtext {
    color: #9E9EB8;
    font-size: 9.5pt;
}

/* Address Code Box */
.clink-address-box {
    background-color: #0F0F16;
    border: 1px solid #222234;
    border-radius: 10px;
    padding: 10px 14px;
    font-family: monospace, "Fira Code", "Courier New", monospace;
    font-size: 9.5pt;
    color: #D6D6F0;
    line-height: 1.5;
}

/* QR Code Container Frame */
.clink-qr-frame {
    background-color: #FFFFFF;
    border-radius: 12px;
    padding: 8px;
    border: 2px solid #FF3366;
    box-shadow: 0 0 20px rgba(255, 51, 102, 0.25);
}

/* Primary Action Buttons (Pink Gradient) */
.clink-btn-primary {
    background: linear-gradient(135deg, #FF3366 0%, #D81B60 100%);
    color: #FFFFFF;
    font-weight: 700;
    font-size: 10pt;
    border-radius: 9px;
    border: none;
    padding: 9px 18px;
    box-shadow: 0 3px 8px rgba(255, 51, 102, 0.35);
    transition: all 180ms ease-in-out;
}

.clink-btn-primary:hover {
    background: linear-gradient(135deg, #FF4D79 0%, #FF3366 100%);
    box-shadow: 0 5px 16px rgba(255, 51, 102, 0.55);
}

.clink-btn-primary:active {
    background-color: #C2185B;
    box-shadow: 0 1px 4px rgba(255, 51, 102, 0.4);
}

/* Secondary Action Buttons (Dark Glass) */
.clink-btn-secondary {
    background-color: #242436;
    color: #E2E2F8;
    border: 1px solid #383852;
    border-radius: 9px;
    padding: 8px 16px;
    font-weight: 600;
    font-size: 10pt;
    transition: all 180ms ease-in-out;
}

.clink-btn-secondary:hover {
    background-color: #313148;
    border-color: #505075;
    color: #FFFFFF;
}

/* Danger / Destructive Action Buttons */
.clink-btn-danger {
    background-color: #26121C;
    color: #FF4D6D;
    border: 1px solid #4D1A2B;
    border-radius: 9px;
    padding: 8px 16px;
    font-weight: 700;
    font-size: 10pt;
    transition: all 180ms ease-in-out;
}

.clink-btn-danger:hover {
    background-color: #381626;
    border-color: #6E233C;
    color: #FF6685;
}

/* Badges */
.clink-badge-active {
    background-color: #0D261A;
    color: #00E676;
    border: 1px solid #144D33;
    border-radius: 20px;
    padding: 5px 12px;
    font-weight: 800;
    font-size: 9.5pt;
}

.clink-badge-info {
    background-color: #0D2230;
    color: #00E5FF;
    border: 1px solid #134661;
    border-radius: 20px;
    padding: 5px 12px;
    font-weight: 800;
    font-size: 9.5pt;
}

.clink-badge-stopped {
    background-color: #2A1017;
    color: #FF3366;
    border: 1px solid #4A1926;
    border-radius: 20px;
    padding: 5px 12px;
    font-weight: 800;
    font-size: 9.5pt;
}

/* Switches */
switch {
    background-color: #26263A;
    border: 1px solid #3A3A54;
    border-radius: 16px;
}

switch:checked {
    background-color: #FF3366;
    border-color: #FF3366;
}

switch slider {
    background-color: #FFFFFF;
    border-radius: 14px;
    min-width: 22px;
    min-height: 22px;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
}

/* ComboBox Dropdown */
combobox button {
    background-color: #242436;
    color: #FFFFFF;
    border: 1px solid #383852;
    border-radius: 9px;
    padding: 6px 14px;
    font-weight: 600;
}

combobox button:hover {
    background-color: #2E2E44;
    border-color: #4C4C6E;
}

/* Scrollbars */
scrollbar {
    background-color: transparent;
}

scrollbar slider {
    background-color: #2D2D42;
    border-radius: 6px;
    min-width: 6px;
    min-height: 24px;
}

scrollbar slider:hover {
    background-color: #434362;
}
"""

def apply_clink_theme() -> bool:
    """Inject the Clink custom CSS theme into the default GDK screen."""
    try:
        import gi
        gi.require_version("Gtk", "3.0")
        gi.require_version("Gdk", "3.0")
        from gi.repository import Gtk, Gdk
        if not Gtk.init_check()[0]:
            return False
        screen = Gdk.Screen.get_default()
        if not screen:
            return False
        provider = Gtk.CssProvider()
        provider.load_from_data(CLINK_GTK_CSS.encode("utf-8"))
        Gtk.StyleContext.add_provider_for_screen(
            screen, provider, Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        return True
    except Exception as e:
        logger.debug("Failed to apply Clink CSS theme: %s", e)
        return False
