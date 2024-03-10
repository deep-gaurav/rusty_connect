use api::all_devices::DeviceWithStateFields;
use leptos::{component, prelude::*, view, IntoView};

#[component]
pub fn Device(device: DeviceWithStateFields) -> impl IntoView {
    view! {
        <div class="p-4 shadow-md border rounded-md flex items-center  mt-2">
            <i class="fa-solid fa-mobile fa-2xl"></i>
            <div class="w-2"></div>
            <div class="flex-grow flex flex-col">
                <div>{device.device.identity.device_name}</div>
                <div class="text-xs">{device.device.identity.device_id}</div>
            </div>
        </div>
    }
}

