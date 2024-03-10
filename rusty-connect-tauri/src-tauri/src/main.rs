// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fmt::format};

use api::{BroadcastUdp, Pair};
use gql_subscription::listen_to_server;
use once_cell::sync::{Lazy, OnceCell};
use server::run_server;
use tauri::{
    ActivationPolicy, App, AppHandle, CustomMenuItem, Manager, RunEvent, Runtime, SystemTray,
    SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, Url,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod gql_subscription;
pub mod server;

static REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);
static MAIN_URL: OnceCell<Url> = OnceCell::new();
static GQL_PORT: u32 = 33372;

#[tauri::command]
async fn refresh_devices(app: AppHandle) -> Result<(), String> {
    let response = graphql_client::reqwest::post_graphql::<BroadcastUdp, _>(
        &REQWEST_CLIENT,
        format!("http://localhost:{GQL_PORT}"),
        api::broadcast_udp::Variables,
    )
    .await
    .map_err(|e| format!("{e:?}"));
    info!("Broadcasted UDP");
    gql_subscription::refresh_devices(&REQWEST_CLIENT, app, GQL_PORT);
    Ok(())
}

#[tauri::command]
async fn pair(app: AppHandle, device_id: String, pair: bool) -> Result<(), String> {
    info!("Sending pair {device_id} {pair}");
    let response = graphql_client::reqwest::post_graphql::<Pair, _>(
        &REQWEST_CLIENT,
        format!("http://localhost:{GQL_PORT}"),
        api::pair::Variables {
            id: device_id,
            pair,
        },
    )
    .await
    .map_err(|e| format!("{e:?}"));
    info!("Responded with {response:?}");
    gql_subscription::refresh_devices(&REQWEST_CLIENT, app, GQL_PORT);
    Ok(())
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let main_url = app.get_window("main").unwrap().url();
    if let Err(err) = MAIN_URL.set(main_url) {
        warn!("Cant set mainurl {err:?}")
    }
    let handle = app.app_handle();
    let handle2 = app.app_handle();
    let gqlport = GQL_PORT;
    tauri::async_runtime::spawn(async move {
        info!("Running GQL Server on port {gqlport}");
        if let Err(err) = run_server(&handle, gqlport).await {
            warn!("GQL Server stopped with error {err:?}")
        }
        info!("GQL Server Stopped");
        handle.exit(2);
    });
    tauri::async_runtime::spawn(async move {
        info!("Running GQL Subscription Listener from port {gqlport}");

        if let Err(err) = listen_to_server(gqlport, &handle2).await {
            warn!("GQL Listener stopped with error {err:?}");
        }
        info!("GQL Listener stopped");
        handle2.exit(2)
    });
    Ok(())
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("open".to_string(), "Open");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);
    let tray = SystemTray::new().with_menu(tray_menu);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![refresh_devices, pair])
        .setup(setup)
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
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
                        .build();
                        if let Ok(main_window) = main_window {
                            if let Err(err) = main_window.show() {
                                warn!("Cannot show new main window")
                            }
                        }
                    }
                }
                id => {
                    let mut splits = id.split(";");
                    if let (Some(device_id), Some("send_clipboard")) =
                        (splits.next(), splits.next())
                    {}
                    info!("Clicked id {id} and tray_id {tray_id} ")
                }
            },
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}
