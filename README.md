# Steam HDR Launcher

A simple, modular batch script to automatically toggle HDR on when a game launches and off when it closes. Designed for use with Steam Launch Options.

## How It Works
1. When you launch a game, the script enables HDR for your specific monitor.
2. It launches the game and waits in the background.
3. When you quit the game, it automatically switches HDR back to SDR.

## Installation

1. Download or Clone this repository to a permanent folder (e.g., `C:\GamingTools\`).
2. That's it!

## How to Use

### 1. Find Your Monitor Name
Run `LaunchHDR.bat` directly from the folder. It will detect that you are missing arguments and display a list of your connected monitors. Copy the exact name of your HDR monitor.

### 2. Set Steam Launch Options
Right-click your game in Steam -> **Properties** -> **General** -> **Launch Options**.

Enter the following command:

```text
"C:\GamingTools\LaunchHDR.bat" "YOUR_MONITOR_NAME" %command%