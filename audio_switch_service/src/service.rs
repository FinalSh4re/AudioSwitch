use crate::config::Config;
use crate::tray::{UserEvent, create_tray};

use anyhow::Result;
use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use std::thread;
use tauri_winrt_notification::{Duration, Toast};
use win_hotkey::keys::{ModifiersKey, VirtualKey};
use win_hotkey::{HotkeyManager, HotkeyManagerImpl};
use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, DEVICE_STATE_DISABLED, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture,
    eConsole, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::core::PCWSTR;
use winit::event_loop::EventLoop;

pub struct Service {
    config: Config,
    event_loop: EventLoop<UserEvent>,
}

impl Service {
    pub fn new() -> Self {
        let config: Config = confy::load("AudioSwitch", None).expect("Failed to load config.");
        let event_loop = EventLoop::<UserEvent>::with_user_event()
            .build()
            .expect("Failed to initialize event loop.");

        Self { config, event_loop }
    }

    pub fn run_app(self) {
        self.setup_hotkey_handler();
        create_tray(self.event_loop);
    }

    fn setup_hotkey_handler(&self) {
        let mut proxies = vec![];

        for _ in 0..self.config.profiles.len() {
            proxies.push(self.event_loop.create_proxy());
        }

        let profiles = self.config.profiles.clone();

        thread::spawn(|| {
            unsafe {
                CoInitializeEx(None, COINIT_MULTITHREADED).expect("Failed to initialize Thread.");
            }

            let mut hkm = HotkeyManager::new();

            for (p, proxy) in profiles.into_iter().zip(proxies) {
                let vk = VirtualKey::from_keyname(
                    &p.hotkey
                        .hotkey
                        .strip_prefix("VK_")
                        .unwrap_or(&p.hotkey.hotkey),
                )
                .expect("Invalid Key.");
                let vk_mod = p
                    .hotkey
                    .modifier
                    .map(|i| vec![ModifiersKey::from_keyname(&i).unwrap()]);

                hkm.register(
                    vk,
                    vk_mod.as_ref().map(|v| v.as_slice()),
                    Some(move || {
                        if let Ok(_) = Self::set_endpoint_visibility(false) {
                            match Self::set_profile(p.input_id.clone(), p.output_id.clone()) {
                                Ok(_) => {
                                    Self::send_toast(format!(
                                        "Activated Profile {}",
                                        p.profile_name.clone()
                                    ))
                                    .expect("Failed to send notification.");

                                    if let Some(color) = p.color.clone() {
                                        let _ =
                                            proxy.clone().send_event(UserEvent::ColorChange(color));
                                    }
                                }

                                Err(_) => {
                                    Self::set_endpoint_visibility(true)
                                        .expect("Failed to reactivate endpoints.");

                                    Self::send_toast(format!(
                                        "Failed to activate Profile {}",
                                        p.profile_name.clone()
                                    ))
                                    .expect("Failed to send notification.");
                                }
                            }
                        } else {
                            eprintln!("Failed to deactivate endpoints.");
                        }
                    }),
                )
                .expect("Failed to register hotkey.");
            }

            hkm.event_loop();

            unsafe {
                CoUninitialize();
            }
        });
    }

    fn set_profile(input_id: String, output_id: String) -> Result<()> {
        unsafe {
            let policy_config: IPolicyConfig =
                CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let _capture_collection = enumerator
                .EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE | DEVICE_STATE_DISABLED)?;

            let mut input_id_u16 = input_id.encode_utf16().collect::<Vec<u16>>();
            let input_id_raw_ptr = input_id_u16.as_mut_ptr();
            let input_id_pcwstr = PCWSTR(input_id_raw_ptr);

            policy_config.SetEndpointVisibility(input_id_pcwstr, true)?;
            policy_config.SetDefaultEndpoint(input_id_pcwstr, eConsole)?;

            let _render_collection = enumerator
                .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE | DEVICE_STATE_DISABLED)?;

            let mut output_id_u16 = output_id.encode_utf16().collect::<Vec<u16>>();
            let output_id_raw_ptr = output_id_u16.as_mut_ptr();
            let output_id_pcwstr = PCWSTR(output_id_raw_ptr);

            policy_config.SetEndpointVisibility(output_id_pcwstr, true)?;
            policy_config.SetDefaultEndpoint(output_id_pcwstr, eConsole)?;
        }

        Ok(())
    }

    fn set_endpoint_visibility(toggle: bool) -> Result<()> {
        unsafe {
            let policy_config: IPolicyConfig =
                CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;

            let enumerator: IMMDeviceEnumerator =
                CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let capture_collection =
                enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;

            for i in 0..capture_collection.GetCount()? {
                let raw_device = capture_collection.Item(i)?;
                let device_id = raw_device.GetId()?;
                let device_id_pcwstr = PCWSTR(device_id.0 as *const u16);

                policy_config.SetEndpointVisibility(device_id_pcwstr, toggle)?;
            }

            let render_collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;

            for i in 0..render_collection.GetCount()? {
                let raw_device = render_collection.Item(i)?;
                let device_id = raw_device.GetId()?;
                let device_id_pcwstr = PCWSTR(device_id.0 as *const u16);

                policy_config.SetEndpointVisibility(device_id_pcwstr, toggle)?;
            }
        }

        Ok(())
    }

    fn send_toast(msg: String) -> Result<()> {
        Toast::new(Toast::POWERSHELL_APP_ID)
            .title(&msg)
            .duration(Duration::Short)
            .show()
            .expect("Toast failed.");

        Ok(())
    }
}
