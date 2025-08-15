use cpal::traits::{DeviceTrait, HostTrait};

type Devices = std::iter::Filter<cpal::Devices, for<'a> fn(&'a cpal::Device) -> bool>;

/// A little helper enum to set the type of a device.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Hash)]
pub enum DeviceType {
    /// Input audio devices
    Input,
    /// Output audio devices
    Output,
}

/// Returns the given output/input device with the given name.
/// You can retrieve a list of available names by using the [`get_device_names`] function.
///
/// Returns `Err` if there's a problem retrieving an output/input device.
/// Returns `Ok(None)` if retrieveing the output/input devices worked find but it couldn't find a device with the given name.
pub fn get_device<S: AsRef<str>>(
    name: S,
    device_type: DeviceType,
) -> Result<Option<cpal::Device>, cpal::DevicesError> {
    let mut devices = get_devices(device_type)?;

    Ok(devices.find(|d| {
        d.name()
            .map(|d_name| d_name.as_str() == name.as_ref())
            .unwrap_or(false)
    }))
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

/// Returns a list of device names which you can use for [`get_device`].
/// Retunrs `Err` if there's a problem retrieving an output/input device.
pub fn get_device_names(device_type: DeviceType) -> Result<Vec<String>, cpal::DevicesError> {
    let devices = get_devices(device_type)?;

    Ok(devices.filter_map(|d| d.name().ok()).collect())
}
