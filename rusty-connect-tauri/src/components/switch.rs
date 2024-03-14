use std::{borrow::Cow, fmt::Display};

use leptos::{component, view, IntoView};

#[component]
pub fn Switch(state: bool, label: Cow<'static, str>) -> impl IntoView {
    view! {
        <label for="toggleB" class="flex items-center cursor-pointer">
            <div class="mr-3 text-gray-700 font-medium flex-grow">{label}</div>
            <div class="relative">
                <input checked=state type="checkbox" id="toggleB" class="sr-only"/>
                <div class="block bg-gray-600 w-7 h-4 rounded-full"></div>
                <div class=format!(
                    "dot absolute left-0.5 top-0.5 bg-white w-3 h-3 rounded-full transition {}",
                    if state { "translate-x-full" } else { "" },
                )></div>
            </div>
        </label>
    }
}

