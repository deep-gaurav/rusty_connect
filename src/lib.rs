use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQL, GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use axum::response::{Html, IntoResponse};
use axum::Extension;
use cert::certgen::generate_cert;
use devices::DeviceManager;
use plugins::PluginManager;
use schema::subscription::Subscription;
use schema::{mutation::Mutation, query::Query, GQSchema};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio::sync::RwLock;
use tokio_rustls::client::TlsStream;
use tokio_rustls::rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer, ServerName};
use tokio_rustls::rustls::ClientConfig;
use tokio_rustls::TlsConnector;

use tracing::{debug, error, warn};

use crate::cert::no_veifier::NoVerifier;
use crate::payloads::{IdentityPayloadBody, Payload};

pub mod cert;
pub mod devices;
pub mod payloads;
pub mod plugins;
pub mod schema;
pub mod utils;

pub struct RustyConnect<C: AsRef<Path>, K: AsRef<Path>> {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub cert_path: C,
    pub key_path: K,
    pub plugin_manager: Arc<PluginManager>,
    pub device_manager: Arc<RwLock<DeviceManager>>,
}

impl<C: AsRef<Path>, K: AsRef<Path>> RustyConnect<C, K> {
    pub fn new(id: &str, name: &str, device_type: &str, cert_path: C, key_path: K) -> Self {
        let (tx, rx) = flume::bounded(0);
        Self {
            id: id.to_string(),
            name: name.to_string(),
            device_type: device_type.to_string(),
            cert_path,
            key_path,
            plugin_manager: Arc::new(PluginManager::new(
                id.to_string(),
                name.to_string(),
                device_type.to_string(),
            )),
            device_manager: Arc::new(RwLock::new(DeviceManager {
                devices: HashMap::new(),
                sender: tx,
                receiver: rx,
            })),
        }
    }

