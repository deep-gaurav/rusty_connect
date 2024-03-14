use api::{SendClipboard, SendPing};
use clipboard::{ClipboardContext, ClipboardProvider};
use tauri::AppHandle;

use crate::{GQL_PORT, REQWEST_CLIENT};

#[tauri::command]
pub async fn send_ping(device_id: Option<String>) -> Result<String, String> {
    let response = graphql_client::reqwest::post_graphql::<SendPing, _>(
        &REQWEST_CLIENT,
        format!("http://localhost:{GQL_PORT}"),
        api::send_ping::Variables { device_id },
    )
    .await
    .map_err(|e| format!("{e:?}"))?;
    let response = response
        .data
        .ok_or("Failed to send clipboard".to_string())?;
    Ok(response.plugins.ping.send_ping)
}
