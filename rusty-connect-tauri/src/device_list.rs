use api::all_devices::DeviceWithStateFields;
use leptos::{component, create_memo, view, IntoView, ReadSignal};
use leptos::{For, SignalGet};

use crate::device::Device;

#[component]
pub fn DeviceList(devices: ReadSignal<Vec<DeviceWithStateFields>>) -> impl IntoView {
    let non_paired_devices = create_memo(move |_| {
        devices
            .get()
            .into_iter()
            .filter(|d| !d.device.paired)
            .collect::<Vec<_>>()
    });

    let remembered_devices = create_memo(move |_| {
        devices
            .get()
            .into_iter()
            .filter(|d| d.device.paired)
            .collect::<Vec<_>>()
    });
    view! {
        <div>
            <h2 class="font-bold text-2xl">"Devices"</h2>
            <hr/>
            <h2 class="font-medium text-lg mt-2">
                "Remembered Devices" " (" {move || remembered_devices.get().len()} ")"
            </h2>

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

