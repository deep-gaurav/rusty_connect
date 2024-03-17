use api::{all_devices::IdentityPayloadFields, download_progress, DownloadProgress, API};
use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
use futures::StreamExt;
use graphql_ws_client::{graphql::StreamingOperation, Client};
use tauri::AppHandle;
use tracing::warn;

pub async fn monitor_download_progress(
    app: AppHandle,
    port: u32,
    device: IdentityPayloadFields,
    download_id: String,
) -> anyhow::Result<()> {
    let mut request = format!("ws://localhost:{port}/ws").into_client_request()?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_str("graphql-transport-ws")?,
    );
    let (connection, _) = async_tungstenite::tokio::connect_async(request).await?;
    let mut subscription = Client::build(connection)
        .subscribe(StreamingOperation::<DownloadProgress>::new(
            download_progress::Variables {
                download_id: download_id.clone(),
            },
        ))
        .await?;

    while let Some(Ok(item)) = subscription.next().await {
        if let Some(data) = item.data {
            if let Err(err) = (API {
                download_progress: Some((
                    download_id.clone(),
                    device.clone(),
                    data.download_update,
                )),
                ..Default::default()
            })
            .emit(&app, api::APIEmit::DownloadProgress)
            {
                warn!("Cant send download update {err:?}")
            }
        }
    }
    Ok(())
}
