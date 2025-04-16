//use std::collections::VecDeque;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
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


#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[tauri::command]
pub async fn get_work_area(monitor_name: String, window: tauri::Window) -> Result<Area, String> {
    info!("Getting work area for monitor: {}", &monitor_name);

    let monitors: Vec<MonitorInfo> = os_monitors_info();
    for m in monitors {
        info!("Monitor: {:?}", m);
        info!("Monitor name: {:?}", m.name);
        if m.name.eq(&monitor_name)
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
use windows_core::BOOL;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::{LPARAM, RECT},
    Win32::Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW},
};

#[cfg(target_os = "windows")]
#[tauri::command]
fn os_monitors_info() -> Vec<MonitorInfo> {
    let mut results: Vec<MonitorInfo> = Vec::new();
    let res;
    unsafe {
    res =
        EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitor_proc),
            LPARAM(&mut results as *mut _ as isize),
        );
    };
    if res.as_bool() {
        info!("Found {} displays", results.len());
    } else {
        info!("Failed to enumerate monitors");
    }
    results
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_monitor_proc(
    hmon: HMONITOR,
    _hdc: HDC,
    _rc: *mut RECT,
    lparam: LPARAM,
) -> BOOL {
    let vec_ptr = lparam.0 as *mut Vec<MonitorInfo>;
    if let Some(monitors) = vec_ptr.as_mut() {
        let mut info = MONITORINFOEXW {
            monitorInfo: MONITORINFOEXW::default().monitorInfo,
            szDevice: [0; 32],
        };
        info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;

        let result = GetMonitorInfoW(hmon, &mut info as *mut _ as *mut _);
        if result.as_bool() {
            let device_name = String::from_utf16_lossy(
                &info.szDevice.iter().take_while(|&&c| c != 0).copied().collect::<Vec<u16>>(),
            );

            monitors.push(MonitorInfo {
                name: device_name,
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


// pub struct MonitorHandle(isize);
//
// unsafe extern "system" fn monitor_enum_proc(
//     hmonitor: HMONITOR,
//     _hdc: HDC,
//     _place: *mut RECT,
//     data: LPARAM,
// ) -> BOOL {
//     let monitors = data.0 as *mut VecDeque<MonitorHandle>;
//     (*monitors).push_back(MonitorHandle::new(hmonitor));
//     true.into() // continue enumeration
// }
//
// #[tauri::command]
// pub fn available_monitors() -> VecDeque<MonitorHandle> {
//     let mut monitors: VecDeque<MonitorHandle> = VecDeque::new();
//     unsafe {
//         let _ = EnumDisplayMonitors(
//             None,
//             None,
//             Some(monitor_enum_proc),
//             LPARAM(&mut monitors as *mut _ as _),
//         );
//     }
//     monitors
// }
