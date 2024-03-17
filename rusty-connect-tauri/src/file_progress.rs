use std::path::PathBuf;

use api::{
    all_devices::IdentityPayloadFields, download_progress::DownloadProgressDownloadUpdate, API,
};
use leptos::{component, view, IntoView, SignalGet};

#[component]
pub fn ProgressNotificationToaster() -> impl IntoView {
    let (download, set_download) = API::use_download_progress(None);

    view! {
        <div class="absolute right-0 top-0">

            {move || {
                if let Some(download) = download.get() {
                    view! { <DownloadProgress device=download.1 update=download.2/> }.into_view()
                } else {
                    view! {}.into_view()
                }
            }}

        </div>
    }
}

#[component]
pub fn DownloadProgress(
    device: IdentityPayloadFields,
    update: DownloadProgressDownloadUpdate,
) -> impl IntoView {
    view! {
        <div class="mt-2 mr-2 bg-slate-100 p-4 rounded-md border shadow-md flex items-center">
            <div class="mr-2">
                <i class="fa-solid fa-download fa-2xl"></i>
            </div>
            <div>

                {match update {
                    DownloadProgressDownloadUpdate::NotStarted(_) => view! {}.into_view(),
                    DownloadProgressDownloadUpdate::Progress(data) => {
                        let received = humansize::format_size_i(
                            data.read_bytes,
                            humansize::DECIMAL,
                        );
                        let total = humansize::format_size_i(data.total_bytes, humansize::DECIMAL);
                        view! {
                            <div class="font-semibold text-md">
                                Receiving File: {" "} {device.device_name}
                            </div>
                            <div class="w-full bg-gray-200 rounded-full h-2.5 dark:bg-gray-700">
                                <div
                                    class="bg-blue-600 h-2.5 rounded-full"
                                    style=format!(
                                        "width: {}%",
                                        data.read_bytes * 100 / data.total_bytes,
                                    )
                                >
                                </div>
                            </div>
                            <div class="text-sm">{format!("File Size: {received}/{total}")}</div>
                        }
                            .into_view()
                    }
                    DownloadProgressDownloadUpdate::Completed(data) => {
                        let size = humansize::format_size_i(data.total_bytes, humansize::DECIMAL);
                        view! {
                            <div class="font-semibold text-md">Received File</div>
                            <div>
                                {if let Some(name) = PathBuf::from(data.path).file_name() {
                                    name.to_string_lossy().to_string()
                                } else {
                                    "Unknown".to_string()
                                }}

                            </div>
                            <div class="text-sm">{format!("File Size: {size}")}</div>
                        }
                            .into_view()
                    }
                    DownloadProgressDownloadUpdate::DownloadFailed(_) => view! {}.into_view(),
                }}

            </div>
        </div>
    }
}

