use api::all_devices::DeviceWithStateFields;
use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    SystemTraySubmenu,
};
use tracing::{info, warn};

use crate::plugins::clipboard::send_clipboard;

pub fn generate_system_tray_menu(
    devices: &[DeviceWithStateFields],
) -> anyhow::Result<SystemTrayMenu> {
    let mut system_menu =
        SystemTrayMenu::new().add_item(CustomMenuItem::new("devices", "Devices").disabled());
    for device in devices.iter().filter(|d| d.is_connected && d.device.paired) {
        let menu = SystemTrayMenu::new().add_item(CustomMenuItem::new(
            format!("{};send_clipboard", device.device.id),
            "Send Clipboard",
        ));
        system_menu = system_menu
            .add_submenu(SystemTraySubmenu::new(
                device.device.identity.device_name.clone(),
                menu,
            ))
            .add_native_item(SystemTrayMenuItem::Separator);
    }

    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("open".to_string(), "Open");
    system_menu = system_menu
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    Ok(system_menu)
}

pub fn handle_system_tray(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a left click");
        }
        SystemTrayEvent::RightClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a right click");
        }
        SystemTrayEvent::DoubleClick {
            position: _,
            size: _,
            ..
        } => {
            println!("system tray received a double click");
        }
        SystemTrayEvent::MenuItemClick { id, tray_id, .. } => match id.as_str() {
            "quit" => {
                app.exit(0);
            }
            "open" => {
                let window = app.get_window("main");
                if let Some(window) = window {
                    window.show().unwrap();
                } else {
                    let main_window = tauri::WindowBuilder::new(
                        app,
                        "main", /* the unique window label */
                        tauri::WindowUrl::App("index.html".into()),
                    )
                    .title("RustyConnect")
                    .build();
                    if let Ok(main_window) = main_window {
                        #[cfg(target_os = "macos")]
                        {
                            use cocoa::appkit::{
                                NSApp, NSApplication, NSApplicationActivationPolicy,
                            };
                            unsafe {
                                let app = NSApp();
                                app.setActivationPolicy_(
                                    NSApplicationActivationPolicy::NSApplicationActivationPolicyRegular,
                                );
                            }
                        }
                        if let Err(err) = main_window.show() {
                            warn!("Cannot show new main window {err:?}")
                        }
                    }
                }
            }
            id => {
                let mut splits = id.split(';');
                if let (Some(device_id), Some("send_clipboard")) = (splits.next(), splits.next()) {
                    let device_id = Some(device_id.to_string());
                    let app = app.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(err) = send_clipboard(app, device_id).await {
                            warn!("Cant send clipboard {err:?}")
                        }
                    });
                }
                info!("Clicked id {id} and tray_id {tray_id} ")
            }
        },
        _ => {}
    }
}