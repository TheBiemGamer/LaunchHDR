use std::time::{Duration, Instant};
use std::thread;

#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;

#[cfg(windows)]
pub fn is_process_running(target_name: &str) -> bool {
    unsafe {
        // CreateToolhelp32Snapshot returns Result<HANDLE, Error>
        // We match on it to get the handle or return false on failure.
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(handle) => handle,
            Err(_) => return false,
        };

        let mut entry = PROCESSENTRY32W::default();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                // szExeFile is [u16; 260] in PROCESSENTRY32W
                let name_slice = &entry.szExeFile;
                let len = name_slice.iter().position(|&c| c == 0).unwrap_or(name_slice.len());
                let process_name = String::from_utf16_lossy(&name_slice[0..len]);

                if process_name.eq_ignore_ascii_case(target_name) {
                    let _ = CloseHandle(snapshot);
                    return true;
                }

                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
    false
}

#[cfg(not(windows))]
pub fn is_process_running(_target_name: &str) -> bool {
    // Linux implementation placeholder
    false 
}

/// Waits for a process to START, then waits for it to STOP.
pub fn wait_for_process_lifecycle(target_name: &str) {
    println!("[Launcher] Waiting for process '{}' to start...", target_name);
    
    // 1. Wait for it to appear (Timeout: 60 seconds)
    let start_wait = Instant::now();
    let mut found = false;
    
    while start_wait.elapsed() < Duration::from_secs(60) {
        if is_process_running(target_name) {
            found = true;
            break;
        }
        thread::sleep(Duration::from_secs(1));
    }

    if !found {
        println!("[Launcher] Timed out: '{}' did not start.", target_name);
        return;
    }

    println!("[Launcher] '{}' is running. Monitoring...", target_name);

    // 2. Wait for it to disappear
    loop {
        if !is_process_running(target_name) {
            println!("[Launcher] '{}' has closed.", target_name);
            break;
        }
        thread::sleep(Duration::from_secs(2));
    }
}