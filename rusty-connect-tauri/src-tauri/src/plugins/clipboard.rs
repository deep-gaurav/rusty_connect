use api::SendClipboard;
use clipboard::{ClipboardContext, ClipboardProvider};
use tauri::AppHandle;

use crate::{GQL_PORT, REQWEST_CLIENT};

#[tauri::command]
pub async fn send_clipboard(device_id: Option<String>) -> Result<String, String> {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let content = ctx
        .get_contents()
        .map_err(|e| format!("Cannot get clipboard  {e:?}"))?;
    let response = graphql_client::reqwest::post_graphql::<SendClipboard, _>(
        &REQWEST_CLIENT,
        format!("http://localhost:{GQL_PORT}"),
        api::send_clipboard::Variables { content, device_id },
    )
    .await
    .map_err(|e| format!("{e:?}"))?;
    let response = response
        .data
        .ok_or("Failed to send clipboard".to_string())?;
    Ok(response.plugins.clipboard.send_clipboard)
}
