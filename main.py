import sys
import subprocess
import shutil
import traceback
from pathlib import Path

# --- DEPENDENCY CHECK ---
try:
    from PyQt6.QtWidgets import (QApplication, QWidget, QVBoxLayout, QLabel, 
                                 QComboBox, QPushButton, QLineEdit, QMessageBox, QFrame)
    from PyQt6.QtCore import Qt
    from PyQt6.QtGui import QFont, QIcon
except ImportError:
    print("CRITICAL ERROR: PyQt6 is missing.")
    print("Please install it: pip install PyQt6")
    sys.exit(1)

# --- LOCAL IMPORTS ---
from config import RED, GREEN, CYAN, YELLOW, RESET
from utils import copy_to_clipboard, is_frozen, send_notification
from backends import get_backend

# --- GAME RUNNER LOGIC (HEADLESS) ---
def run_game_mode(backend, monitor_name, game_cmd):
    """
    Runs silently in the background.
    Toggles HDR -> Launches Game -> Waits -> Toggles HDR Off.
    """
    # 1. Enable HDR
    print(f"[HDR WRAPPER] Enabling HDR for: {monitor_name}")
    backend.toggle_hdr(monitor_name, enable=True)
    send_notification("HDR Enabled", f"Monitor: {monitor_name}")

    # 2. Launch Game
    try:
        # Use check_call to wait for the game to finish
        subprocess.check_call(game_cmd)
    except Exception as e:
        print(f"[ERROR] Failed to launch game: {e}")
        send_notification("Game Error", f"Failed to launch: {e}")
    finally:
        # 3. Disable HDR (Runs even if game crashes)
        print(f"[HDR WRAPPER] Game closed. Disabling HDR...")
        backend.toggle_hdr(monitor_name, enable=False)
        send_notification("HDR Disabled", "Returning to SDR mode.")

# --- GUI APPLICATION (GENERATOR) ---
class GeneratorApp(QWidget):
    def __init__(self, backend):
        super().__init__()
        self.backend = backend
        self.monitors = []
        self.init_ui()
        self.load_monitors()

    def init_ui(self):
        self.setWindowTitle("Steam HDR Launcher Generator")
        self.setMinimumWidth(450)
        self.setMinimumHeight(300)

        layout = QVBoxLayout()
        layout.setSpacing(15)
        layout.setContentsMargins(20, 20, 20, 20)

        # Title
        title_lbl = QLabel("Steam HDR Launcher")
        title_font = QFont()
        title_font.setBold(True)
        title_font.setPointSize(14)
        title_lbl.setFont(title_font)
        title_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
        layout.addWidget(title_lbl)

        # Monitor Selection
        layout.addWidget(QLabel("Select your HDR Monitor:"))
        self.combo = QComboBox()
        layout.addWidget(self.combo)

        # Refresh Button
        self.refresh_btn = QPushButton("Rescan Monitors")
        self.refresh_btn.clicked.connect(self.load_monitors)
        layout.addWidget(self.refresh_btn)

        # Separator
        line = QFrame()
        line.setFrameShape(QFrame.Shape.HLine)
        line.setFrameShadow(QFrame.Shadow.Sunken)
        layout.addWidget(line)

        # Output Area
        layout.addWidget(QLabel("Steam Launch Option:"))
        self.output_line = QLineEdit()
        self.output_line.setReadOnly(True)
        self.output_line.setPlaceholderText("Select a monitor to generate command...")
        layout.addWidget(self.output_line)

        # Action Buttons
        btn_layout = QVBoxLayout()
        self.gen_btn = QPushButton("Generate Command")
        self.gen_btn.clicked.connect(self.generate_command)
        self.gen_btn.setStyleSheet("background-color: #2196F3; color: white; font-weight: bold; padding: 8px;")
        
        self.copy_btn = QPushButton("Copy to Clipboard")
        self.copy_btn.clicked.connect(self.copy_command)
        self.copy_btn.setEnabled(False)
        
        btn_layout.addWidget(self.gen_btn)
        btn_layout.addWidget(self.copy_btn)
        layout.addLayout(btn_layout)

        # Status
        self.status_lbl = QLabel("")
        self.status_lbl.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.status_lbl.setStyleSheet("color: #666;")
        layout.addWidget(self.status_lbl)

        self.setLayout(layout)

    def load_monitors(self):
        self.combo.clear()
        self.monitors = self.backend.get_monitors()
        if self.monitors:
            self.combo.addItems(self.monitors)
            self.status_lbl.setText(f"Found {len(self.monitors)} monitors.")
        else:
            self.combo.addItem("No monitors detected")
            self.status_lbl.setText("Error: No monitors found.")

    def generate_command(self):
        if not self.monitors:
            return

        selected_mon = self.combo.currentText()
        
        # Determine path to self
        if is_frozen():
            exe_path = sys.executable
        else:
            # If python script, wrap in python command
            py_exe = sys.executable
            script_path = Path(__file__).absolute()
            exe_path = f'"{py_exe}" "{script_path}"'

        # Generate String
        # Logic: "Executable" "Monitor ID" %command%
        final_cmd = f'"{exe_path}" "{selected_mon}" %command%'
        
        self.output_line.setText(final_cmd)
        self.copy_btn.setEnabled(True)
        self.status_lbl.setText("Command generated!")

    def copy_command(self):
        text = self.output_line.text()
        if text:
            copy_to_clipboard(text)
            self.status_lbl.setText("Copied to clipboard!")
            self.status_lbl.setStyleSheet("color: green; font-weight: bold;")

# --- MAIN ENTRY POINT ---
def main():
    # 1. Initialize Backend
    backend = get_backend()
    if not backend:
        print("Error: Unsupported OS")
        sys.exit(1)

    # 2. Check Arguments
    # If we have arguments, we are in "Steam Mode" (Headless)
    if len(sys.argv) > 1:
        monitor_name = sys.argv[1]
        
        # Sanity Check
        if monitor_name.lower().endswith(".exe") or monitor_name.lower() == "%command%":
            send_notification("HDR Launcher Error", "Incorrect args. Check Steam Launch Options.")
            sys.exit(1)

        if len(sys.argv) < 3:
            send_notification("HDR Launcher Error", "Game command missing!")
            sys.exit(1)

        game_cmd = sys.argv[2:]
        run_game_mode(backend, monitor_name, game_cmd)
        sys.exit(0)

    # 3. If NO arguments, launch GUI (Generator Mode)
    else:
        app = QApplication(sys.argv)
        
        # Check backend dependencies before showing UI
        if not backend.check_dependencies():
            QMessageBox.critical(None, "Missing Dependency", 
                "Could not find the necessary HDR tools (HdrSwitcher or kscreen-doctor).\n\n"
                "Please ensure you are on a supported platform.")
            sys.exit(1)

        window = GeneratorApp(backend)
        window.show()
        sys.exit(app.exec())

if __name__ == "__main__":
    main()