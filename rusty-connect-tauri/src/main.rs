mod app;
mod components;
mod device;
mod device_list;
mod invoke;

use app::*;
use leptos::*;

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|| {
        view! { <App/> }
    })
}

