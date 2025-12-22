# LaunchHDR

**LaunchHDR** is a lightweight, native utility designed to automatically toggle **Windows HDR** ON when you launch a game and OFF when you close it. 

It supports **Steam** (via Launch Options) and **Epic Games/GOG/Standalone** titles (via a Wrapper mode), providing a "set and forget" HDR experience.

![License](https://img.shields.io/github/license/TheBiemGamer/LaunchHDR)
![Version](https://img.shields.io/github/v/release/TheBiemGamer/LaunchHDR)

## Installation

1.  Download the latest installer (`LaunchHDR_Setup.exe`) from the [**Releases Page**](https://github.com/TheBiemGamer/LaunchHDR/releases).
2.  Run the installer.
3.  Launch **LaunchHDR** from your Start Menu or Desktop.

## Usage

### 1. Steam Games
For games launched via Steam:

1.  Open **LaunchHDR**.
2.  Select your **HDR Monitor** from the dropdown.
3.  (Optional) If the game uses a launcher (e.g., Cyberpunk 2077, Witcher 3), use the **Wait Process** picker to select the actual game executable.
4.  Click **"Generate Command"** and then **"Copy"**.
5.  In Steam, right-click your game -> **Properties** -> **General** -> **Launch Options**.
6.  Paste the command (e.g., `"C:\...\LaunchHDR.exe" "MonitorName" -- %command%`).

### 2. Epic Games / GOG / Standalone
For games where you cannot add launch arguments:

1.  Open **LaunchHDR** and switch to the **Epic/Standalone** tab.
2.  Click **"Browse Game Executable..."** and select the game's `.exe` file.
3.  Click **"Install Wrapper"**.
    *   *This will rename the original game file (e.g., `Game.exe` -> `Game_Original.exe`) and replace it with LaunchHDR.*
4.  Launch the game normally through Epic/GOG. LaunchHDR will toggle HDR, then launch the original game.

*To remove it, simply click **"Uninstall Wrapper"** in the same menu.*

## Building from Source

**Prerequisites:**
*   [Rust Toolchain](https://rustup.rs/) (stable)
*   [Inno Setup](https://jrsoftware.org/isinfo.php) (optional, for installer)

**Build:**
```powershell
# Clone the repository
git clone https://github.com/TheBiemGamer/LaunchHDR
cd LaunchHDR

# Build release binary
cargo build --release
```

The executable will be located in `target/release/LaunchHDR.exe`.

## License

This project is licensed under the **MIT License**.