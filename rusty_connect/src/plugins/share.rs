use std::{
    collections::HashMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_graphql::{Object, SimpleObject, Subscription, Union};
use async_stream::stream;

use futures::Stream;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};
use tokio_native_tls::{
    // rustls::{
    //     pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer, ServerName},
    //     ClientConfig,
    // },
    native_tls::{self, Identity},
};
use tracing::{debug, info, warn};

use crate::{cert::CertPair, payloads::PayloadTransferInfo};

use super::Plugin;

pub struct Share {
    pub downloads_path: PathBuf,
    pub certs: CertPair,
    pub download_tasks: Arc<Mutex<HashMap<String, tokio::sync::watch::Receiver<DownloadProgress>>>>,
}

#[Object]
impl Share {
    pub async fn send_file(&self) -> anyhow::Result<&str> {
        Ok("Failed")
    }
}

impl Plugin for Share {
    type PluginPayload = SharePayload;
    type PluginConfig = ShareConfig;
    type PluginState = ShareState;

    fn init(device_mangager: &crate::devices::DeviceManager) -> Self {
        Self {
            certs: device_mangager.certs.clone(),
            downloads_path: device_mangager.downloads_path.clone(),
            download_tasks: device_mangager.download_tasks.clone(),
        }
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec![
            "kdeconnect.share.request".to_string(),
            "kdeconnect.share.request.update".to_string(),
        ]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![]
    }

    async fn parse_payload(
        &self,
        payload: &crate::payloads::Payload,
        address: SocketAddr,
    ) -> Option<Self::PluginPayload> {
        info!("Payload received {payload:#?}");

        if payload.r#type == "kdeconnect.share.request" {
            if let Ok(mut share_payload) =
                serde_json::from_value::<Self::PluginPayload>(payload.body.clone())
            {
                if share_payload.number_of_files == Some(1) {
                    if let (Some(file_name), Some(total_size), Some(PayloadTransferInfo { port })) = (
                        &share_payload.filename,
                        share_payload.total_payload_size,
                        &payload.payload_transfer_info,
                    ) {
                        let file_path = self.downloads_path.join(file_name);
                        let (tx, rx) =
                            tokio::sync::watch::channel(DownloadProgress::NotStarted(NotStarted {
                                total_bytes: total_size,
                            }));
                        let certs = self.certs.clone();
                        let port = *port;
                        let _download_task = tokio::spawn(async move {
                            if let Err(err) = Self::receive_file(
                                address,
                                port,
                                total_size as usize,
                                &file_path,
                                certs,
                                &tx,
                            )
                            .await
                            {
                                warn!("Download failed {err:?} ");
                                tx.send_replace(DownloadProgress::Failed(DownloadFailed {
                                    reason: format!("{err:#?}"),
                                }));
                            }
                        });
                        {
                            let download_id = uuid::Uuid::new_v4().to_string();
                            let mut tasks = self.download_tasks.lock().await;
                            tasks.insert(download_id.clone(), rx);
                            share_payload.download_id = Some(download_id);
                        }
                        return Some(share_payload);
                    }
                } else {
                    warn!("file number not 1");
                }
            }
        }
        None
    }

    fn is_enabled(&self, config: &Option<Self::PluginConfig>) -> bool {
        if let Some(config) = config {
            config.enabled
        } else {
            true
        }
    }

    fn should_send(
        &self,
        _config: &Option<Self::PluginConfig>,
        _state: &mut Self::PluginState,
        _payload: &Self::PluginPayload,
    ) -> bool {
        false
    }
}

impl Share {
    pub async fn receive_file(
        address: SocketAddr,
        port: u16,
        size: usize,
        path: &Path,
        certs: CertPair,
        progress_sender: &tokio::sync::watch::Sender<DownloadProgress>,
    ) -> anyhow::Result<()> {
        const BUFFER_SIZE: usize = 10 * 1024;

        debug!("TCP Stream initialising");
        let stream = TcpStream::connect(SocketAddr::new(address.ip(), port)).await?;

        debug!("TLS Config initializing");
        let key = Identity::from_pkcs8(&certs.0, &certs.1)?;
        let connector = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .identity(key)
            .build()?;
        let tls_connector = tokio_native_tls::TlsConnector::from(connector);

        // let server_name = ServerName::IpAddress(address.ip().into());
        let server_name = address.to_string();
        debug!("Upgrading to TLS Stream to server name: {server_name:?}");
        let mut tls_stream = tls_connector.connect(&server_name, stream).await?;

        // debug!("Creating file");
        let mut file = tokio::fs::File::create(path).await?;

        let mut buffer = vec![0u8; BUFFER_SIZE];

        let mut total_bytes_read = 0;
        while total_bytes_read < size {
            let bytes_to_read = std::cmp::min(size - total_bytes_read, BUFFER_SIZE);
            let bytes_read = tls_stream.read(&mut buffer[..bytes_to_read]).await?;
            if bytes_read == 0 {
                // End of stream reached
                break;
            }

            // Write the read data to the file
            file.write_all(&buffer[..bytes_read]).await?;

            // Update the total bytes read
            total_bytes_read += bytes_read;

            progress_sender.send_replace(DownloadProgress::Downloading(Progress {
                total_bytes: size as u64,
                read_bytes: total_bytes_read as u64,
            }));
        }
        if total_bytes_read == size {
            progress_sender.send_replace(DownloadProgress::Completed(Completed {
                total_bytes: total_bytes_read as u64,
                path: path.to_string_lossy().to_string(),
            }));
        } else {
            progress_sender.send_replace(DownloadProgress::Failed(DownloadFailed {
                reason: "stopped before completion".to_string(),
            }));
        }
        debug!("Receive icon completed");
        Ok(())
    }
}

//https://github.com/KDE/kdeconnect-kde/blob/705a72c0779babae809928fef4ad018c8562470e/plugins/notifications/README#L13C1-L22C1
#[derive(SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharePayload {
    text: Option<String>,
    filename: Option<String>,
    last_modified: Option<u64>,
    number_of_files: Option<u32>,
    total_payload_size: Option<u64>,

    #[serde(default)]
    download_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct ShareConfig {
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct ShareState {
    enabled: bool,
}

#[derive(Debug, SimpleObject, Clone)]
pub struct Progress {
    total_bytes: u64,
    read_bytes: u64,
}

#[derive(Debug, SimpleObject, Clone)]
pub struct NotStarted {
    total_bytes: u64,
}

#[derive(Debug, SimpleObject, Clone)]
pub struct Completed {
    total_bytes: u64,
    path: String,
}

#[derive(Debug, SimpleObject, Clone)]
pub struct DownloadFailed {
    reason: String,
}

#[derive(Debug, Union, Clone)]
pub enum DownloadProgress {
    NotStarted(NotStarted),
    Downloading(Progress),
    Completed(Completed),
    Failed(DownloadFailed),
}
