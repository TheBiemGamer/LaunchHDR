@echo off
REM Python and PyInstaller are needed to run this build script

REM Ensure the script operates in the folder where this file is located
cd /d "%~dp0"

REM Run PyInstaller with relative paths
pyinstaller --noconfirm --onefile --console --name "LaunchHDR" --add-data "bin\HdrSwitcher.exe;." "LaunchHDR.py"

pause