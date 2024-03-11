use api::all_devices::DeviceWithStateFields;
use api::{KDEEvents, API};
use leptos::html::Dialog;
use leptos::logging::warn;
use leptos::{
    component, create_effect, create_memo, create_node_ref, create_resource, view, IntoView,
    ReadSignal, SignalGetUntracked,
};
use leptos::{For, SignalGet};
use log::info;
use wasm_bindgen::JsValue;

use crate::device::Device;
use crate::invoke::{invoke, pair, refresh_devices};

#[component]
pub fn DeviceList(devices: ReadSignal<Vec<DeviceWithStateFields>>) -> impl IntoView {
    let non_paired_devices = create_memo(move |_| {
        devices
            .get()
            .into_iter()
            .filter(|d| !d.device.paired && d.is_connected)
            .collect::<Vec<_>>()
    });

    let remembered_devices = create_memo(move |_| {
        devices
            .get()
            .into_iter()
            .filter(|d| d.device.paired)
            .collect::<Vec<_>>()
    });

    let (event, set_event) = API::use_event(KDEEvents::None);

    let pairing_dialog_ref = create_node_ref::<Dialog>();

    create_effect(move |_| match event.get() {
        KDEEvents::None => {}
        KDEEvents::PairRequest(_device_id) => {
            if let Some(dialog) = pairing_dialog_ref.get_untracked() {
                if let Err(err) = dialog.show_modal() {
                    warn!("Cant open dialog {err:?}")
                }
            }
        }
    });

    create_effect(move |_| {
        let devices = devices.get();
        info!("Devices updated");
    });
    view! {
        <div>

            <dialog
                _ref=pairing_dialog_ref
                class="backdrop:bg-slate-500/25 rounded-md shadow-md p-4"
            >
                <h2 class="font-bold text-xl">Pairing Requested</h2>

                {move || {
                    let device_id = event.get();
                    if let KDEEvents::PairRequest(device_id) = device_id {
                        let device = non_paired_devices
                            .get()
                            .into_iter()
                            .find(|d| d.device.id == device_id);
                        if let Some(device) = device {
                            let device_id1 = device_id.clone();
                            view! {
                                <Device device=device/>

                                <div class="flex mt-2">
                                    <div class="flex-grow"></div>
                                    <button
                                        on:click=move |_| {
                                            let device_id = device_id.clone();
                                            if let Some(dialog) = pairing_dialog_ref.get_untracked() {
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    if let Err(err) = pair(device_id, true).await {
                                                        warn!("Cant reject {err:?}")
                                                    }
                                                });
                                                dialog.close();
                                            }
                                        }

                                        class="bg-green-400 rounded-md p-2 hover:shadow-md transition-all"
                                    >
                                        Accept
                                    </button>
                                    <div class="w-2"></div>
                                    <button

                                        on:click=move |_| {
                                            let device_id = device_id1.clone();
                                            if let Some(dialog) = pairing_dialog_ref.get_untracked() {
                                                wasm_bindgen_futures::spawn_local(async move {
                                                    if let Err(err) = pair(device_id, false).await {
                                                        warn!("Cant reject {err:?}")
                                                    }
                                                });
                                                dialog.close();
                                            }
                                        }

                                        class="bg-red-400 rounded-md p-2 hover:shadow-md transition-all"
                                    >
                                        Reject
                                    </button>
                                </div>
                            }
                                .into_view()
                        } else {
                            view! { "No device found with given id" }.into_view()
                        }
                    } else {
                        view! { <p>{format!("{device_id:?}")}</p> }.into_view()
                    }
                }}

            </dialog>
            <div class="flex mb-2">
                <h2 class="font-bold text-2xl flex-grow">"Devices"</h2>
                <button
                    on:click=move |_ev| {
                        wasm_bindgen_futures::spawn_local(async move {
                            refresh_devices().await;
                        })
                    }

                    class="bg-purple-300 rounded-md p-2 hover:shadow-md transition-all"
                >
                    Refresh
                </button>
            </div>
            <hr/>
            <h2 class="font-medium text-lg mt-2">
                "Paired Devices" " (" {move || remembered_devices.get().len()} ")"
            </h2>
            <For
                each=move || remembered_devices.get()
                key=|device| {
                    format!("{}{}{}", device.device.id, device.is_connected, device.device.paired)
                }

                children=move |device| {
                    view! { <Device device=device/> }
                }
            />

            <hr/>
            <h2 class="font-medium text-lg mt-2">
                "Available Devices" " (" {move || non_paired_devices.get().len()} ")"
            </h2>
            <For
                each=move || non_paired_devices.get()
                key=|device| device.device.id.clone()
                children=move |device| {
                    view! { <Device device=device/> }
                }
            />

        </div>
    }
}

