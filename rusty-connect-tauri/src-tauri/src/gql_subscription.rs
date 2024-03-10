use api::{connection_subscription, APIEmit, AllDevices, ConnectionSubscription, API};
use clipboard::{ClipboardContext, ClipboardProvider};
use flume::Sender;
use futures::StreamExt;
use graphql_client::GraphQLQuery;

use async_tungstenite::tungstenite::{client::IntoClientRequest, http::HeaderValue};
use graphql_ws_client::graphql::StreamingOperation;
use graphql_ws_client::Client;
use tauri::{AppHandle, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, SystemTraySubmenu};
use tracing::{debug, info, warn};

pub async fn listen_to_server(port: u32, app: &AppHandle) -> anyhow::Result<()> {
    let mut request = format!("ws://localhost:{port}/ws").into_client_request()?;
    request.headers_mut().insert(
        "Sec-WebSocket-Protocol",
        HeaderValue::from_str("graphql-transport-ws")?,
    );
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    info!("Running GQL Listener");
    let (connection, _) = async_tungstenite::tokio::connect_async(request).await?;
    info!("GQL Listener Started");

    let request_client = reqwest::Client::new();

    let mut subscription = Client::build(connection)
        .subscribe(StreamingOperation::<ConnectionSubscription>::new(
            connection_subscription::Variables,
        ))
        .await?;
    while let Some(item) = subscription.next().await {
        if let Ok(response) = item {
            if let Some(data) = response.data {
                match data.payloads.payload{
                    connection_subscription::ConnectionSubscriptionPayloadsPayload::Connected(_data) => {
                        info!("Connected device, updating list {:?}", _data);
                        refresh_devices(&request_client, app.clone(), port);
                    },
                    connection_subscription::ConnectionSubscriptionPayloadsPayload::Disconnected(_data) => {
                        info!("Disconnected device, updating list {:?}", _data);
                        refresh_devices(&request_client, app.clone(), port);
                    },
                    connection_subscription::ConnectionSubscriptionPayloadsPayload::IdentityPayloadBody(_) => {},
                    connection_subscription::ConnectionSubscriptionPayloadsPayload::PairPayloadBody(_) => {
                        if let Err(err) =  (API{
                            event:api::KDEEvents::PairRequest(data.payloads.device_id),
                            ..Default::default()
                        }).emit(app,APIEmit::Event) {
                            debug!("Cant send pair request event {err:?}")
                        }
                    },
                    connection_subscription::ConnectionSubscriptionPayloadsPayload::PingPayload(_) => {},
                    connection_subscription::ConnectionSubscriptionPayloadsPayload::ClipboardPayload(data) => {
                        info!("Copying {data:?} to clipboard");
                        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                        if let Err(err)= ctx.set_contents(data.content){
                            warn!("Cant write to clipboard {err:?}")
                        }
                    },
                    connection_subscription::ConnectionSubscriptionPayloadsPayload::Payload(_) => {},
                }
            }
        }
    }
    Ok(())
}

pub fn refresh_devices(request_client: &reqwest::Client, app: AppHandle, port: u32) {
    let client = request_client.clone();
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        let response = graphql_client::reqwest::post_graphql::<AllDevices, _>(
            &client,
            format!("http://localhost:{port}"),
            api::all_devices::Variables,
        )
        .await;

        match response {
            Ok(response) => {
                if let Some(data) = response.data {
                    let tray_handle = app.tray_handle();
                    let mut system_menu = SystemTrayMenu::new()
                        .add_item(CustomMenuItem::new("devices", "Devices").disabled());
                    for device in data
                        .devices
                        .iter()
                        .filter(|d| d.is_connected && d.device.paired)
                    {
                        let menu = SystemTrayMenu::new().add_item(CustomMenuItem::new(
                            format!("{};send_clipboard", device.device.id),
                            "Send Clipboard",
                        ));
                        system_menu = system_menu
                            .add_submenu(SystemTraySubmenu::new(
                                device.device.identity.device_name.clone(),
                                menu,
                            ))
                            .add_native_item(SystemTrayMenuItem::Separator);
                    }

                    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
                    let hide = CustomMenuItem::new("open".to_string(), "Open");
                    system_menu = system_menu
                        .add_item(quit)
                        .add_native_item(SystemTrayMenuItem::Separator)
                        .add_item(hide);
                    if let Err(err) = tray_handle.set_menu(system_menu) {
                        warn!("Cannot update system tray menu {err:?}")
                    }
                    // info!("Received device list {data:?}");
                    if let Err(err) = (API {
                        devices: data.devices,
                        ..Default::default()
                    })
                    .emit(&handle, APIEmit::Devices)
                    {
                        warn!("Cannot emit devices {err:?}")
                    }
                }
            }
            Err(err) => {
                warn!("Couldnt refresh devices {err:?}")
            }
        }
    });
}