    pub async fn run(&mut self, gql_port: u32) -> anyhow::Result<()> {
        debug!("Starting RustyConnect on port 1716 and GQL on {gql_port}");
        let certs = generate_cert(&self.id, &self.cert_path, &self.key_path).await?;
        let tcp_listener = TcpListener::bind("0.0.0.0:1716").await?;
        let tcp_fut = {
            // let certs = certs.clone();
            let device_manager = self.device_manager.clone();

            tokio::spawn(async move {
                loop {
                    match tcp_listener.accept().await {
                        Ok((mut socket, address)) => {
                            debug!("Connected from address {address:?}");
                            let certs = certs.clone();
                            let device_manager = device_manager.clone();

                            tokio::spawn(async move {
                                let identity = {
                                    let mut buf = vec![0u8; 1024];
                                    let mut data_buffer = vec![];
                                    loop {
                                        match socket.read(&mut buf).await {
                                            Ok(0) => {
                                                debug!("Connection closed by client.");
                                                break None;
                                            }
                                            Ok(n) => {
                                                let data = &buf[..n];
                                                data_buffer.extend_from_slice(data);
                                                if let Some(position) =
                                                    data_buffer.iter().position(|el| *el == b'\n')
                                                {
                                                    if let Ok(identity) =
                                                        serde_json::from_slice::<Payload>(
                                                            &data_buffer[..position],
                                                        )
                                                    {
                                                        debug!("{identity:#?}");
                                                        break Some(identity);
                                                    }
                                                    data_buffer =
                                                        Vec::from(&data_buffer[position + 1..]);
                                                }
                                            }
                                            Err(e) => {
                                                warn!("Failed to read from socket: {}", e);
                                                break None;
                                            }
                                        }
                                    }
                                };
                                if let Some(identity) = identity {
                                    let identity = serde_json::from_value::<IdentityPayloadBody>(
                                        identity.body,
                                    );
                                    match identity {
                                        Ok(identity) => {
                                            let config = ClientConfig::builder();
                                            let (cert, key) = certs;
                                            let config = config
                                                .dangerous()
                                                .with_custom_certificate_verifier(Arc::new(
                                                    NoVerifier,
                                                ))
                                                .with_client_auth_cert(
                                                    vec![cert.into()],
                                                    PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                                                        key,
                                                    )),
                                                )
                                                .unwrap();
                                            match TlsConnector::from(Arc::new(config))
                                                .connect(
                                                    ServerName::IpAddress(address.ip().into()),
                                                    socket,
                                                )
                                                .await
                                            {
                                                Ok(tls_stream) => {
                                                    let device_id = identity.device_id.clone();
                                                    let device = {
                                                        let mut device_manager =
                                                            device_manager.write().await;
                                                        device_manager.connected_to(identity)
                                                    };
                                                    match device {
                                                        Ok((tx, rx)) => {
                                                            if let Err(err) =
                                                                Self::handle_tls_stream(
                                                                    tls_stream, address, device_id,
                                                                    tx, rx,
                                                                )
                                                                .await
                                                            {
                                                                warn!("Error running tls stream {err:?}")
                                                            };
                                                        }

                                                        Err(e) => {
                                                            warn!("Cannot connect to device {e:?}")
                                                        }
                                                    }
                                                }
                                                Err(e) => error!("Cannot upgrade to tls {e:?}"),
                                            }
                                        }

                                        Err(e) => error!("Not identity payload {e:?}"),
                                    }
                                }
                            });
                        }
                        Err(err) => error!("Cannot establish tcp connection {err:?}"),
                    };
                }
            })
        };
        let gql_fut = self.run_gql(gql_port);
        tokio::pin!(gql_fut);
        futures::future::select(tcp_fut, gql_fut).await;
        Ok(())
    }

    pub async fn run_gql(&self, port: u32) -> anyhow::Result<()> {
        let schema = GQSchema::build(
            Query {
                device_manager: self.device_manager.clone(),
            },
            Mutation {
                plugin_manager: self.plugin_manager.clone(),
                device_manager: self.device_manager.clone(),
            },
            Subscription {
                plugin_manager: self.plugin_manager.clone(),
                device_manager: self.device_manager.clone(),
            },
        )
        .finish();

        let app = axum::Router::new()
            .route(
                "/",
                axum::routing::get(Html(playground_source(
                    GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
                )))
                .post_service(GraphQL::new(schema.clone())),
            )
            .route_service("/ws", GraphQLSubscription::new(schema));
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

        axum::serve(listener, app).await?;
        Ok(())
    }

    pub async fn handle_tls_stream(
        tls_stream: TlsStream<TcpStream>,
        _address: SocketAddr,
        device_id: String,
        tx: flume::Sender<(String, Payload)>,
        rx: flume::Receiver<Payload>,
    ) -> anyhow::Result<()> {
        debug!("Listening TLS for id {device_id:?}");

        let (mut read_stream, mut write_stream) = tokio::io::split(tls_stream);

        let out_sender = tokio::spawn(async move {
            while let Ok(data) = rx.recv_async().await {
                let data = serde_json::to_vec(&data);
                match data {
                    Ok(data) => {
                        write_stream.write_all(&data).await.unwrap();
                        write_stream.write_all(&[b'\n']).await.unwrap();
                    }
                    Err(er) => warn!("Cannot convert payload to json {er:?}"),
                }
            }
        });

        let out_receiver = tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            let mut data_buffer = vec![];
            while let Ok(n) = read_stream.read(&mut buf).await {
                let data = &buf[..n];
                data_buffer.extend_from_slice(data);
                if let Some(position) = data_buffer.iter().position(|el| *el == b'\n') {
                    dbg!(position);
                    dbg!(data_buffer.len());
                    if let Ok(payload) = serde_json::from_slice::<Payload>(&data_buffer[..position])
                    {
                        debug!("Received payload {payload:?}");
                        match tx.try_send((device_id.to_string(), payload)) {
                            Err(err) => warn!("Nothing to handle payload {err:?}"),
                            Ok(_) => debug!("Sent payload to channel"),
                        }
                    }
                    data_buffer = Vec::from(&data_buffer[position + 1..]);
                }
            }
        });
        futures::future::select(out_sender, out_receiver).await;
        Ok(())
    }
}
