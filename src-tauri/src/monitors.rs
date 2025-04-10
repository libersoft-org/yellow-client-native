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
        info!("Monitor: {:?}", m);
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
        left: 55,
        top: 55,
        right: 555,
        bottom: 555,
    });
}

#[cfg(target_os = "windows")]
use windows_core::{ BOOL };


#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::{LPARAM, RECT },
    Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR, MONITORINFO, GetMonitorInfoW},
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
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let result = GetMonitorInfoW(hmon, &mut info);
        if result.as_bool() {
            // Generate a monitor name from its handle ID
            let monitor_name = format!("Monitor_{:p}", hmon.0);

            monitors.push(MonitorInfo {
                name: monitor_name,
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
    }
    BOOL(1)
}


// #[cfg(target_os = "windows")]
// use windows_core::{ BOOL, PCSTR };
//
// #[cfg(target_os = "windows")]
// use windows::{
//     Win32::Foundation::{ RECT },
//     Win32::Graphics::Gdi::{
//         EnumDisplayDevicesA, DISPLAY_DEVICEA
//     },
// };
//
// #[cfg(target_os = "windows")]
// #[tauri::command]
// fn os_monitors_info() -> Vec<MonitorInfo> {
//     let mut results: Vec<MonitorInfo> = Vec::new();
//     unsafe {
//         let mut device_index = 0;
//         loop {
//             let mut display_device = DISPLAY_DEVICEA {
//                 cb: std::mem::size_of::<DISPLAY_DEVICEA>() as u32,
//                 ..Default::default()
//             };
//
//             // Get display device info
//             if !EnumDisplayDevicesA(
//                 PCSTR::null(),
//                 device_index,
//                 &mut display_device,
//                 0, // No flags
//             ).as_bool() {
//                 // No more devices
//                 break;
//             }
//
//             // Convert device name to string
//             let device_name = std::ffi::CStr::from_ptr(display_device.DeviceName.as_ptr() as *const i8)
//                 .to_string_lossy()
//                 .to_string();
//
//             // Add a dummy monitor info (actual area would need to be obtained separately)
//             results.push(MonitorInfo {
//                 name: device_name,
//                 area: Area {
//                     left: 0,
//                     top: 0,
//                     right: 1920, // Default values, would need to be fetched correctly
//                     bottom: 1080,
//                 },
//                 work_area: Area {
//                     left: 0,
//                     top: 0,
//                     right: 1920,
//                     bottom: 1080,
//                 },
//             });
//
//             device_index += 1;
//         }
//     }
//
//     info!("Found {} displays", results.len());
//     results
// }

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
