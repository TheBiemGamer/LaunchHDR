use anyhow::Result;

pub trait HdrBackend {
    /// scan for available monitors
    fn get_monitors(&self) -> Result<Vec<String>>;
    
    /// toggle HDR for a specific monitor
    fn toggle_hdr(&self, monitor_name: &str, enable: bool) -> Result<()>;

    /// Send a desktop notification
    fn notify(&self, title: &str, message: &str);
}

// Module declarations
#[cfg(windows)]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;

// Factory to get the correct backend for the current OS
pub fn get_backend() -> Box<dyn HdrBackend> {
    #[cfg(windows)]
    {
        Box::new(windows::WindowsBackend)
    }
    #[cfg(target_os = "linux")]
    {
        Box::new(linux::LinuxBackend)
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        panic!("Unsupported OS");
    }
}