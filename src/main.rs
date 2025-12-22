#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Keep windows subsystem, but we will attach console if needed

mod backends;
mod gui;
mod process;

use clap::Parser;
use std::process::Command;
use std::env;
use std::path::Path;
use std::fs;
use backends::get_backend;

// --- CONFIG STRUCT FOR WRAPPER MODE ---
struct WrapperConfig {
    monitor: String,
    target: String,
    wait: Option<String>,
}

fn load_wrapper_config() -> Option<WrapperConfig> {
    if !Path::new("hdr_config.ini").exists() {
        return None;
    }

    let content = fs::read_to_string("hdr_config.ini").ok()?;
    let mut monitor = String::new();
    let mut target = String::new();
    let mut wait = None;

    for line in content.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "monitor" => monitor = value.trim().to_string(),
                "target" => target = value.trim().to_string(),
                "wait" => {
                    let w = value.trim();
                    if !w.is_empty() {
                        wait = Some(w.to_string());
                    }
                },
                _ => {}
            }
        }
    }

    if monitor.is_empty() || target.is_empty() {
        return None;
    }

    Some(WrapperConfig { monitor, target, wait })
}

#[derive(Parser)]
#[command(name = "HDR Launcher")]
#[command(version = "1.2")]
#[command(about = "Automatically toggles HDR for Steam/Epic games")]
struct Cli {
    monitor: String,
    #[arg(short, long)]
    wait: Option<String>,
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    command: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    // 1. WRAPPER MODE CHECK (Priority 1)
    // If hdr_config.ini exists, we assume we are impersonating a game executable.
    // We ignore CLI args for ourself, but pass them ALL to the target.
    if let Some(config) = load_wrapper_config() {
        // We might want to attach console to log errors to a file or stdout if possible,
        // but for a game wrapper, silent failure (or message box) is usually preferred.
        // We'll try to log to stdout if attached.
        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }

        return run_wrapper_session(config);
    }

    // 2. CLI MODE SETUP (Priority 2)
    // If explicit args are passed (and no config), we might be in Steam mode.
    if env::args().len() > 1 {
        #[cfg(windows)]
        unsafe {
            use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
            let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }

    // 3. GUI MODE (Priority 3)
    if env::args().len() <= 1 {
        if let Err(e) = gui::run() {
            eprintln!("Failed to run GUI: {}", e);
        }
        return Ok(());
    }

    // 4. RUN STANDARD CLI
    let cli = Cli::parse();
    if cli.command.is_empty() {
        eprintln!("Error: No command provided.");
        return Ok(());
    }

    run_session(&cli.monitor, &cli.command[0], &cli.command[1..], cli.wait)
}

// --- LOGIC FOR WRAPPER (EPIC) ---
fn run_wrapper_session(config: WrapperConfig) -> anyhow::Result<()> {
    // Pass ALL arguments received by this process to the target game
    // skip(1) removes the executable path itself
    let args: Vec<String> = env::args().skip(1).collect();
    let args_slices: Vec<&String> = args.iter().collect(); // Convert to format needed by run_session logic

    // Convert Vec<&String> to &[&String] is tricky, so we just reconstruct logic here roughly
    // simpler to just call the shared logic
    
    // We need to pass args as slice of strings
    // The Command::args takes an iterator of strings
    
    let backend = get_backend();
    
    // Enable HDR
    backend.notify("HDR Launcher", &format!("Wrapper: Enabling HDR for {}", config.monitor));
    let _ = backend.toggle_hdr(&config.monitor, true);

    // Launch Target
    println!("Launching target: {} with args: {:?}", config.target, args);
    
    let mut child = Command::new(&config.target)
        .args(&args)
        .spawn();

    match child {
        Ok(mut child) => {
            // Wait Logic
            if let Some(wait_target) = config.wait {
                process::wait_for_process_lifecycle(&wait_target);
            } else {
                let _ = child.wait();
            }
        },
        Err(e) => {
             backend.notify("HDR Error", &format!("Failed to launch target '{}': {}", config.target, e));
        }
    }

    // Disable HDR
    backend.notify("HDR Launcher", "Game closed. Disabling HDR...");
    let _ = backend.toggle_hdr(&config.monitor, false);

    Ok(())
}

// --- LOGIC FOR STANDARD CLI (STEAM) ---
fn run_session(monitor: &str, exe: &str, args: &[String], wait_target: Option<String>) -> anyhow::Result<()> {
    let backend = get_backend();
    
    println!("[Launcher] Enabling HDR for '{}'...", monitor);
    backend.notify("HDR Launcher", &format!("Enabling HDR for {}", monitor));
    
    if let Err(e) = backend.toggle_hdr(monitor, true) {
        let err_msg = format!("Failed to enable HDR: {}", e);
        eprintln!("Warning: {}", err_msg);
        backend.notify("HDR Error", &err_msg);
    }

    let mut child = Command::new(exe)
        .args(args)
        .spawn()
        .expect("Failed to start game process");

    if let Some(target) = wait_target {
        process::wait_for_process_lifecycle(&target);
    } else {
        let _ = child.wait();
    }

    println!("[Launcher] Game exited. Disabling HDR...");
    backend.notify("HDR Launcher", "Game closed. Disabling HDR...");
    
    if let Err(e) = backend.toggle_hdr(monitor, false) {
        eprintln!("Warning: Failed to disable HDR: {}", e);
    }

    Ok(())
}