"""QR code generation for seamless mobile device pairing."""
from __future__ import annotations
import io
import logging
from pathlib import Path
from typing import Optional
import qrcode

logger = logging.getLogger(__name__)

QR_CACHE_PATH = Path.home() / ".config" / "clink" / "pairing_qr.png"

def make_pairing_uri(host: str, port: int, pin: str) -> str:
    """Format standard Clink pairing URI scheme."""
    return f"clink://pair?host={host}&port={port}&pin={pin}"

def generate_pairing_qr_image(host: str, port: int, pin: str, output_path: Path = QR_CACHE_PATH) -> Path:
    """Generate and save PNG QR code for mobile scanner."""
    output_path.parent.mkdir(parents=True, exist_ok=True)
    uri = make_pairing_uri(host, port, pin)

    qr = qrcode.QRCode(
        version=1,
        error_correction=qrcode.constants.ERROR_CORRECT_M,
        box_size=5,
        border=2,
    )
    qr.add_data(uri)
    qr.make(fit=True)

    img = qr.make_image(fill_color="#121218", back_color="#FFFFFF")
    img.save(str(output_path))
    logger.info("Pairing QR code generated at %s (URI: %s)", output_path, uri)
    return output_path

def get_terminal_qr(host: str, port: int, pin: str) -> str:
    """Return UTF-8 string of ASCII QR code for terminal output."""
    uri = make_pairing_uri(host, port, pin)
    qr = qrcode.QRCode(version=1, box_size=1, border=1)
    qr.add_data(uri)
    qr.make(fit=True)
    out = io.StringIO()
    qr.print_ascii(out=out, invert=True)
    return out.getvalue()
