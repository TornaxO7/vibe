use cpal::{
    traits::{DeviceTrait, HostTrait},
    DeviceId,
};

type Devices = std::iter::Filter<cpal::Devices, for<'a> fn(&'a cpal::Device) -> bool>;

/// A little helper enum to set the type of a device.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum DeviceType {
    /// Input audio devices
    Input,
    /// Output audio devices
    Output,
}

/// Returns the given output/input device of the given id.
/// You can retrieve a list of available ids by using the [`get_device_names`] function.
///
/// Returns `Err` if there's a problem retrieving an output/input device.
/// Returns `Ok(None)` if retrieveing the output/input devices worked find but it couldn't find a device with the given name.
pub fn get_device(
    device_id: DeviceId,
    device_type: DeviceType,
) -> Result<Option<cpal::Device>, cpal::DevicesError> {
    let mut devices = get_devices(device_type)?;

    Ok(devices.find(|dev| dev.id().map(|id| id == device_id).unwrap_or(false)))
}

/// Returns the default device of he given device type (if available).
pub fn get_default_device(device_type: DeviceType) -> Option<cpal::Device> {
    let host = cpal::default_host();

    match device_type {
        DeviceType::Input => host.default_input_device(),
        DeviceType::Output => host.default_output_device(),
    }
}

fn get_devices(device_type: DeviceType) -> Result<Devices, cpal::DevicesError> {
    let host = cpal::default_host();

    match device_type {
        DeviceType::Input => host.input_devices(),
        DeviceType::Output => host.output_devices(),
    }
}

/// Returns a list of device ids which you can use for [`get_device`].
/// Returns `Err` if there's a problem retrieving an output/input device.
pub fn get_device_ids(device_type: DeviceType) -> Result<Vec<DeviceId>, cpal::DevicesError> {
    get_devices(device_type).map(|devices| devices.filter_map(|d| d.id().ok()).collect())
}
