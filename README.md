# Steam HDR Launcher

A simple tool that automatically toggles HDR **ON** when you launch a game and **OFF** when you close it.

Designed for use with **Steam Launch Options**, this tool eliminates the need to manually toggle Windows settings before playing HDR-supported games.

## Installation

1.  Go to the **[suspicious link removed]** page.
2.  Download the latest `.exe` file.
3.  Save it to a permanent folder (e.g., `C:\GamingTools\`).

## How to Use

### Method 1: The Easy Way (Setup Wizard)

The tool includes a built-in wizard that writes the launch code for you.

1.  Double-click **`LaunchHDR.exe`**.
2.  Type `y` to start the wizard.
3.  Select your HDR monitor from the list (e.g., type `1`).
4.  The tool will **automatically copy the correct Launch Option** to your clipboard.
5.  Paste it into Steam.

### Method 2: The Manual Way

If you prefer to type it yourself, the syntax for Steam Launch Options is:

```text
"Path\To\LaunchHDR.exe" "YOUR_MONITOR_NAME" %command%
```

**Example:**

> `"C:\GamingTools\LaunchHDR.exe" "34M2C6500" %command%`

-----

## Troubleshooting

  * **HDR doesn't turn on**:
      * Ensure "Use HDR" is allowed in your Windows Display Settings.
      * Check that your monitor name in the command matches exactly (use the Wizard to be sure).
  * **Game doesn't launch**:
      * Make sure you included `%command%` at the end of the line in Steam.

## Credits & License

**License:**
This project is licensed under the **MIT License**. See [LICENSE](https://www.google.com/search?q=LICENSE) for details.

**Credits:**
The HDR switching logic utilizes the **HdrSwitcher** tool created by [Vaiz](https://github.com/Vaiz/HdrSwitcher/).