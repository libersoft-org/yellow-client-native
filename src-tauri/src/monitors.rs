use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Area {
    left: u32,
    top: u32,
    right: u32,
    bottom: u32,
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
        if m.name.eq(&monitor_name) {
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
                left: monitor.position().x as u32,
                top: monitor.position().y as u32 + 60,
                right: monitor.position().x as u32 + monitor.size().width as u32,
                bottom: monitor.position().y as u32 + monitor.size().height as u32 - 60,
            });
        }
    }
    info!("Monitor not found in available_monitors: {}", &monitor_name);
    return Result::Ok(Area {
        left: 50,
        top: 50,
        right: 150,
        bottom: 150,
    });
}

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::{LPARAM, RECT},
    core::BOOL,
    Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR},
    Win32::UI::WindowsAndMessaging::{GetMonitorInfoA, MONITORINFOEXA},
};

#[cfg(target_os = "windows")]
#[tauri::command]
fn os_monitors_info() -> Vec<MonitorInfo> {
    let mut results: Vec<MonitorInfo> = Vec::new();
    unsafe {
        let _ = EnumDisplayMonitors(
            None,   // kontext (null = všechny monitory)
            None, // klipovací obdélník (null = neomezovat)
            // Callback
            Some(enum_monitor_proc),
            // Předáme ukazatel na náš vektor jako lParam:
            LPARAM(&mut results as *mut _ as isize),
        );
    }
    results
}

// Callback funkce pro každý monitor (Win32 API volá tuto funkci):
#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_monitor_proc(
    hmon: HMONITOR,
    _hdc: HDC,
    _rc: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let vec_ptr = lparam.0 as *mut Vec<MonitorInfo>;
    if let Some(monitors) = vec_ptr.as_mut() {
        let mut info = MONITORINFOEXA {
            monitorInfo: windows::Win32::UI::WindowsAndMessaging::MONITORINFO {
                cbSize: std::mem::size_of::<windows::Win32::UI::WindowsAndMessaging::MONITORINFO>() as u32,
                rcMonitor: Default::default(),
                rcWork: Default::default(),
                dwFlags: 0,
            },
            szDevice: [0; 32],
        };
        
        let result = GetMonitorInfoA(hmon, &mut info.monitorInfo as *mut _);
        if result.as_bool() {
            // Generate a monitor name from its handle ID
            let monitor_name = format!("Monitor_{}", hmon.0);
            
            monitors.push(MonitorInfo {
                name: monitor_name,
                area: Area {
                    left: info.monitorInfo.rcMonitor.left as u32,
                    top: info.monitorInfo.rcMonitor.top as u32,
                    right: info.monitorInfo.rcMonitor.right as u32,
                    bottom: info.monitorInfo.rcMonitor.bottom as u32,
                },
                work_area: Area {
                    left: info.monitorInfo.rcWork.left as u32,
                    top: info.monitorInfo.rcWork.top as u32,
                    right: info.monitorInfo.rcWork.right as u32,
                    bottom: info.monitorInfo.rcWork.bottom as u32,
                },
            });
        }
    }
    BOOL(1)
}

//
// #[cfg(target_os = "macos")]
//
//
//
//

#[cfg(target_os = "linux")]
#[tauri::command]
fn os_monitors_info() -> Vec<MonitorInfo> {
    Vec::new()
}
