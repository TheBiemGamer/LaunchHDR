mod backends;
mod gui;
mod process; // Import the new module

use clap::Parser;
use std::process::Command;
use std::env;
use backends::get_backend;

#[derive(Parser)]
#[command(name = "HDR Launcher")]
#[command(version = "1.1")]
#[command(about = "Automatically toggles HDR for Steam games")]
struct Cli {
    /// The name of the monitor (e.g., "M27Q", "Display1")
    monitor: String,

    /// Optional: Wait for this specific executable (e.g., "witcher3.exe") instead of the launcher
    #[arg(short, long)]
    wait: Option<String>,

    /// The command to run (use -- to separate args)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    // 1. GUI MODE: Check if we have arguments.
    // If NO arguments are passed, we assume the user double-clicked the .exe.
    if env::args().len() <= 1 {
        // HIDE CONSOLE WINDOW explicitly for GUI mode
        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Console::FreeConsole;
            let _ = FreeConsole();
        }

        if let Err(e) = gui::run() {
            // Re-attach console if GUI fails so we can see error? 
            // Or just log it. simple message box might be better here but eprintln is fallback.
            eprintln!("Failed to run GUI: {}", e);
        }
        return Ok(());
    }

    // 2. CLI MODE: Parse arguments
    // Since we removed "windows_subsystem", the CLI works natively now.
    let cli = Cli::parse();
    
    if cli.command.is_empty() {
        eprintln!("Error: No command provided to launch.");
        return Ok(());
    }

    let backend = get_backend();
    let monitor = &cli.monitor;

    // --- ENABLE HDR ---
    println!("[Launcher] Enabling HDR for '{}'...", monitor);
    backend.notify("HDR Launcher", &format!("Enabling HDR for {}", monitor));
    
    if let Err(e) = backend.toggle_hdr(monitor, true) {
        let err_msg = format!("Failed to enable HDR: {}", e);
        eprintln!("Warning: {}", err_msg);
        backend.notify("HDR Error", &err_msg);
    }

    // --- LAUNCH GAME ---
    let cmd_exe = &cli.command[0];
    let cmd_args = &cli.command[1..];
    
    let mut child = Command::new(cmd_exe)
        .args(cmd_args)
        .spawn()
        .expect("Failed to start game process");

    // --- WAIT STRATEGY ---
    if let Some(target_exe) = cli.wait {
        // Strategy A: User specified a specific game exe to watch (The Witcher 3 Fix)
        // We still wait for the launcher to finish (optional, but good practice),
        // but the REAL wait happens in wait_for_process_lifecycle
        println!("[Launcher] Launcher started. Switching to watch mode for: {}", target_exe);
        process::wait_for_process_lifecycle(&target_exe);
    } else {
        // Strategy B: Standard behavior (Wait for the launched command itself)
        let _ = child.wait();
    }

    // --- DISABLE HDR ---
    println!("[Launcher] Game/Process exited. Disabling HDR...");
    backend.notify("HDR Launcher", "Game closed. Disabling HDR...");
    
    if let Err(e) = backend.toggle_hdr(monitor, false) {
        let err_msg = format!("Failed to disable HDR: {}", e);
        eprintln!("Warning: {}", err_msg);
        backend.notify("HDR Error", &err_msg);
    }

    Ok(())
}