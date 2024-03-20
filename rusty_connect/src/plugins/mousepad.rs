use std::{net::SocketAddr, sync::Arc};

use async_graphql::{Context, Object, SimpleObject};
use enigo::{Key, KeyboardControllable, MouseControllable};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use super::{Plugin, PluginExt};

pub struct Mousepad {
    // enigo: Arc<enigo::Enigo>,
}

static SPECIAL_KEYS_MAP: phf::Map<u32, Key> = phf::phf_map! {
    1_u32 => Key::Backspace,
    2_u32 => Key::Tab,

    4_u32 => Key::LeftArrow,
    5_u32 => Key::UpArrow,
    6_u32 => Key::RightArrow,
    7_u32 => Key::DownArrow,

    8_u32 => Key::PageUp,
    9_u32 => Key::PageDown,

    10_u32 => Key::Home,
    11_u32 => Key::End,

    12_u32 => Key::Return,

    13_u32 => Key::Delete,

    14_u32 => Key::Escape,

    //15_u32 => Key::SysReq,

    // 16_u32 => Key::ScrollLock,

    21_u32 => Key::F1,
    22_u32 => Key::F2,
    23_u32 => Key::F3,
    24_u32 => Key::F4,
    25_u32 => Key::F5,
    26_u32 => Key::F6,
    27_u32 => Key::F7,
    28_u32 => Key::F8,
    29_u32 => Key::F9,
    30_u32 => Key::F10,
    31_u32 => Key::F11,
    32_u32 => Key::F12,
};

#[Object]
impl Mousepad {
    pub async fn send_input<'ctx>(&self, context: &Context<'ctx>) -> anyhow::Result<&str> {
        Ok("failed")
    }
}

impl Plugin for Mousepad {
    type PluginPayload = MousepadPayload;
    type PluginConfig = MousepadConfig;
    type PluginState = MousepadState;

    fn init(_device_mangager: &crate::devices::DeviceManager) -> Self {
        Self {
            // enigo: enigo::Enigo::new(),
        }
    }

    fn incoming_capabilities(&self) -> Vec<String> {
        vec!["kdeconnect.mousepad.request".to_string()]
    }

    fn outgoing_capabilities(&self) -> Vec<String> {
        vec![
            // "kdeconnect.mousepad.keyboardstate".to_string()
        ]
    }

    async fn parse_payload(
        &self,
        payload: &crate::payloads::Payload,
        _address: SocketAddr,
    ) -> Option<Self::PluginPayload> {
        if payload.r#type == "kdeconnect.mousepad.request" {
            // info!("Cannot parse mounse event {payload:#?}")

            let payload = serde_json::from_value::<Self::PluginPayload>(payload.body.clone());
            match payload {
                Ok(payload) => {
                    info!("Parsed payload {payload:?}");
                    return Some(payload);
                }
                Err(err) => {
                    warn!("Error parsing mouse payload {err:?}");
                }
            }
        }
        None
    }

    fn is_enabled(&self, config: &Option<Self::PluginConfig>) -> bool {
        if let Some(config) = config {
            config.enabled
        } else {
            true
        }
    }

    fn should_send(
        &self,
        _config: &Option<Self::PluginConfig>,
        _state: &mut Self::PluginState,
        _payload: &Self::PluginPayload,
    ) -> bool {
        false
    }

    fn update_state(&self, _payload: &Self::PluginPayload, _state: &mut Self::PluginState) {
        // info!("handling mouse event {_payload:?} {_state:?}");

        if _state.handle_mouse_events {
            let mut mouse = enigo::Enigo::new();
            if _payload.singleclick == Some(true) {
                mouse.mouse_click(enigo::MouseButton::Left);
            }
            if _payload.rightclick == Some(true) {
                mouse.mouse_click(enigo::MouseButton::Right);
            }
            if _payload.middleclick == Some(true) {
                mouse.mouse_click(enigo::MouseButton::Middle);
            }
            if _payload.doubleclick == Some(true) {
                mouse.mouse_click(enigo::MouseButton::Left);
                mouse.mouse_click(enigo::MouseButton::Left);
            }
            if _payload.singlehold == Some(true) {
                mouse.mouse_down(enigo::MouseButton::Left);
            }
            if let (Some(dx), Some(dy)) = (_payload.dx, _payload.dy) {
                if _payload.scroll == Some(true) {
                    mouse.mouse_scroll_x(dx as i32);
                    mouse.mouse_scroll_y(dy as i32);
                } else {
                    mouse.mouse_move_relative(dx as i32, dy as i32);
                }
            }
        }
        if _state.handle_keyboard_events {
            let mut keyboard = enigo::Enigo::new();
            let mut modifiers = vec![];
            if _payload.alt == Some(true) {
                modifiers.push(Key::Alt);
            }
            if _payload.shift == Some(true) {
                modifiers.push(Key::Shift);
            }
            if _payload.ctrl == Some(true) {
                modifiers.push(Key::Control);
            }
            if let Some(special) = &_payload.special_key {
                let key = SPECIAL_KEYS_MAP.get(special);
                if let Some(key) = key {
                    modifiers.push(*key);
                }
            }
            modifiers.iter().for_each(|key| keyboard.key_down(*key));

            if let Some(key) = &_payload.key {
                keyboard.key_sequence(key);
            }
            modifiers.iter().for_each(|key| keyboard.key_up(*key));
        }
    }
}

#[derive(SimpleObject, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MousepadPayload {
    #[serde(default)]
    dx: Option<f32>,

    #[serde(default)]
    dy: Option<f32>,

    #[serde(default)]
    singleclick: Option<bool>,

    #[serde(default)]
    singlehold: Option<bool>,

    #[serde(default)]
    doubleclick: Option<bool>,

    #[serde(default)]
    middleclick: Option<bool>,

    #[serde(default)]
    rightclick: Option<bool>,

    #[serde(default)]
    scroll: Option<bool>,

    #[serde(default)]
    key: Option<String>,

    #[serde(default)]
    special_key: Option<u32>,

    #[serde(default)]
    shift: Option<bool>,

    #[serde(default)]
    ctrl: Option<bool>,

    #[serde(default)]
    alt: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, SimpleObject)]
pub struct MousepadConfig {
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, SimpleObject)]
pub struct MousepadState {
    enabled: bool,
    handle_mouse_events: bool,
    handle_keyboard_events: bool,
}

impl Default for MousepadState {
    fn default() -> Self {
        Self {
            enabled: true,
            handle_mouse_events: true,
            handle_keyboard_events: true,
        }
    }
}
