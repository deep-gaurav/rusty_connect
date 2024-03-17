use api::{all_devices::DeviceWithStateFields, API};
use leptos::leptos_dom::ev::SubmitEvent;
use leptos::*;
use leptos_router::{Route, Router, Routes};
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::{
    device::{DevicePage, DeviceTile}, device_list::DeviceList, file_progress::ProgressNotificationToaster, invoke::{invoke, refresh_devices}
};

#[component]
pub fn App() -> impl IntoView {
    let (devices, set_devices) = API::use_devices(vec![]);

    provide_context(devices);
    create_effect(|_| {
        wasm_bindgen_futures::spawn_local(async move {
            refresh_devices().await;
        })
    });

    view! {
        <main class="bg-slate-50 dark:bg-gray-950 h-full p-4 dark:text-slate-400">
            <ProgressNotificationToaster/>
            <Router>
                <Routes>
                    <Route
                        path=""
                        view=move || {
                            view! { <DeviceList devices=devices/> }
                        }
                    >

                        <Route path=":id" view=DevicePage/>
                        <Route
                            path=""
                            view=move || {
                                view! {}
                            }
                        />

                    </Route>
                </Routes>
            </Router>
        </main>
    }
}



























