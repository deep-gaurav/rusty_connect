// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use gql_subscription::listen_to_server;
use once_cell::sync::Lazy;
use server::run_server;
use tauri::{App, AppHandle, Manager, Runtime};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub mod gql_subscription;
pub mod server;

static REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(reqwest::Client::new);
static GQL_PORT: u32 = 33372;

#[tauri::command]
async fn refresh_devices(app: AppHandle) -> Result<(), String> {
    gql_subscription::refresh_devices(&REQWEST_CLIENT, &app, GQL_PORT);
    Ok(())
}

fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    let handle = app.handle();
    let handle2 = app.handle();
    let gqlport = GQL_PORT;
    tauri::async_runtime::spawn(async move {
        info!("Running GQL Server on port {gqlport}");
        if let Err(err) = run_server(&handle, gqlport).await {
            warn!("GQL Server stopped with error {err:?}")
        }
        info!("GQL Server Stopped")
    });
    tauri::async_runtime::spawn(async move {
        info!("Running GQL Subscription Listener from port {gqlport}");

        if let Err(err) = listen_to_server(gqlport, &handle2).await {
            warn!("GQL Listener stopped with error {err:?}");
        }
        info!("GQL Listener stopped")
    });
    Ok(())
}

fn main() {
    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true);
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![refresh_devices])
        .setup(setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
