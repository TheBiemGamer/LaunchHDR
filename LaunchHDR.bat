@echo off
setlocal
cd /d "%~dp0"

:: --- SETUP COLORS ---
for /F "tokens=1,2 delims=#" %%a in ('"prompt #$H#$E# & echo on & for %%b in (1) do rem"') do set "ESC=%%b"
set "RED=%ESC%[91m"
set "GREEN=%ESC%[92m"
set "CYAN=%ESC%[96m"
set "YELLOW=%ESC%[93m"
set "RESET=%ESC%[0m"

:: --- CRITICAL CHECK: DOES TOOL EXIST? ---
if not exist "HdrSwitcher.exe" goto :ErrorNoTool

:: --- ENTRY POINT CHECK ---
:: If user ran this directly (no arguments), go to the Interactive Setup Wizard
if "%~1"=="" goto :InteractiveSetup

:: --- VALIDATION CHECKS (GAME MODE) ---

:: Check 1: Did the user forget the Monitor Name? (Is the first arg an .exe?)
if /I "%~x1"==".exe" goto :ErrorIsExe

:: Set Monitor Name
set "MONITOR_NAME=%~1"

:: Check 2: Is the Game Command missing?
if "%~2"=="" goto :ErrorNoGame

:: --- GAME LAUNCH SEQUENCE ---

:: 1. ENABLE HDR
echo %GREEN%[HDR SWITCHER] Enabling HDR for: %CYAN%%MONITOR_NAME%%RESET%
HdrSwitcher.exe enable --name "%MONITOR_NAME%" >nul 2>&1

:: 2. PREPARE GAME (Shift arguments so %2 becomes %1)
shift

:: 3. LAUNCH GAME
echo %GREEN%[HDR SWITCHER] Launching Game: %CYAN%%1%RESET%
start /wait "" %1 %2 %3 %4 %5 %6 %7 %8 %9

:: 4. DISABLE HDR
echo.
echo %GREEN%[HDR SWITCHER] Game closed. Disabling HDR for: %CYAN%%MONITOR_NAME%%RESET%
HdrSwitcher.exe disable --name "%MONITOR_NAME%" >nul 2>&1
exit /b


:: =========================================================
::             INTERACTIVE SETUP WIZARD
:: =========================================================
:InteractiveSetup
cls
echo %YELLOW%===================================================%RESET%
echo %YELLOW%          STEAM LAUNCH COMMAND GENERATOR%RESET%
echo %YELLOW%===================================================%RESET%
echo.
echo It looks like you ran this script directly.
echo.
set /p "CHOICE=Do you want to generate a Steam Launch Command? (Y/N): "
if /I not "%CHOICE%"=="Y" exit /b

echo.
echo %CYAN%Scanning for monitors...%RESET%
echo.

:: Enable Delayed Expansion specifically for the loop
setlocal EnableDelayedExpansion
set "count=0"

:: Parse HdrSwitcher output to find monitor names
for /f "tokens=2 delims=:" %%A in ('HdrSwitcher.exe list ^| findstr /C:"Target name"') do (
    set /a count+=1
    :: This inner loop trims leading spaces from the monitor name
    for /f "tokens=*" %%B in ("%%A") do set "MON_NAME=%%B"
    set "LIST_!count!=!MON_NAME!"
    echo   %GREEN%[!count!]%RESET% !MON_NAME!
)

if !count! EQU 0 (
    echo %RED%[ERROR] No monitors detected!%RESET%
    pause
    exit /b
)

echo.
set /p "SELECTION=Enter the number of your HDR Monitor: "

:: Validate Input
if not defined LIST_%SELECTION% (
    echo.
    echo %RED%Invalid selection.%RESET%
    pause
    endlocal
    goto :InteractiveSetup
)

:: Extract the chosen name and pass it out of the delayed expansion block
set "FINAL_MON=!LIST_%SELECTION%!"
endlocal && set "SELECTED_MON=%FINAL_MON%"

:: Build the command string
set "FINAL_CMD="%~f0" "%SELECTED_MON%" %%command%%"

:: Copy to Clipboard
echo %FINAL_CMD%| clip

cls
echo %YELLOW%===================================================%RESET%
echo %GREEN%              SUCCESS! COPIED TO CLIPBOARD%RESET%
echo %YELLOW%===================================================%RESET%
echo.
echo The command for %CYAN%%SELECTED_MON%%RESET% is now in your clipboard.
echo.
echo %YELLOW%---------------------------------------------------%RESET%
echo %CYAN%%FINAL_CMD%%RESET%
echo %YELLOW%---------------------------------------------------%RESET%
echo.
echo %YELLOW%INSTRUCTIONS:%RESET%
echo 1. Open Steam.
echo 2. Right-click your game and go to %CYAN%Properties%RESET%.
echo 3. Paste (Ctrl+V) this into the %CYAN%Launch Options%RESET% box.
echo.
echo Press any key to exit...
pause >nul
exit /b


:: =========================================================
::                  ERROR HANDLERS
:: =========================================================

:ErrorNoTool
cls
echo %RED%[ERROR] HdrSwitcher.exe not found!%RESET%
echo.
echo Please ensure HdrSwitcher.exe is in:
echo "%~dp0"
pause
exit /b

:ErrorIsExe
cls
echo %RED%[ERROR] It looks like you forgot the monitor name!%RESET%
echo.
echo The script detected an .exe as the first input:
echo %CYAN%"%~1"%RESET%
echo.
echo The script expects a MONITOR NAME first.
goto :ShowHelp

:ErrorNoGame
cls
echo %RED%[ERROR] Game command missing!%RESET%
echo.
echo You specified the monitor (%CYAN%%MONITOR_NAME%%RESET%), but no game.
echo You likely forgot to add %CYAN%%%command%%%RESET% in Steam.
goto :ShowHelp

:ShowHelp
:: Just show the monitor list static help if arguments were botched
echo.
echo %YELLOW%===================================================%RESET%
echo %YELLOW%              HDR LAUNCHER HELP%RESET%
echo %YELLOW%===================================================%RESET%
echo.
echo CORRECT SYNTAX:
echo "%~f0" %CYAN%"YOUR_MONITOR_NAME"%RESET% %%command%%
echo.
echo %YELLOW%---------------------------------------------------%RESET%
echo %YELLOW%YOUR AVAILABLE MONITORS:%RESET%
echo %CYAN%
HdrSwitcher.exe list
echo %RESET%
echo %YELLOW%---------------------------------------------------%RESET%
pause >nul
exit /b