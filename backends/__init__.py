import platform
import os
from config import YELLOW, RED, RESET
from .windows import WindowsHDRBackend
from .kde import KDEHDRBackend

def get_backend():
    system = platform.system()
    if system == "Windows":
        return WindowsHDRBackend()
    elif system == "Linux":
        # Check for KDE session
        desktop = os.environ.get("XDG_CURRENT_DESKTOP", "").lower()
        if "kde" in desktop:
            return KDEHDRBackend()
        else:
            print(f"{YELLOW}[WARN] Linux detected but not KDE. Attempting KDE backend anyway...{RESET}")
            return KDEHDRBackend()
    else:
        print(f"{RED}[ERROR] Unsupported OS: {system}{RESET}")
        return None