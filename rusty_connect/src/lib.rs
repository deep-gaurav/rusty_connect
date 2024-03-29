use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::response::Html;

use cert::certgen::generate_cert;
use cert::CertPair;
use devices::DeviceManager;

use mdns_sd::ServiceInfo;
use payloads::PayloadType;
use plugins::{PluginManager, ReceivedPayload};
use schema::subscription::Subscription;
use schema::{mutation::Mutation, query::Query, GQSchema};
use socket2::TcpKeepalive;
use tokio_native_tls::native_tls::{self, Certificate, Identity, TlsAcceptor};
use tokio_native_tls::TlsStream;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

use tokio::sync::RwLock;

use tracing::{debug, error, info, warn};

use crate::payloads::{IdentityPayloadBody, Payload};

pub mod cert;
pub mod devices;
pub mod network;
pub mod payloads;
pub mod plugins;
pub mod schema;
pub mod utils;

pub struct RustyConnect {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub cert: Vec<u8>,
    pub key: Vec<u8>,
    pub plugin_manager: Arc<PluginManager>,
    pub device_manager: Arc<RwLock<DeviceManager>>,
}

impl RustyConnect {
    pub async fn new(
        id: &str,
        name: &str,
        device_type: &str,
        data_folder: &Path,
    ) -> anyhow::Result<Self> {
        tokio::fs::create_dir_all(data_folder).await?;
        let cert_path = data_folder.join("cert.pem");
        let key_path = data_folder.join("key.pem");
        let certs = match generate_cert(id, &cert_path, &key_path).await {
            Ok(certs) => certs,
            Err(err) => {
                warn!("Error generating cert {err:?}");
                return Err(err);
            }
        };

        let (tx, rx) = flume::bounded(0);
        let device_manager =
            DeviceManager::load_or_create(data_folder, tx, rx, certs.clone()).await?;
        let plugin_manager = PluginManager::new(
            id.to_string(),
            name.to_string(),
            device_type.to_string(),
            &device_manager,
        );
        Ok(Self {
            id: id.to_string(),
            name: name.to_string(),
            device_type: device_type.to_string(),
            cert: certs.0,
            key: certs.1,
            plugin_manager: Arc::new(plugin_manager),
            device_manager: Arc::new(RwLock::new(device_manager)),
        })
    }

