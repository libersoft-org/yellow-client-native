#[cfg(target_os = "linux")]
use gtk::gdk;
#[cfg(target_os = "linux")]
use gtk::prelude::*;
#[cfg(target_os = "linux")]
use std::sync::mpsc;


#[cfg(target_os = "macos")]
use cocoa::appkit::NSScreen;
#[cfg(target_os = "macos")]
use cocoa::base::nil;
#[cfg(target_os = "macos")]
use cocoa::foundation::NSRect;
#[cfg(target_os = "macos")]
use objc::runtime::Object;

//#[cfg(not(any(target_os = "android", target_os = "ios")))]
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
        info!("input name string length: {}", monitor_name.len());
        info!("monitor name string length: {}", m.name.len());
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


#[cfg(target_os = "linux")]
fn os_monitors_info() -> Vec<MonitorInfo> {
    let (tx, rx) = mpsc::channel();
   glib::MainContext::default().invoke(move || {
        let monitors = os_monitors_info2();
        // Send the result back through the channel. If the receiver is waiting, this will unblock it.
        let _ = tx.send(monitors);
    });
    rx.recv().unwrap_or_else(|_| {
        eprintln!("Failed to get monitor information");
        vec![]
    })
}

#[cfg(target_os = "linux")]
fn os_monitors_info2() -> Vec<MonitorInfo> {

    info!("command thread id: {:?}", std::thread::current().id());

    // Get the default GDK display.
    let display = match gtk::gdk::Display::default() {
        Some(d) => d,
        None => {
            eprintln!("No default GDK display available.");
            return vec![];
        }
    };

    let mut monitors_info = Vec::new();

    // The number of monitors attached to the display.
    let n_monitors = display.n_monitors();

    for i in 0..n_monitors {
        if let Some(monitor) = display.monitor(i) {
            // Retrieve the geometry and workarea.
            // Both methods are available via the MonitorExt trait.
            let geometry = monitor.geometry();
            let workarea = monitor.workarea();
            let scale_factor = monitor.scale_factor() as f64; // Get scale factor as f64

            // Try to obtain a name for the monitor.
            // Depending on your GDK version, you might have methods like model() or manufacturer().
            // If those are not available, we fall back to a generated name.
            let name: String = match monitor.model() {
                Some(model) => model.to_string(),
                None => format!("Monitor {}", i),
            };

            // Calculate scaled dimensions
            let geo_x = geometry.x() as f64;
            let geo_y = geometry.y() as f64;
            let geo_width = geometry.width() as f64;
            let geo_height = geometry.height() as f64;

            let work_x = workarea.x() as f64;
            let work_y = workarea.y() as f64;
            let work_width = workarea.width() as f64;
            let work_height = workarea.height() as f64;


            monitors_info.push(MonitorInfo {
                name,
                area: Area {
                    left: (geo_x * scale_factor) as u32,
                    top: (geo_y * scale_factor) as u32,
                    right: ((geo_x + geo_width) * scale_factor) as u32,
                    bottom: ((geo_y + geo_height) * scale_factor) as u32,
                },
                work_area: Area {
                    left: (work_x * scale_factor) as u32,
                    top: (work_y * scale_factor) as u32,
                    right: ((work_x + work_width) * scale_factor) as u32,
                    bottom: ((work_y + work_height) * scale_factor) as u32,
                },
            });
        }
    }

    monitors_info
}




#[cfg(target_os = "macos")]
fn os_monitors_info() -> Vec<MonitorInfo> {
    use cocoa::appkit::NSScreen;
    use cocoa::base::nil;
    use objc::runtime::Object;
    let screens = unsafe { NSScreen::screens(nil) };
    let count = unsafe { screens.count() };
    let mut monitors_info = Vec::new();
    for i in 0..count {
        // Get the NSScreen pointer
        let screen = unsafe { screens.objectAtIndex(i) };
        // Retrieve the full frame (in points)
        let frame = unsafe { NSScreen::frame(screen) };
        // Retrieve the visible (work) frame (in points)
        let visible_frame = unsafe { NSScreen::visibleFrame(screen) };
        // Retrieve the scale factor (backingScaleFactor) to convert points to physical pixels
        let scale_factor = unsafe { NSScreen::backingScaleFactor(screen) } as f64;
        let area = Area {
            left: (frame.origin.x * scale_factor) as u32,
            top: (frame.origin.y * scale_factor) as u32,
            right: ((frame.origin.x + frame.size.width) * scale_factor) as u32,
            bottom: ((frame.origin.y + frame.size.height) * scale_factor) as u32,
        };
        let work_area = Area {
            left: (visible_frame.origin.x * scale_factor) as u32,
            top: (visible_frame.origin.y * scale_factor) as u32,
            right: ((visible_frame.origin.x + visible_frame.size.width) * scale_factor) as u32,
            bottom: ((visible_frame.origin.y + visible_frame.size.height) * scale_factor) as u32,
        };
        // macOS does not always provide a human‐friendly display name, so we use an index.
        let name = format!("Screen {}", i);
        monitors_info.push(MonitorInfo { name, area, work_area });
    }
    monitors_info
}

