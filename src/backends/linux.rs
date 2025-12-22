use super::HdrBackend;
use anyhow::{Result, anyhow};
use std::process::Command;

pub struct LinuxBackend;

impl HdrBackend for LinuxBackend {
    fn get_monitors(&self) -> Result<Vec<String>> {
        // Simple implementation: parse kscreen-doctor output
        let output = Command::new("kscreen-doctor").arg("-j").output()?;
        let stdout = String::from_utf8(output.stdout)?;
        // Placeholder for JSON parsing logic or simple grep
        // Ideally you would parse the JSON here
        Ok(vec!["eDP-1".to_string(), "HDMI-A-1".to_string()])
    }

    fn toggle_hdr(&self, monitor_name: &str, enable: bool) -> Result<()> {
        let state = if enable { "enable" } else { "disable" };
        let arg_hdr = format!("output.{}.hdr.{}", monitor_name, state);
        let arg_wcg = format!("output.{}.wcg.{}", monitor_name, state);
        
        let status = Command::new("kscreen-doctor")
            .arg(arg_hdr)
            .arg(arg_wcg)
            .status()?;
            
        if !status.success() {
            return Err(anyhow!("kscreen-doctor failed"));
        }
        Ok(())
    }

    fn notify(&self, title: &str, message: &str) {
        // Fire and forget notification
        let _ = Command::new("notify-send")
            .arg(title)
            .arg(message)
            .spawn();
    }
}