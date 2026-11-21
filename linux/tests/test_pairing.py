"""Tests for device pairing and authentication."""
from pathlib import Path
from clink.devices.pairing import PairingManager

def test_pairing_workflow(tmp_path: Path):
    mgr = PairingManager(devices_file=tmp_path / "test_devices.json")
    pin = mgr.generate_pin()
    assert len(pin) == 6
    assert pin.isdigit()

    # Wrong PIN
    assert mgr.pair_device("dev_1", "Test Phone", "000000" if pin != "000000" else "111111") is None

    # Re-generate PIN and succeed
    pin2 = mgr.generate_pin()
    token = mgr.pair_device("dev_1", "Test Phone", pin2)
    assert token is not None
    assert len(token) == 32

    # Authenticate token
    assert mgr.authenticate_token(token) is True
    assert mgr.authenticate_token("invalid_token") is False

    # List devices
    devs = mgr.list_devices()
    assert "dev_1" in devs
    assert devs["dev_1"]["device_name"] == "Test Phone"

    # Remove device
    assert mgr.remove_device("dev_1") is True
    assert mgr.authenticate_token(token) is False
