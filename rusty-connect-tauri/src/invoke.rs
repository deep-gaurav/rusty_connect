use std::error::Error;

use js_sys::wasm_bindgen;
use serde::Serialize;
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    pub async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

pub async fn refresh_devices() -> JsValue {
    invoke("refresh_devices", JsValue::NULL).await
}

pub async fn pair(device_id: String, pair: bool) -> Result<JsValue, Box<dyn Error>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PairArgs {
        device_id: String,
        pair: bool,
    }
    let args = PairArgs { device_id, pair };
    Ok(invoke("pair", to_value(&args)?).await)
}


pub async fn send_ping(device_id: Option<String>)-> Result<JsValue, Box<dyn Error>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SendPingArgs {
        device_id: Option<String>,
    }
    let args = SendPingArgs { device_id };
    Ok(invoke("send_ping", to_value(&args)?).await)
}

pub async fn send_clipboard(device_id: Option<String>)-> Result<JsValue, Box<dyn Error>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SendClipboardArgs {
        device_id: Option<String>,
    }
    let args = SendClipboardArgs { device_id };
    Ok(invoke("send_clipboard", to_value(&args)?).await)
}

