"""Device pairing and token authentication manager."""
from __future__ import annotations
import json
import logging
import os
from pathlib import Path
import secrets
from typing import Dict, Optional

logger = logging.getLogger(__name__)

CONFIG_DIR = Path.home() / ".config" / "clink"
DEVICES_FILE = CONFIG_DIR / "devices.json"
PIN_FILE = CONFIG_DIR / "pin.txt"

class PairingManager:
    def __init__(self, devices_file: Path = DEVICES_FILE, pin_file: Path = PIN_FILE):
        self.devices_file = devices_file
        self.pin_file = pin_file
        self.devices: Dict[str, Dict] = {}
        self.active_pin: Optional[str] = None
        self._load()

    def _load(self) -> None:
        if self.devices_file.exists():
            try:
                with open(self.devices_file, "r", encoding="utf-8") as f:
                    self.devices = json.load(f)
            except Exception as e:
                logger.warning("Could not load paired devices file: %s", e)
                self.devices = {}
        else:
            self.devices = {}

        if self.pin_file.exists():
            try:
                saved_pin = self.pin_file.read_text(encoding="utf-8").strip()
                if saved_pin:
                    self.active_pin = saved_pin
            except Exception:
                pass

    def _save(self) -> None:
        self.devices_file.parent.mkdir(parents=True, exist_ok=True)
        try:
            with open(self.devices_file, "w", encoding="utf-8") as f:
                json.dump(self.devices, f, indent=2)
        except Exception as e:
            logger.error("Failed to save paired devices: %s", e)

    def generate_pin(self) -> str:
        """Generate a 6-digit numeric pairing PIN and persist it for clients."""
        self.active_pin = f"{secrets.randbelow(1000000):06d}"
        self.pin_file.parent.mkdir(parents=True, exist_ok=True)
        try:
            self.pin_file.write_text(self.active_pin, encoding="utf-8")
        except Exception as e:
            logger.warning("Could not write PIN file: %s", e)
        logger.info("New pairing PIN generated: %s", self.active_pin)
        return self.active_pin

    def get_current_pin(self) -> str:
        if not self.active_pin:
            return self.generate_pin()
        return self.active_pin

    def verify_pin(self, pin: str) -> bool:
        candidate = pin.strip()
        current = self.active_pin
        if not current and self.pin_file.exists():
            try:
                current = self.pin_file.read_text(encoding="utf-8").strip()
                self.active_pin = current
            except Exception:
                pass
        if current and secrets.compare_digest(current, candidate):
            return True
        return False

    def pair_device(self, device_id: str, device_name: str, pin: str) -> Optional[str]:
        """Pair a device if PIN matches, generating and saving a persistent token."""
        if not self.verify_pin(pin):
            logger.warning("PIN verification failed for device '%s' (provided '%s')", device_name, pin)
            return None
        token = secrets.token_hex(16)
        self.devices[device_id] = {
            "device_name": device_name,
            "token": token,
            "paired_at": int(secrets.randbelow(100000))
        }
        self._save()
        logger.info("Successfully paired device '%s' (%s)", device_name, device_id)
        return token

    def authenticate_token(self, token: str) -> bool:
        if not token:
            return False
        for dev in self.devices.values():
            if secrets.compare_digest(dev.get("token", ""), token):
                return True
        return False

    def list_devices(self) -> Dict[str, Dict]:
        self._load()
        return self.devices

    def remove_device(self, device_id: str) -> bool:
        if device_id in self.devices:
            del self.devices[device_id]
            self._save()
            return True
        return False
