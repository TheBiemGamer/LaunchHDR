import sys
import subprocess
from pathlib import Path
from .base import HDRBackend
from utils import get_app_path, is_frozen
from config import RED, RESET

class WindowsHDRBackend(HDRBackend):
    def __init__(self):
        self.tool_path = self._find_hdr_switcher()

    def _find_hdr_switcher(self):
        """Locates HdrSwitcher.exe in temp (frozen) or local folders."""
        # 1. Check internal temp (PyInstaller)
        if is_frozen() and hasattr(sys, '_MEIPASS'):
            temp_dir = Path(sys._MEIPASS)
            paths = [temp_dir / "HdrSwitcher.exe", temp_dir / "bin" / "HdrSwitcher.exe"]
            for p in paths:
                if p.exists(): return str(p)

        # 2. Check external folders
        base_dir = get_app_path()
        paths = [base_dir / "bin" / "HdrSwitcher.exe", base_dir / "HdrSwitcher.exe"]
        for p in paths:
            if p.exists(): return str(p)
        return None

    def check_dependencies(self):
        if not self.tool_path:
            print(f"{RED}[ERROR] HdrSwitcher.exe not found!{RESET}")
            return False
        return True

    def get_monitors(self):
        try:
            result = subprocess.run(
                [self.tool_path, "list"],
                capture_output=True, text=True, encoding='utf-8'
            )
            monitors = []
            for line in result.stdout.splitlines():
                if "Target name" in line:
                    parts = line.split(":", 1)
                    if len(parts) > 1:
                        monitors.append(parts[1].strip())
            return monitors
        except Exception as e:
            print(f"{RED}[ERROR] Scan failed: {e}{RESET}")
            return []

    def toggle_hdr(self, monitor_id, enable: bool):
        action = "enable" if enable else "disable"
        try:
            subprocess.run(
                [self.tool_path, action, "--name", monitor_id],
                capture_output=True, text=True
            )
        except Exception as e:
            print(f"{RED}[ERROR] Failed to switch HDR: {e}{RESET}")