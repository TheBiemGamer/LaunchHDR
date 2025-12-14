import sys
import os
import subprocess
import time
from pathlib import Path

# --- CONFIGURATION & COLORS ---
RED = "\033[91m"
GREEN = "\033[92m"
CYAN = "\033[96m"
YELLOW = "\033[93m"
RESET = "\033[0m"

# Enable colors in Windows CMD
os.system('')

def is_frozen():
    """Checks if the script is running as a compiled PyInstaller executable."""
    return getattr(sys, 'frozen', False)

def get_app_path():
    """
    Returns the permanent path to the executable or script.
    Used for generating the Steam Launch Command.
    """
    if is_frozen():
        return Path(sys.executable).parent
    else:
        return Path(__file__).parent.absolute()

def find_hdr_switcher():
    """
    Locates HdrSwitcher.exe.
    Priority 1: Bundled inside the Exe (sys._MEIPASS)
    Priority 2: External 'bin' folder
    Priority 3: External root folder
    """
    
    # 1. CHECK INTERNAL TEMP FOLDER (PyInstaller Bundle)
    if is_frozen() and hasattr(sys, '_MEIPASS'):
        temp_dir = Path(sys._MEIPASS)
        
        # Check if packed in root of temp
        internal_root = temp_dir / "HdrSwitcher.exe"
        if internal_root.exists():
            return str(internal_root)
            
        # Check if packed in bin/ folder of temp
        internal_bin = temp_dir / "bin" / "HdrSwitcher.exe"
        if internal_bin.exists():
            return str(internal_bin)

    # 2. CHECK EXTERNAL FOLDERS (Fallback)
    base_dir = get_app_path()
    
    external_bin = base_dir / "bin" / "HdrSwitcher.exe"
    if external_bin.exists():
        return str(external_bin)

    external_root = base_dir / "HdrSwitcher.exe"
    if external_root.exists():
        return str(external_root)

    return None

def toggle_hdr(tool_path, action, monitor_name):
    """Runs the enable/disable command silently."""
    try:
        subprocess.run(
            [tool_path, action, "--name", monitor_name],
            capture_output=True,
            text=True,
            check=False
        )
    except Exception as e:
        print(f"{RED}[ERROR] Failed to switch HDR: {e}{RESET}")

def get_monitors(tool_path):
    """Parses 'list' command to return a list of monitor names."""
    try:
        result = subprocess.run(
            [tool_path, "list"],
            capture_output=True,
            text=True,
            encoding='utf-8'
        )
        
        monitors = []
        for line in result.stdout.splitlines():
            if "Target name" in line:
                parts = line.split(":", 1)
                if len(parts) > 1:
                    monitors.append(parts[1].strip())
        return monitors
    except Exception as e:
        print(f"{RED}[ERROR] Could not scan monitors: {e}{RESET}")
        return []

def copy_to_clipboard(text):
    cmd = f'echo {text.strip()}| clip'
    subprocess.check_call(cmd, shell=True)

def interactive_wizard(tool_path):
    print(f"{YELLOW}==================================================={RESET}")
    print(f"{YELLOW}          STEAM LAUNCH COMMAND GENERATOR{RESET}")
    print(f"{YELLOW}==================================================={RESET}")
    print("\nIt looks like you ran this tool directly.")
    
    choice = input("Do you want to generate a Steam Launch Command? (y/n): ").lower()
    if choice != 'y':
        return

    print(f"\n{CYAN}Scanning for monitors...{RESET}")
    monitors = get_monitors(tool_path)

    if not monitors:
        print(f"{RED}[ERROR] No monitors detected.{RESET}")
        input("Press Enter to exit...")
        return

    for i, mon in enumerate(monitors, 1):
        print(f"  {GREEN}[{i}]{RESET} {mon}")

    print()
    try:
        sel = int(input("Enter the number of your HDR Monitor: "))
        if sel < 1 or sel > len(monitors):
            raise ValueError
        
        selected_mon = monitors[sel-1]
        
        # --- COMMAND GENERATION LOGIC ---
        if is_frozen():
            # IMPORTANT: Use sys.executable to point to the permanent .exe on disk
            # Do NOT use sys._MEIPASS here, or the shortcut will break when the app closes.
            exe_path = sys.executable
            final_cmd = f'"{exe_path}" "{selected_mon}" %command%'
        else:
            py_exe = sys.executable
            script_path = Path(__file__).absolute()
            final_cmd = f'"{py_exe}" "{script_path}" "{selected_mon}" %command%'
        
        copy_to_clipboard(final_cmd)
        
        print(f"\n{YELLOW}==================================================={RESET}")
        print(f"{GREEN}              SUCCESS! COPIED TO CLIPBOARD{RESET}")
        print(f"{YELLOW}==================================================={RESET}")
        print(f"\nCommand for {CYAN}{selected_mon}{RESET} is in your clipboard:\n")
        print(f"{CYAN}{final_cmd}{RESET}\n")
        print(f"{YELLOW}INSTRUCTIONS:{RESET}")
        print(f"1. Open Steam -> Right-click Game -> Properties.")
        print(f"2. Paste into {CYAN}Launch Options{RESET}.")
        
    except ValueError:
        print(f"{RED}Invalid selection.{RESET}")
    
    input("\nPress Enter to exit...")

def main():
    tool_path = find_hdr_switcher()
    
    if not tool_path:
        print(f"{RED}[ERROR] HdrSwitcher.exe not found!{RESET}")
        print(f"Checked internal bundle and external folders.")
        input("Press Enter to exit...")
        sys.exit(1)

    # --- ARGUMENT HANDLING ---
    
    # Check if run with no args (Wizard Mode)
    if len(sys.argv) < 2:
        interactive_wizard(tool_path)
        sys.exit(0)

    monitor_name = sys.argv[1]
    
    # Validation 1: User likely forgot monitor name
    if monitor_name.lower().endswith(".exe"):
        print(f"{RED}[ERROR] It looks like you forgot the monitor name!{RESET}")
        print(f"Steam passed the game executable as the first argument: {CYAN}{monitor_name}{RESET}")
        sys.exit(1)

    # Validation 2: User likely forgot %command%
    if len(sys.argv) < 3:
        print(f"{RED}[ERROR] Game command missing!{RESET}")
        print(f"Monitor: {CYAN}{monitor_name}{RESET}")
        print(f"But no game was passed. Did you forget {CYAN}%command%{RESET} in Steam?")
        sys.exit(1)

    # --- GAME LAUNCH SEQUENCE ---
    
    # 1. Enable HDR
    print(f"{GREEN}[HDR SWITCHER] Enabling HDR for: {CYAN}{monitor_name}{RESET}")
    toggle_hdr(tool_path, "enable", monitor_name)

    # 2. Launch Game (sys.argv[2:] is the game and its flags)
    game_cmd = sys.argv[2:]
    print(f"{GREEN}[HDR SWITCHER] Launching Game: {CYAN}{game_cmd[0]}{RESET}")
    
    try:
        subprocess.call(game_cmd)
    except Exception as e:
        print(f"{RED}[ERROR] Failed to launch game: {e}{RESET}")

    # 3. Disable HDR
    print(f"\n{GREEN}[HDR SWITCHER] Game closed. Disabling HDR for: {CYAN}{monitor_name}{RESET}")
    toggle_hdr(tool_path, "disable", monitor_name)

if __name__ == "__main__":
    main()