use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Area {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct MonitorInfo {
    pub name: String,
    pub area: Area,
    pub work_area: Area,
}

#[tauri::command]
pub async fn get_work_area(monitor_name: String, window: tauri::Window) -> Result<Area, String> {
    info!("Getting work area for monitor: {}", &monitor_name);

    let monitors: Vec<MonitorInfo> = os_monitors_info();
    for m in monitors {
        info!("Monitor: {:?}", m);
        //if m.name.eq(&monitor_name)
        {
            info!("Monitor found: {}", &monitor_name);
            return Result::Ok(m.work_area);
        }
    }

    info!("Monitor not found in os_monitors_info: {}", &monitor_name);

    let _monitors2 = window.available_monitors();
    let monitors2 = match _monitors2 {
        Ok(m) => m,
        Err(e) => {
            info!("Error getting available monitors: {}", e);
            return Result::Err(format!("Error getting available monitors: {}", e));
        }
    };
    for monitor in monitors2 {
        if monitor.name().eq(&Some(&monitor_name)) {
            info!("Monitor found in window.available_monitors: {}", &monitor_name);
            return Result::Ok(Area {
                left: monitor.position().x,
                top: monitor.position().y + 60,
                right: monitor.position().x + monitor.size().width as i32,
                bottom: monitor.position().y + monitor.size().height as i32 - 60,
            });
        }
    }
    info!("Monitor not found in available_monitors: {}", &monitor_name);
    return Result::Ok(Area {
        left: 55,
        top: 55,
        right: 555,
        bottom: 555,
    });
}


#[cfg(target_os = "windows")]
use windows_core::{ BOOL, PCSTR, PWSTR };

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::{ RECT, LPARAM },
    Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, 
        HDC, HMONITOR, DISPLAY_DEVICEW, MONITORINFO, MONITORINFOEXA,
        MONITORINFOEXW
    },
};

#[cfg(target_os = "windows")]
extern "system" fn monitor_enum_proc(monitor: HMONITOR, _: HDC, _: *mut RECT, data: LPARAM) -> BOOL {
    unsafe {
        let results = &mut *(data.0 as *mut Vec<MonitorInfo>);
        
        // Get monitor info
        let mut monitor_info = MONITORINFOEXW {
            monitorInfo: MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFOEXW>() as u32,
                ..Default::default()
            },
            ..Default::default()
        };
        
        if GetMonitorInfoW(monitor, &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO) {
            // Convert device name to string
            let device_name = String::from_utf16_lossy(
                &monitor_info.szDevice[0..monitor_info.szDevice.iter()
                    .position(|&c| c == 0)
                    .unwrap_or(monitor_info.szDevice.len())]
            );
            
            // Extract monitor area (entire screen) and work area (screen minus taskbar/dock)
            let area = &monitor_info.monitorInfo.rcMonitor;
            let work_area = &monitor_info.monitorInfo.rcWork;
            
            results.push(MonitorInfo {
                name: device_name,
                area: Area {
                    left: area.left,
                    top: area.top,
                    right: area.right,
                    bottom: area.bottom,
                },
                work_area: Area {
                    left: work_area.left,
                    top: work_area.top,
                    right: work_area.right,
                    bottom: work_area.bottom,
                },
            });
        }
        
        BOOL(1) // Continue enumeration
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn os_monitors_info() -> Vec<MonitorInfo> {
    let mut results: Vec<MonitorInfo> = Vec::new();
    
    unsafe {
        // Enumerate all display monitors
        EnumDisplayMonitors(
            HDC(0),
            std::ptr::null(),
            Some(monitor_enum_proc),
            LPARAM(&mut results as *mut _ as isize)
        );
        
        // If no monitors found (rare case), try fallback method with EnumDisplayDevices
        if results.is_empty() {
            let mut device_index = 0;
            loop {
                let mut display_device = DISPLAY_DEVICEW {
                    cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
                    ..Default::default()
                };

                // Get display device info
                if !EnumDisplayDevicesW(
                    PCSTR::null(),
                    device_index,
                    &mut display_device,
                    0, // No flags
                ).as_bool() {
                    // No more devices
                    break;
                }

                // Only process devices attached to desktop
                if display_device.StateFlags & 0x1 != 0 { // DISPLAY_DEVICE_ATTACHED_TO_DESKTOP
                    // Convert device name to string
                    let device_name = String::from_utf16_lossy(
                        &display_device.DeviceName[0..display_device.DeviceName.iter()
                            .position(|&c| c == 0)
                            .unwrap_or(display_device.DeviceName.len())]
                    );

                    // Add monitor info with fallback values
                    results.push(MonitorInfo {
                        name: device_name,
                        area: Area {
                            left: 0,
                            top: 0,
                            right: 1020,
                            bottom: 1080,
                        },
                        work_area: Area {
                            left: 0,
                            top: 0,
                            right: 1020,
                            bottom: 1040, // Approximate taskbar height subtracted
                        },
                    });
                }

                device_index += 1;
            }
        }
    }

    info!("Found {} displays", results.len());
    results
}

#[cfg(target_os = "linux")]
#[tauri::command]
fn os_monitors_info() -> Vec<MonitorInfo> {
    Vec::new()
}

//
// #[cfg(target_os = "macos")]
//
//
//
//
