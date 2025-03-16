use anyhow::Result;

use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::{
    DEVICE_STATE_ACTIVE, IMMDeviceEnumerator, MMDeviceEnumerator, eCapture, eRender,
};
use windows::Win32::System::Com::{
    CLSCTX_ALL, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize, STGM_READ,
};

#[derive(Debug, PartialEq)]
pub enum DeviceType {
    Output,
    Input,
}

#[derive(Debug, PartialEq)]
pub struct Device {
    pub device_type: DeviceType,
    pub name: String,
    pub device_id: String,
}

impl Device {
    fn new(device_type: DeviceType, name: String, device_id: String) -> Self {
        Self {
            device_type,
            name,
            device_id,
        }
    }
}

impl std::fmt::Display for Device {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

pub fn enumerate_devices() -> Result<Vec<Device>> {
    let mut render_endpoints = Vec::<Device>::new();

    unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED)?;

        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let output_device_collection =
            enumerator.EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)?;

        for i in 0..output_device_collection.GetCount()? {
            let raw_device = output_device_collection.Item(i)?;
            let device_id = raw_device.GetId()?.to_string()?;
            let property_store = raw_device.OpenPropertyStore(STGM_READ)?;
            let name = property_store
                .GetValue(&PKEY_Device_FriendlyName)?
                .Anonymous
                .Anonymous
                .Anonymous
                .pwszVal
                .to_string()?;

            render_endpoints.push(Device::new(DeviceType::Output, name, device_id));
        }

        let input_device_collection =
            enumerator.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE)?;

        for i in 0..input_device_collection.GetCount()? {
            let raw_device = input_device_collection.Item(i)?;
            let device_id = raw_device.GetId()?.to_string()?;
            let property_store = raw_device.OpenPropertyStore(STGM_READ)?;
            let name = property_store
                .GetValue(&PKEY_Device_FriendlyName)?
                .Anonymous
                .Anonymous
                .Anonymous
                .pwszVal
                .to_string()?;

            render_endpoints.push(Device::new(DeviceType::Input, name, device_id));
        }

        CoUninitialize();
    }

    Ok(render_endpoints)
}
