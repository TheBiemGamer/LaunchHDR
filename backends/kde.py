import shutil
import subprocess
import json
from .base import HDRBackend
from config import RED, CYAN, RESET

class KDEHDRBackend(HDRBackend):
    def __init__(self):
        self.cmd = "kscreen-doctor"

    def check_dependencies(self):
        if not shutil.which(self.cmd):
            print(f"{RED}[ERROR] '{self.cmd}' not found. Are you on KDE Plasma?{RESET}")
            return False
        return True

    def get_monitors(self):
        # METHOD 1: Try JSON output (More robust on modern Plasma)
        try:
            result = subprocess.run([self.cmd, "-j"], capture_output=True, text=True)
            if result.returncode == 0 and result.stdout.strip():
                try:
                    data = json.loads(result.stdout)
                    monitors = []
                    for output in data.get("outputs", []):
                        if output.get("connected"):
                            name = output.get("name")
                            if name:
                                monitors.append(name)
                    if monitors:
                        return monitors
                except json.JSONDecodeError:
                    pass 
        except Exception:
            pass

        # METHOD 2: Fallback to Text Parsing
        try:
            result = subprocess.run([self.cmd, "--outputs"], capture_output=True, text=True)
            monitors = []
            for line in result.stdout.splitlines():
                line = line.strip()
                if line.startswith("Output:"):
                    parts = line.split()
                    if len(parts) >= 3:
                        monitors.append(parts[2])
            return monitors
        except Exception as e:
            print(f"{RED}[ERROR] Failed to query kscreen-doctor: {e}{RESET}")
            return []

    def toggle_hdr(self, monitor_id, enable: bool):
        state = "enable" if enable else "disable"
        arg = f"output.{monitor_id}.hdr.{state}"
        wcg_arg = f"output.{monitor_id}.wcg.{state}"
        
        print(f"{CYAN}Running: {self.cmd} {arg} {wcg_arg}{RESET}")
        try:
            subprocess.run([self.cmd, arg, wcg_arg], capture_output=True)
        except Exception as e:
            print(f"{RED}[ERROR] Failed to switch HDR on KDE: {e}{RESET}")