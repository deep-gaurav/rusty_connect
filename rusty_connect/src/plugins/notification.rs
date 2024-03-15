use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_graphql::{Object, SimpleObject};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_native_tls::{
    // rustls::{
    //     pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer, ServerName},
    //     ClientConfig,
    // },
    native_tls::{self, Identity},
    TlsConnector,
};
use tracing::{debug, info, warn};

use crate::cert::CertPair;

use super::Plugin;

pub struct Notification {
    pub icons_path: PathBuf,
    pub certs: CertPair,
}

#[Object]
impl Notification {
    pub async fn send_notification(&self) -> anyhow::Result<&str> {
        Ok("Failed")
    }
}

impl Plugin for Notification {
    type PluginPayload = NotificationPayload;
    type PluginConfig = NotificationConfig;
    type PluginState = NotificationState;

    fn init(device_mangager: &crate::devices::DeviceManager) -> Self {
        Self {
            icons_path: device_mangager.icons_path.clone(),
            certs: device_mangager.certs.clone(),
        }
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.notification".to_string()]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![]
    }

    async fn parse_payload(
        &self,
        payload: &crate::payloads::Payload,
        address: SocketAddr,
    ) -> Option<Self::PluginPayload> {
        if payload.r#type == "kdeconnect.notification" {
            // info!("Received notification payload {payload:#?}");
            let notif_payload = serde_json::from_value::<Self::PluginPayload>(payload.body.clone());
            match notif_payload {
                Ok(mut notif_payload) => {
                    if let (Some(size), Some(transfer_info), Some(hash)) = (
                        payload.payload_size,
                        &payload.payload_transfer_info,
                        &notif_payload.payload_hash,
                    ) {
                        let file_path = self.icons_path.join(hash);
                        let port = transfer_info.port;
                        notif_payload.icon_path = Some(file_path.to_string_lossy().to_string());

                        if let Err(err) = Self::receive_icon(
                            address,
                            port,
                            size as usize,
                            file_path.as_path(),
                            self.certs.clone(),
                        )
                        .await
                        {
                            warn!("Cannot get icon {err:?}")
                        }
                    }
                    info!("Returning notification payload");
                    return Some(notif_payload);
                }
                Err(err) => warn!("Cant parse notification payload {err:#?}"),
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

impl Notification {
    pub async fn receive_icon(
        address: SocketAddr,
        port: u16,
        size: usize,
        path: &Path,
        certs: CertPair,
    ) -> anyhow::Result<()> {
        const BUFFER_SIZE: usize = 1024;

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
        }
        debug!("Receive icon completed");
        Ok(())
    }
}

//https://github.com/KDE/kdeconnect-kde/blob/705a72c0779babae809928fef4ad018c8562470e/plugins/notifications/README#L13C1-L22C1
#[derive(SimpleObject, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationPayload {
    id: String,

    #[serde(default)]
    app_name: Option<String>,

    #[serde(default)]
    ticker: Option<String>,

    #[serde(default)]
    is_clearable: Option<bool>,

    #[serde(default)]
    is_cancel: Option<bool>,

    #[serde(default)]
    title: Option<String>,

    #[serde(default)]
    text: Option<String>,

    #[serde(default)]
    request_reply_id: Option<String>,

    #[serde(default)]
    silent: Option<bool>,

    #[serde(default)]
    payload_hash: Option<String>,

    #[serde(default)]
    icon_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct NotificationConfig {
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct NotificationState {
    enabled: bool,
}
