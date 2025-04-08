use log::info;
use serde::{Deserialize, Serialize};
use tauri::Monitor;

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

    info!("Monitor not found: {}", &monitor_name);

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
            info!("Monitor found in available_monitors: {}", &monitor_name);
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
        left: 100,
        top: 100,
        right: 400,
        bottom: 400,
    });
}

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{BOOL, LPARAM};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR, RECT};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetMonitorInfoW, MONITORINFOEXA};

#[cfg(target_os = "windows")]
#[tauri::command]
fn os_monitors_info() -> Vec<MonitorInfo> {
    let mut results: Vec<MonitorInfo> = Vec::new();
    unsafe {
        EnumDisplayMonitors(
            HDC::default(),   // kontext (null = všechny monitory)
            std::ptr::null(), // klipovací obdélník (null = neomezovat)
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
            cbSize: std::mem::size_of::<MONITORINFOEXA>() as u32,
            ..Default::default()
        };
        GetMonitorInfoW(hmon, &mut info);
        monitors.push(MonitorInfo {
            name: info.szDevice.to_string_lossy().into_owned(),
            area: Area {
                left: info.rcMonitor.left as u32,
                top: info.rcMonitor.top as u32,
                right: info.rcMonitor.right as u32,
                bottom: info.rcMonitor.bottom as u32,
            },
            work_area: Area {
                left: info.rcWork.left as u32,
                top: info.rcWork.top as u32,
                right: info.rcWork.right as u32,
                bottom: info.rcWork.bottom as u32,
            },
        });
    }
    true.into()
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
