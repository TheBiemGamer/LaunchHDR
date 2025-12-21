import sys
import subprocess
import platform
import shutil
from pathlib import Path
from config import YELLOW, RESET

def is_frozen():
    """Checks if the script is running as a compiled PyInstaller executable."""
    return getattr(sys, 'frozen', False)

def get_app_path():
    """Returns the permanent path to the executable or script."""
    if is_frozen():
        return Path(sys.executable).parent
    else:
        return Path(__file__).parent.absolute()

def copy_to_clipboard(text):
    """Cross-platform clipboard copy."""
    text = text.strip()
    system = platform.system()
    try:
        if system == "Windows":
            cmd = f'echo {text}| clip'
            subprocess.check_call(cmd, shell=True)
            return True
        elif system == "Linux":
            # Try wl-copy (Wayland) first, then xclip (X11)
            if shutil.which("wl-copy"):
                subprocess.run(["wl-copy"], input=text.encode(), check=True)
                return True
            elif shutil.which("xclip"):
                subprocess.run(["xclip", "-selection", "clipboard"], input=text.encode(), check=True)
                return True
    except Exception as e:
        print(f"{YELLOW}[WARN] Could not copy to clipboard: {e}{RESET}")
    return False

def send_notification(title, message):
    """Sends a desktop notification."""
    if platform.system() == "Linux":
        if shutil.which("notify-send"):
            try:
                subprocess.run(["notify-send", "-a", "HDR Launcher", title, message])
            except Exception:
                pass