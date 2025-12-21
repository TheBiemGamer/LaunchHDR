import PyInstaller.__main__
import platform
import os
import shutil
from pathlib import Path

# --- CONFIGURATION ---
ENTRY_POINT = "main.py"
OUTPUT_NAME = "LaunchHDR"
WINDOWS_BINARY = Path("bin") / "HdrSwitcher.exe"

def build():
    system = platform.system()
    print(f"Detected Platform: {system}")

    # clean previous builds
    if os.path.exists("build"):
        shutil.rmtree("build")
    if os.path.exists("dist"):
        shutil.rmtree("dist")

    args = [
        ENTRY_POINT,
        f'--name={OUTPUT_NAME}',
        '--onefile',
        '--windowed',
        '--noconfirm',
        '--clean',
        '--collect-all=PyQt6', # <--- ADDED: Forces PyInstaller to include all Qt plugins (Wayland/XCB)
    ]

    # --- OS SPECIFIC CONFIGURATION ---
    if system == "Windows":
        if not WINDOWS_BINARY.exists():
            print(f"Error: Could not find {WINDOWS_BINARY}")
            print("Please ensure the 'bin' folder exists with HdrSwitcher.exe inside.")
            return

        # On Windows, separator is ';'
        args.append(f'--add-data={WINDOWS_BINARY};.')
    
    elif system == "Linux":
        # On Linux, no extra binaries needed usually
        pass

    # --- RUN BUILD ---
    print(f"Running PyInstaller with args: {args}")
    PyInstaller.__main__.run(args)
    
    print("\n" + "="*30)
    print(f"Build Complete! Executable is in /dist/")
    print("="*30)

if __name__ == "__main__":
    build()