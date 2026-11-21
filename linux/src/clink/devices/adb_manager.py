"""ADB connection manager for USB reverse port forwarding."""
from __future__ import annotations
import logging
import shutil
import subprocess
from typing import List, Dict

logger = logging.getLogger(__name__)

class AdbManager:
    def __init__(self, port: int = 8765):
        self.port = port
        self.adb_bin = shutil.which("adb")

    def is_adb_available(self) -> bool:
        return self.adb_bin is not None

    def list_connected_devices(self) -> List[Dict[str, str]]:
        if not self.is_adb_available():
            return []
        try:
            res = subprocess.run([self.adb_bin, "devices", "-l"], capture_output=True, text=True, check=True)
            devices = []
            for line in res.stdout.strip().splitlines()[1:]:
                parts = line.strip().split()
                if len(parts) >= 2 and parts[1] == "device":
                    model = "Unknown"
                    for p in parts[2:]:
                        if p.startswith("model:"):
                            model = p.split(":", 1)[1]
                    devices.append({"serial": parts[0], "status": parts[1], "model": model})
            return devices
        except Exception as e:
            logger.error("Failed to list ADB devices: %s", e)
            return []

    def setup_reverse_port(self) -> bool:
        """Run adb reverse tcp:<port> tcp:<port> for connected devices."""
        if not self.is_adb_available():
            logger.warning("ADB executable not found in PATH")
            return False
        try:
            cmd = [self.adb_bin, "reverse", f"tcp:{self.port}", f"tcp:{self.port}"]
            subprocess.run(cmd, capture_output=True, text=True, check=True)
            logger.info("Configured ADB reverse forwarding for port %d", self.port)
            return True
        except subprocess.CalledProcessError as e:
            logger.warning("Failed to configure adb reverse: %s (%s)", e.stderr.strip(), e)
            return False
        except Exception as e:
            logger.error("Unexpected error configuring adb reverse: %s", e)
            return False

    def remove_reverse_port(self) -> bool:
        if not self.is_adb_available():
            return False
        try:
            subprocess.run([self.adb_bin, "reverse", "--remove", f"tcp:{self.port}"], capture_output=True, text=True)
            return True
        except Exception:
            return False
