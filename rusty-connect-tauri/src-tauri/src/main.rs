// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{collections::HashMap, error::Error};

use api::{BroadcastUdp, Pair};
use gql_subscription::listen_to_server;
use once_cell::sync::{Lazy, OnceCell};
use server::run_server;
use state::{Devices, NotificationState};
use system_tray::generate_system_tray_menu;
use tauri::{
    App, AppHandle, CustomMenuItem, Manager, RunEvent, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, Url,
};
use tokio::sync::RwLock;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use plugins::{clipboard::send_clipboard, ping::send_ping};

pub mod gql_subscription;
pub mod plugins;
pub mod server;
pub mod state;
pub mod system_tray;

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

    let (gql_starter_tx, gql_started_rx) = tokio::sync::oneshot::channel::<()>();

    tauri::async_runtime::spawn(async move {
        info!("Running GQL Server on port {gqlport}");
        if let Err(err) = run_server(&handle, gqlport, gql_starter_tx).await {
            warn!("GQL Server stopped with error {err:?}")
        }
        info!("GQL Server Stopped");
        handle.exit(2);
    });
    gql_started_rx.blocking_recv()?;
    info!("Starting listener");
    let (started_tx, started_rx) = tokio::sync::oneshot::channel::<()>();
    tauri::async_runtime::spawn(async move {
        info!("Running GQL Subscription Listener from port {gqlport}");

        if let Err(err) = listen_to_server(gqlport, &handle2, started_tx).await {
            warn!("GQL Listener stopped with error {err:?}");
        }
        info!("GQL Listener stopped");
        handle2.exit(2)
    });
    started_rx.blocking_recv()?;
    Ok(())
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();

    let tray_menu = generate_system_tray_menu(&[]).unwrap_or_default();
    let tray = SystemTray::new().with_menu(tray_menu);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tauri::Builder::default()
        .manage(Devices {
            devices: RwLock::new(Vec::new()),
        })
        .manage(NotificationState {
            notifications: RwLock::new(HashMap::new()),
        })
        .invoke_handler(tauri::generate_handler![
            refresh_devices,
            pair,
            send_clipboard,
            send_ping,
        ])
        .setup(setup)
        .system_tray(tray)
        .on_system_tray_event(system_tray::handle_system_tray)
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                #[cfg(target_os = "macos")]
                {
                    use cocoa::appkit::{NSApp, NSApplication, NSApplicationActivationPolicy};
                    unsafe {
                        let app = NSApp();
                        app.setActivationPolicy_(
                            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
                        );
                    }
                }
                api.prevent_exit();
            }
        });
}
