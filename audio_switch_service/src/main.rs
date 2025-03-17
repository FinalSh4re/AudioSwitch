//#![windows_subsystem = "windows"]

mod config;
mod tray;

use anyhow::Result;
use com_policy_config::{IPolicyConfig, PolicyConfigClient};
use std::thread;
use tauri_winrt_notification::{Duration, Toast};
use win_hotkey::keys::{ModifiersKey, VirtualKey};
use win_hotkey::{HotkeyManager, HotkeyManagerImpl};
use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eConsole, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::core::PCWSTR;

pub fn main() -> Result<()> {
    setup_hotkey_handler();
    tray::create_tray();

    Ok(())
}

fn setup_hotkey_handler() {
    thread::spawn(|| {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED).expect("Failed to initialize Thread.");
        }

        let config: config::Config =
            confy::load("AudioSwitch", None).expect("Failed to open config.");
        let mut hkm = HotkeyManager::new();

        for p in config.profiles {
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
                Some(
                    move || match set_profile(p.input_id.clone(), p.output_id.clone()) {
                        Ok(_) => {
                            send_toast(format!("Activated Profile {}", p.profile_name.clone()))
                                .expect("Failed to send notification.");
                        }

                        Err(_) => {
                            send_toast(format!(
                                "Failed to activate Profile {}",
                                p.profile_name.clone()
                            ))
                            .expect("Failed to send notification.");
                        }
                    },
                ),
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
        let policy_config: IPolicyConfig = CoCreateInstance(&PolicyConfigClient, None, CLSCTX_ALL)?;

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
        let _capture_collection = enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;

        let mut input_id_u16 = input_id.encode_utf16().collect::<Vec<u16>>();
        let input_id_raw_ptr = input_id_u16.as_mut_ptr();
        let input_id_pcwstr = PCWSTR(input_id_raw_ptr);

        policy_config.SetDefaultEndpoint(input_id_pcwstr, eConsole)?;

        let _render_collection = enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;

        let mut output_id_u16 = output_id.encode_utf16().collect::<Vec<u16>>();
        let output_id_raw_ptr = output_id_u16.as_mut_ptr();
        let output_id_pcwstr = PCWSTR(output_id_raw_ptr);

        policy_config.SetDefaultEndpoint(output_id_pcwstr, eConsole)?;
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
