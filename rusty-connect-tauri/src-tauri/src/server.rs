use anyhow::anyhow;
use rusty_connect::RustyConnect;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
struct RustyConfig {
    device_id: String,
    device_name: String,
    device_type: String,
}

pub async fn run_server(
    apphandle: &AppHandle,
    port: u32,
    started_sender: tokio::sync::oneshot::Sender<()>,
) -> anyhow::Result<()> {
    let local_app_dir = apphandle
        .path_resolver()
        .app_local_data_dir()
        .ok_or(anyhow!("No local dir"))?;
    if !tokio::fs::try_exists(&local_app_dir).await? {
        tokio::fs::create_dir_all(&local_app_dir).await?;
    }
    if !tokio::fs::metadata(&local_app_dir).await?.is_dir() {
        return Err(anyhow::anyhow!("Local app dir not directory"));
    }
    let config_path = local_app_dir.join("conf.json");
    let config = {
        let data = tokio::fs::read(&config_path)
            .await
            .map_err(|e| anyhow::anyhow!("{e:?}"))
            .and_then(|data| {
                serde_json::from_slice::<RustyConfig>(&data).map_err(|e| anyhow::anyhow!("{e:?}"))
            });
        match data {
            Ok(config) => config,
            Err(err) => {
                debug!("Cannot load config, creating new {err:?}");
                let config = RustyConfig {
                    device_id: uuid::Uuid::new_v4().to_string(),
                    device_name: "RustyConnect".to_string(),
                    device_type: "laptop".to_string(),
                };
                tokio::fs::write(
                    &config_path,
                    serde_json::to_vec(&config)
                        .map_err(|e| anyhow::anyhow!("Cannot serialize config {e:?}"))?,
                )
                .await?;
                config
            }
        }
    };
    started_sender
        .send(())
        .map_err(|_| anyhow::anyhow!("Cannot send started"))?;
    let mut rusty = RustyConnect::new(
        &config.device_id,
        &config.device_name,
        &config.device_type,
        &local_app_dir,
    )
    .await?;
    rusty.run(port).await?;
    Ok(())
}
