use api::all_devices::{
    DeviceFieldsPluginConfigsBatttery, DeviceFieldsPluginConfigsClipboard,
    DeviceFieldsPluginConfigsNotification, DeviceFieldsPluginConfigsPing,
    DeviceFieldsPluginConfigsShare, DeviceWithStateFields,
};
use leptos::{component, create_resource, logging::warn, prelude::*, use_context, view, IntoView};
use leptos_router::{use_navigate, use_params_map};
use log::info;
use std::rc::Rc;

use crate::{components::switch::Switch, invoke::send_ping};

#[component]
pub fn DeviceTile(device: DeviceWithStateFields) -> impl IntoView {
    let navigate = use_navigate();
    view! {
        <div
            class="p-4 bg-red shadow-md border rounded-md flex items-center mt-2 cursor-pointer dark:shadow-slate-700"
            on:click=move |_| {
                info!("Clicked device {}", device.device.id);
                navigate(&device.device.id.to_string(), Default::default())
            }
        >

            <i class=format!(
                "fa-solid fa-mobile fa-2xl {}",
                if device.is_connected { "text-green-600" } else { "text-grey-300" },
            )></i>
            <div class="w-2"></div>
            <div class="flex-grow flex flex-col">
                <div>{device.device.identity.device_name}</div>
                <div class="text-xs">{device.device.identity.device_id}</div>
            </div>
        </div>
    }
}

#[component]
pub fn DevicePage() -> impl IntoView {
    let params = use_params_map();
    let devices = use_context::<ReadSignal<Vec<DeviceWithStateFields>>>().unwrap();
    let device = create_memo(move |_| {
        let params = params.get();
        let device_id = params.get("id");
        if let Some(device_id) = device_id {
            devices
                .get()
                .into_iter()
                .find(|d| &d.device.id == device_id)
        } else {
            None
        }
    });
    view! {
        <div class="flex-grow">

            {move || match device.get() {
                Some(device) => view! { <Device device=device/> }.into_view(),
                None => view! {}.into_view(),
            }}

        </div>
    }
}

#[component]
fn Device(device: DeviceWithStateFields) -> impl IntoView {
    view! {
        <div class="rounded-md bg-slate-200 dark:bg-slate-600 dark:text-slate-300 w-full h-full p-4 overflow-auto">
            <div class="flex items-center">
                <h2 class="font-bold text-2xl">{&device.device.identity.device_name}</h2>
                <div class="w-4"></div>
                <div class="flex items-center dark:bg-slate-800 bg-slate-50 p-2 rounded-full flex-shrink">
                    <div class=format!(
                        "w-2 h-2 rounded-full {}",
                        {
                            move || match device.is_connected {
                                true => "bg-green-900",
                                false => "bg-red-900",
                            }
                        }(),
                    )></div>
                    <div class="w-2"></div>
                    <div class="text-xs">
                        {move || match device.is_connected {
                            true => "Connected",
                            false => "Disconnected",
                        }}

                    </div>
                </div>
            </div>
            <div class="h-2"></div>
            <div class="w-full flex flex-wrap">
                <div class="p-4 min-w-40 shadow-lg rounded-lg bg-slate-50 border m-4">
                    <i class="fa-solid fa-hands-clapping"></i>
                    <div class="text-lg font-medium">Ping</div>

                    {
                        let config = {
                            if let Some(ping_config) = &device.device.plugin_configs.ping {
                                ping_config.clone()
                            } else {
                                DeviceFieldsPluginConfigsPing {
                                    enabled: true,
                                    send_enabled: true,
                                }
                            }
                        };
                        view! {
                            <Switch state=config.enabled label="Receive Enabled".into()/>
                            <Switch state=config.send_enabled label="Send Enabled".into()/>
                        }
                    }

                    <button
                        on:click={
                            let device_id = device.device.id.clone();
                            move |_| {
                                let device_id = device_id.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    if let Err(err) = send_ping(device_id.into()).await {
                                        warn!("Cant reject {err:?}")
                                    }
                                });
                            }
                        }

                        class="border text-xs p-2 rounded-md mt-2 shadow-sm bg-slate-100 font-semibold"
                    >
                        <i class="fa-solid fa-paper-plane mr-2"></i>
                        Send Ping
                    </button>

                </div>

                <div class="p-4 min-w-40 shadow-lg rounded-lg bg-slate-50 border m-4">
                    <i class="fa-solid fa-clipboard"></i>
                    <div class="text-lg font-medium">Clipboard</div>

                    {
                        let config = {
                            if let Some(clipboard_config) = device.device.plugin_configs.clipboard {
                                clipboard_config
                            } else {
                                DeviceFieldsPluginConfigsClipboard {
                                    enabled: true,
                                    send_enabled: true,
                                }
                            }
                        };
                        view! {
                            <Switch state=config.enabled label="Receive Enabled".into()/>
                            <Switch state=config.send_enabled label="Send Enabled".into()/>
                        }
                    }

                    <button
                        on:click={
                            let device_id = device.device.id.clone();
                            move |_| {
                                let device_id = device_id.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    if let Err(err) = send_ping(device_id.into()).await {
                                        warn!("Cant reject {err:?}")
                                    }
                                });
                            }
                        }

                        class="border text-xs p-2 rounded-md mt-2 shadow-sm bg-slate-100 font-semibold"
                    >
                        <i class="fa-solid fa-paste mr-2"></i>
                        Send Clipboard
                    </button>
                </div>

                <div class="p-4 min-w-40 shadow-lg rounded-lg bg-slate-50 border m-4">
                    <i class="fa-solid fa-battery-empty"></i>
                    <div class="text-lg font-medium">Battery</div>

                    {
                        let config = {
                            if let Some(battery_config) = device.device.plugin_configs.batttery {
                                battery_config
                            } else {
                                DeviceFieldsPluginConfigsBatttery {
                                    enabled: true,
                                    send_enabled: true,
                                }
                            }
                        };
                        view! {
                            <Switch state=config.enabled label="Receive Enabled".into()/>
                            <Switch state=config.send_enabled label="Send Enabled".into()/>
                        }
                    }

                </div>

                <div class="p-4 min-w-40 shadow-lg rounded-lg bg-slate-50 border m-4">
                    <i class="fa-solid fa-bell"></i>
                    <div class="text-lg font-medium">Notification</div>

                    {
                        let config = {
                            if let Some(notification_config) = device
                                .device
                                .plugin_configs
                                .notification
                            {
                                notification_config
                            } else {
                                DeviceFieldsPluginConfigsNotification {
                                    enabled: true,
                                }
                            }
                        };
                        view! { <Switch state=config.enabled label="Receive Enabled".into()/> }
                    }

                </div>

                <div class="p-4 min-w-40 shadow-lg rounded-lg bg-slate-50 border m-4">
                    <i class="fa-solid fa-share"></i>
                    <div class="text-lg font-medium">Share</div>

                    {
                        let config = {
                            if let Some(share_config) = device.device.plugin_configs.share {
                                share_config
                            } else {
                                DeviceFieldsPluginConfigsShare {
                                    enabled: true,
                                }
                            }
                        };
                        view! { <Switch state=config.enabled label="Receive Enabled".into()/> }
                    }

                </div>
            </div>
        </div>
    }
}

