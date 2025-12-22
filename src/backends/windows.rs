use super::HdrBackend;
use anyhow::{anyhow, Result};
use windows::Win32::Devices::Display::{
    DisplayConfigGetDeviceInfo, DisplayConfigSetDeviceInfo, GetDisplayConfigBufferSizes,
    QueryDisplayConfig, DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
    DISPLAYCONFIG_DEVICE_INFO_HEADER, DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE,
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO,
    DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE, DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE_0,
    DISPLAYCONFIG_TARGET_DEVICE_NAME, DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS,
    DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY, QDC_ONLY_ACTIVE_PATHS, QDC_VIRTUAL_MODE_AWARE,
};

pub struct WindowsBackend;

impl HdrBackend for WindowsBackend {
    fn get_monitors(&self) -> Result<Vec<String>> {
        let paths = query_display_config()?;
        let mut names = Vec::new();

        for path in paths {
            let name = get_monitor_name(&path)?;
            if !name.is_empty() {
                names.push(name);
            }
        }
        names.sort();
        names.dedup();
        Ok(names)
    }

    fn toggle_hdr(&self, monitor_name: &str, enable: bool) -> Result<()> {
        let paths = query_display_config()?;
        let mut found = false;

        for path in paths {
            let name = get_monitor_name(&path)?;
            if name.eq_ignore_ascii_case(monitor_name) {
                found = true;
                set_hdr_state(&path, enable)?;
            }
        }

        if !found {
            return Err(anyhow!("Monitor '{}' not found.", monitor_name));
        }
        Ok(())
    }

    fn notify(&self, title: &str, message: &str) {
        // Powershell script to show a balloon tip in the system tray
        let ps_script = format!(
            "[void] [System.Reflection.Assembly]::LoadWithPartialName('System.Windows.Forms'); \
            $obj = New-Object System.Windows.Forms.NotifyIcon; \
            $obj.Icon = [System.Drawing.SystemIcons]::Information; \
            $obj.Visible = $True; \
            $obj.ShowBalloonTip(1000, '{}', '{}', [System.Windows.Forms.ToolTipIcon]::Info); \
            Start-Sleep -s 3; $obj.Visible = $False;",
            title, message
        );

        let _ = std::process::Command::new("powershell")
            .args(["-WindowStyle", "Hidden", "-Command", &ps_script])
            .spawn();
    }
}

// --- Low Level Win32 Helpers ---
// (Keep the existing query_display_config, get_monitor_name, set_hdr_state functions here exactly as before)
// ...
fn query_display_config() -> Result<Vec<DISPLAYCONFIG_PATH_INFO>> {
    let mut num_paths = 0;
    let mut num_modes = 0;
    let flags = QDC_ONLY_ACTIVE_PATHS | QDC_VIRTUAL_MODE_AWARE;

    unsafe {
        let ret = GetDisplayConfigBufferSizes(flags, &mut num_paths, &mut num_modes);
        if ret.0 != 0 {
            return Err(anyhow!("Failed to get display config sizes: {:?}", ret));
        }

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); num_paths as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); num_modes as usize];

        let ret = QueryDisplayConfig(
            flags,
            &mut num_paths,
            paths.as_mut_ptr(),
            &mut num_modes,
            modes.as_mut_ptr(),
            None,
        );

        if ret.0 != 0 {
            return Err(anyhow!("QueryDisplayConfig failed: {:?}", ret));
        }
        paths.truncate(num_paths as usize);
        Ok(paths)
    }
}

fn get_monitor_name(path: &DISPLAYCONFIG_PATH_INFO) -> Result<String> {
    let mut target_name = DISPLAYCONFIG_TARGET_DEVICE_NAME {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME,
            size: std::mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32,
            adapterId: path.sourceInfo.adapterId,
            id: path.targetInfo.id,
        },
        flags: DISPLAYCONFIG_TARGET_DEVICE_NAME_FLAGS::default(),
        outputTechnology: DISPLAYCONFIG_VIDEO_OUTPUT_TECHNOLOGY::default(),
        edidManufactureId: 0,
        edidProductCodeId: 0,
        connectorInstance: 0,
        monitorFriendlyDeviceName: [0; 64],
        monitorDevicePath: [0; 128],
    };

    unsafe {
        let ret = DisplayConfigGetDeviceInfo(&mut target_name.header);
        if ret != 0 { return Ok("Unknown".to_string()); }
    
        let name_slice = &target_name.monitorFriendlyDeviceName;
        let len = name_slice.iter().position(|&c| c == 0).unwrap_or(name_slice.len());
        let name = String::from_utf16_lossy(&name_slice[0..len]);
        Ok(name)
    }
}

fn set_hdr_state(path: &DISPLAYCONFIG_PATH_INFO, enable: bool) -> Result<()> {
    let mut set_packet = DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE {
        header: DISPLAYCONFIG_DEVICE_INFO_HEADER {
            r#type: DISPLAYCONFIG_DEVICE_INFO_SET_ADVANCED_COLOR_STATE,
            size: std::mem::size_of::<DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE>() as u32,
            adapterId: path.sourceInfo.adapterId,
            id: path.targetInfo.id,
        },
        Anonymous: DISPLAYCONFIG_SET_ADVANCED_COLOR_STATE_0 {
            value: if enable { 1 } else { 0 },
        },
    };

    unsafe {
        let ret = DisplayConfigSetDeviceInfo(&mut set_packet.header);
        if ret != 0 {
            return Err(anyhow!("Failed to set HDR state. Error Code: {:?}", ret));
        }
    }
    Ok(())
}