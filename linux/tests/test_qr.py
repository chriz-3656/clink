"""Tests for QR code generation."""
from pathlib import Path
from clink.devices.qr_manager import make_pairing_uri, generate_pairing_qr_image, get_terminal_qr

def test_qr_uri_creation():
    uri = make_pairing_uri("192.168.1.50", 8765, "123456")
    assert uri == "clink://pair?host=192.168.1.50&port=8765&pin=123456"

def test_qr_image_generation(tmp_path: Path):
    out = tmp_path / "test_qr.png"
    res = generate_pairing_qr_image("192.168.1.50", 8765, "123456", output_path=out)
    assert res.exists()
    assert res.stat().st_size > 0

def test_qr_terminal_ascii():
    ascii_qr = get_terminal_qr("192.168.1.50", 8765, "123456")
    assert isinstance(ascii_qr, str)
    assert len(ascii_qr) > 50
