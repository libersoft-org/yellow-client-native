// use tauri::Manager;
// use tao::event_loop::EventLoopWindowTarget;
// use tao::monitor::MonitorHandle;

// fn find_monitor_by_name<T>(
//     event_loop: &EventLoopWindowTarget<T>,
//     target_name: &str,
// ) -> Option<MonitorHandle> {
//     event_loop
//         .available_monitors()
//         .find(|monitor| monitor.name().map_or(false, |name| name == target_name))
// }
//
// #[cfg(not(any(target_os = "android", target_os = "ios")))]
// #[tauri::command]
// pub async fn get_work_area2(
//     event_loop: &EventLoopWindowTarget<T>,
//     monitor_name: &str,
// ) -> Result<Area, String> {
//     info!("Getting work area for monitor: {}", &monitor_name);
//     if let Some(monitor) = find_monitor_by_name(event_loop, monitor_name) {
//         let work_area = monitor.workarea();
//         println!("Work area: {:?}", work_area);
//     }
//     return Result::Ok(work_area);
// }

// todo on linux, try to use GDK workarea()
// #[cfg(not(any(target_os = "android", target_os = "ios")))]
// #[tauri::command]
// pub async fn get_work_area2(
//     event_loop: &EventLoopWindowTarget<T>,
//     monitor_name: &str,
// ) -> Result<Area, String> {
//     info!("Getting work area for monitor: {}", &monitor_name);
//     if let Some(monitor) = find_monitor_by_name(event_loop, monitor_name) {
//         let work_area = monitor.workarea();
//         println!("Work area: {:?}", work_area);
//     }
//     return Result::Ok(work_area);
// }
//