    pub async fn run(&mut self, gql_port: u32) -> anyhow::Result<()> {
        debug!("Starting RustyConnect on port 1716 and GQL on {gql_port}");
        let certs = (self.cert.clone(), self.key.clone());
        let tcp_listener = TcpListener::bind("0.0.0.0:1716").await?;
        let tcp_fut = {
            // let certs = certs.clone();
            let device_manager = self.device_manager.clone();
            let plugin_manager = self.plugin_manager.clone();

            async move {
                info!("Waiting for device");
                loop {
                    match tcp_listener.accept().await {
                        Ok((socket, address)) => {
                            let mut socket = match network::set_tcp_timeouts(socket) {
                                Ok(stream) => stream,
                                Err(err) => {
                                    warn!("Cannot set keepalives {err:?} disconnecting");
                                    continue;
                                }
                            };
                            info!("Connected from address {address:?}");
                            let certs = certs.clone();
                            let device_manager = device_manager.clone();
                            let plugin_manager = plugin_manager.clone();

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
                                            let cert = match Certificate::from_pem(&certs.0) {
                                                Ok(cert) => cert,
                                                Err(err) => {
                                                    warn!("Cannot generate cert {err:?}");
                                                    return;
                                                }
                                            };
                                            info!("got certificate ");
                                            let key = match Identity::from_pkcs8(&certs.0, &certs.1)
                                            {
                                                Ok(identity) => identity,
                                                Err(err) => {
                                                    warn!("Cannot generate identity from certs {err:?}");
                                                    return;
                                                }
                                            };
                                            let connector = native_tls::TlsConnector::builder()
                                                .danger_accept_invalid_certs(true)
                                                .identity(key)
                                                .build();
                                            let connector = match connector {
                                                Ok(connector) => {
                                                    tokio_native_tls::TlsConnector::from(connector)
                                                }
                                                Err(err) => {
                                                    warn!("Cannot accept cert {err:?}");
                                                    return;
                                                }
                                            };
                                            match connector
                                                .connect(&address.to_string(), socket)
                                                .await
                                            {
                                                Ok(tls_stream) => {
                                                    let device_id = identity.device_id.clone();
                                                    let device = {
                                                        let mut device_manager =
                                                            device_manager.write().await;
                                                        device_manager
                                                            .connected_to(address, identity)
                                                            .await
                                                    };
                                                    match device {
                                                        Ok((tx, rx, connection_id)) => {
                                                            if let Err(err) =
                                                                Self::handle_tls_stream(
                                                                    tls_stream,
                                                                    address,
                                                                    device_id.clone(),
                                                                    tx,
                                                                    rx,
                                                                    plugin_manager.clone(),
                                                                    device_manager.clone(),
                                                                )
                                                                .await
                                                            {
                                                                warn!("Error running tls stream {err:?}")
                                                            };

                                                            {
                                                                info!("Disconnecting device {device_id}");
                                                                let mut device_manager =
                                                                    device_manager.write().await;
                                                                if let Err(err) = device_manager
                                                                    .disconnect(
                                                                        &device_id,
                                                                        &connection_id,
                                                                    )
                                                                {
                                                                    warn!("Error disconnecting {err:?}")
                                                                }
                                                                info!("Disconnected device {device_id}");
                                                            }
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
            }
        };
        let gql_fut = self.run_gql(gql_port);
        let tx = { self.device_manager.read().await.sender.clone() };
        let broadcast_listener = self.listen_to_broadcast(1716, tx);

        tokio::pin!(gql_fut, tcp_fut, broadcast_listener);
        futures::future::select(
            tcp_fut,
            futures::future::select(gql_fut, broadcast_listener),
        )
        .await;
        Ok(())
    }

    async fn run_gql(&self, port: u32) -> anyhow::Result<()> {
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
        .data(self.device_manager.clone())
        .data(self.plugin_manager.clone())
        .finish();

        let icons_path = { self.device_manager.read().await.icons_path.clone() };
        let icons_server = tower_http::services::ServeDir::new(icons_path);
        let app = axum::Router::new()
            .route(
                "/",
                axum::routing::get(Html(playground_source(
                    GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"),
                )))
                .post_service(GraphQL::new(schema.clone())),
            )
            .nest_service("/icons", icons_server)
            .route_service("/ws", GraphQLSubscription::new(schema));
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

        axum::serve(listener, app).await?;
        Ok(())
    }

    async fn listen_to_broadcast(
        &self,
        kde_port: u16,
        _tx: flume::Sender<PayloadType>,
    ) -> anyhow::Result<()> {
        let socket = UdpSocket::bind(format!("0.0.0.0:{kde_port}")).await?;
        socket.set_broadcast(true)?;

        const SERVICE_NAME: &str = "_kdeconnect._udp.local.";
        let service_daemon = mdns_sd::ServiceDaemon::new()?;
        let receive = service_daemon.browse(SERVICE_NAME)?;

        let identity_body = self
            .plugin_manager
            .get_identity_payload_body(Some(kde_port));

        let identity = self.plugin_manager.get_identity_payload(Some(kde_port))?;
        let host_name = {
            let host_name = hostname::get()?;
            let host_name = host_name
                .to_str()
                .ok_or(anyhow::anyhow!("Host name not valid"))?;
            if host_name.ends_with('.') {
                host_name.to_string()
            } else if host_name.ends_with(".local") {
                format!("{host_name}.")
            } else {
                format!("{host_name}.local.")
            }
        };

        let service_info = ServiceInfo::new(
            SERVICE_NAME,
            &identity_body.device_id,
            &host_name,
            "192.168.53.117",
            1716,
            [
                ("id", identity_body.device_id.clone()),
                ("name", identity_body.device_name.clone()),
                ("type", identity_body.device_type.clone()),
                ("protocol", identity_body.protocol_version.to_string()),
            ]
            .as_slice(),
        )?;
        service_daemon.register(service_info)?;

        tokio::spawn(async move {
            info!("Waiting for mdns");
            while let Ok(event) = receive.recv_async().await {
                match event {
                    mdns_sd::ServiceEvent::ServiceResolved(info) => {
                        info!("Service info {info:#?}");
                        let port = info.get_port();
                        let address = info.get_addresses_v4().into_iter().next();
                        if let Some(address) = address {
                            let Ok(udpsock) = UdpSocket::bind("0.0.0.0:0").await else {
                                continue;
                            };
                            if let Err(err) = udpsock.set_broadcast(true) {
                                warn!("cant set udp broadcast {err:?}");
                                continue;
                            };

                            let Ok(mut payload_bytes) = serde_json::to_vec(&identity) else {
                                continue;
                            };
                            payload_bytes.append(&mut b"\n".to_vec());
                            let advertise_addr =
                                SocketAddr::new(std::net::IpAddr::V4(*address), port);
                            info!("Sending info to socket addr {advertise_addr:?}");
                            if let Err(err) = udpsock.send_to(&payload_bytes, advertise_addr).await
                            {
                                warn!("Cant send to mdns user {err:?}");
                            }
                        }
                    }
                    _other_event => {
                        info!("Received other mdns service event {_other_event:?}")
                    }
                }
            }
            info!("Exited mdns")
        });
        let mut buf = Vec::with_capacity(1024 * 512);
        info!("Waiting from broadcast");
        let device_id = self.plugin_manager.device_id.clone();
        while let Ok((n, address)) = socket.recv_buf_from(&mut buf).await {
            info!("Receiving from udp {n}");
            let data = &buf[..n];
            info!("Received udp from {address:?}");
            if let Ok(payload) = serde_json::from_slice::<Payload>(data) {
                let identity = serde_json::from_value::<IdentityPayloadBody>(payload.body.clone());
                if let Ok(identity) = identity {
                    if identity.device_id != device_id {
                        if let Some(port) = identity.tcp_port {
                            let self_identity = self
                                .plugin_manager
                                .get_identity_payload_body(Some(kde_port));
                            let certs = (self.cert.clone(), self.key.clone());
                            let device_id = identity.device_id.clone();
                            let device = {
                                let mut device_manager = self.device_manager.write().await;
                                device_manager.connected_to(address, identity).await
                            };
                            match device {
                                Ok((tx, rx, connection_id)) => {
                                    let dm = self.device_manager.clone();
                                    let pm = self.plugin_manager.clone();
                                    tokio::spawn(async move {
                                        if let Err(err) = Self::connect_to(
                                            address,
                                            port,
                                            self_identity,
                                            certs,
                                            device_id.clone(),
                                            tx,
                                            rx,
                                            pm,
                                            dm.clone(),
                                        )
                                        .await
                                        {
                                            warn!("Error running tls stream {err:?}")
                                        };

                                        {
                                            info!("Disconnecting device {device_id}");
                                            let mut device_manager = dm.write().await;
                                            if let Err(err) = device_manager
                                                .disconnect(&device_id, &connection_id)
                                            {
                                                warn!("Error disconnecting {err:?}")
                                            }
                                            info!("Disconnected device {device_id}");
                                        }
                                    });
                                }
                                Err(e) => {
                                    warn!("Cannot connect to device {e:?}")
                                }
                            }
                        }
                    } else {
                        info!("Ignoring self discovery")
                    }
                } else {
                    info!("Non identity payload not supported")
                }
            } else {
                match std::str::from_utf8(data) {
                    Ok(data) => warn!("{data}"),
                    Err(err) => warn!("{err:?}"),
                }
                warn!("Not valid payload")
            }
            buf.clear();
        }

        info!("Broadcast receiver stopped");
        Ok(())
    }

    pub async fn handle_tls_stream(
        tls_stream: TlsStream<TcpStream>,
        _address: SocketAddr,
        device_id: String,
        tx: flume::Sender<PayloadType>,
        rx: flume::Receiver<Payload>,

        plugin_manager: Arc<PluginManager>,
        device_manager: Arc<RwLock<DeviceManager>>,
    ) -> anyhow::Result<()> {
        debug!("Listening TLS for id {device_id:?}");

        let (mut read_stream, mut write_stream) = tokio::io::split(tls_stream);

        let out_sender = async move {
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
        };

        let out_receiver = async move {
            let mut buf = vec![0u8; 1024];
            let mut data_buffer = vec![];
            while let Ok(n) = read_stream.read(&mut buf).await {
                if n == 0 {
                    info!("Read 0, disconnected");
                    break;
                }
                let data = &buf[..n];
                data_buffer.extend_from_slice(data);
                if let Some(position) = data_buffer.iter().position(|el| *el == b'\n') {
                    // if let Ok(payload) =
                    //     serde_json::from_slice::<serde_json::Value>(&data_buffer[..position])
                    // {
                    //     info!("Received payload {payload:#?}");

                    // }
                    match serde_json::from_slice::<Payload>(&data_buffer[..position]) {
                        Ok(payload) => {
                            let tx = tx.clone();
                            let device_id = device_id.clone();
                            let plugin_manager = plugin_manager.clone();
                            let device_manager = device_manager.clone();
                            tokio::spawn(async move {
                                match Self::process_payload(
                                    &device_id,
                                    payload,
                                    plugin_manager,
                                    device_manager,
                                )
                                .await
                                {
                                    Ok(payload) => {
                                        match tx.try_send((device_id.to_string(), payload)) {
                                            Err(err) => warn!("Nothing to handle payload {err:?}"),
                                            Ok(_) => debug!("Sent payload to channel"),
                                        }
                                    }
                                    Err(e) => warn!("Error processing payload {e:#?}"),
                                }
                            });
                        }
                        Err(err) => {
                            warn!("parse failed {err:#?}")
                        }
                    }
                    data_buffer = Vec::from(&data_buffer[position + 1..]);
                }
            }
            info!("TCP Disconnected")
        };

        tokio::pin!(out_sender, out_receiver);
        futures::future::select(out_sender, out_receiver).await;
        Ok(())
    }

    async fn connect_to(
        address: SocketAddr,
        port: u16,
        identity: IdentityPayloadBody,
        certs: CertPair,

        device_id: String,
        tx: flume::Sender<PayloadType>,
        rx: flume::Receiver<Payload>,

        plugin_manager: Arc<PluginManager>,
        device_manager: Arc<RwLock<DeviceManager>>,
    ) -> anyhow::Result<()> {
        let stream = TcpStream::connect(SocketAddr::new(address.ip(), port)).await?;
        let mut stream = network::set_tcp_timeouts(stream)?;
        let value = serde_json::to_value(identity.clone())?;
        let identity_payload = Payload::generate_new("kdeconnect.identity", value);
        let idenity_bytes = serde_json::to_vec(&identity_payload)?;
        let _sent = stream.write(&idenity_bytes).await?;
        stream.write_all(&[b'\n']).await?;
        stream.flush().await?;

        // let cert = Certificate::from_pem(&certs.0)?;

        let identity = Identity::from_pkcs8(&certs.0, &certs.1)?;

        let tls_acceptor = TlsAcceptor::builder(identity).build()?;

        // let config = ServerConfig::builder();
        // let (cert, key) = certs;
        // let config = config
        //     .with_client_cert_verifier(Arc::new(NoVerifier))
        //     .with_single_cert_with_ocsp(
        //         vec![cert.into()],
        //         PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key)),
        //         vec![],
        //     )?;
        // let tls_connector = TlsAcceptor::from(Arc::new(config));

        info!("Upgrading to TLS Stream as server");

        let tls_acceptor = tokio_native_tls::TlsAcceptor::from(tls_acceptor);
        let tls_stream = tls_acceptor.accept(stream).await;
        let tls_stream = match tls_stream {
            Ok(stream) => stream,
            Err(err) => {
                warn!("couldnt upgrade tls {err:?}");
                return Err(err.into());
            }
        };
        info!("Upgraded to TLS Stream");

        Self::handle_tls_stream(
            tls_stream,
            address,
            device_id,
            tx,
            rx,
            plugin_manager,
            device_manager,
        )
        .await?;
        Ok(())
    }

    async fn process_payload(
        device_id: &str,
        payload: Payload,
        plugin_manager: Arc<PluginManager>,
        device_manager: Arc<RwLock<DeviceManager>>,
    ) -> anyhow::Result<ReceivedPayload> {
        let device = {
            let devices = device_manager.read().await;
            devices
                .devices
                .get(device_id)
                .ok_or(anyhow::anyhow!("No device with given Id"))?
                .clone()
        };
        debug!("parsing payload");
        let payload = plugin_manager.parse_payload(payload, Some(&device)).await?;
        debug!("parsed payload");
        {
            let mut dm = device_manager.write().await;
            if let Some(device) = dm.devices.get_mut(device_id) {
                plugin_manager.update_state(&payload, device);
            }
        }
        debug!("emitting payload");
        Ok(payload)
    }
}
