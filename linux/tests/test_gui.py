"""Tests for Linux Control Center GUI."""
import os
import pytest
from clink.config.settings import ClinkConfig
from clink.devices.pairing import PairingManager

def test_gui_imports_and_components():
    from clink.gui.control_center import get_lan_ip
    ip = get_lan_ip()
    assert isinstance(ip, str)
    assert len(ip) >= 7

def test_pairing_manager_shared_pin(tmp_path):
    pin_file = tmp_path / "pin.txt"
    dev_file = tmp_path / "devices.json"
    mgr1 = PairingManager(devices_file=dev_file, pin_file=pin_file)
    pin = mgr1.generate_pin()

    # Another instance (like GUI reading CLI PIN) reads the same PIN
    mgr2 = PairingManager(devices_file=dev_file, pin_file=pin_file)
    assert mgr2.get_current_pin() == pin
    assert mgr2.verify_pin(pin) is True
