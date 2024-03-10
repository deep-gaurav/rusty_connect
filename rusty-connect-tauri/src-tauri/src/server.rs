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

pub async fn run_server(apphandle: &AppHandle, port: u32) -> anyhow::Result<()> {
    let local_app_dir = apphandle
        .path_resolver()
        .app_local_data_dir()
        .ok_or(anyhow!("No local dir"))?;
    let config_path = local_app_dir.join("conf.json");
    let cert_path = local_app_dir.join("cert");
    let key_path = local_app_dir.join("key");
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

    let mut rusty = RustyConnect::new(
        &config.device_id,
        &config.device_name,
        &config.device_type,
        cert_path,
        key_path,
    );
    rusty.run(port).await?;
    Ok(())
}
