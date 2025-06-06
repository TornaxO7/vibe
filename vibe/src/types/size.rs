use smithay_client_toolkit::output::OutputInfo;
use wgpu::SurfaceConfiguration;
use winit::dpi::PhysicalSize;

/// A simple `Size` struct which saves `width` and `height`.
#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

impl From<&OutputInfo> for Size {
    fn from(value: &OutputInfo) -> Self {
        let (width, height) = value
            .logical_size
            .map(|(width, height)| (width as u32, height as u32))
            .unwrap();

        Self { width, height }
    }
}

impl From<(u32, u32)> for Size {
    fn from(value: (u32, u32)) -> Self {
        Self {
            width: value.0,
            height: value.1,
        }
    }
}

impl From<&SurfaceConfiguration> for Size {
    fn from(value: &SurfaceConfiguration) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

impl From<PhysicalSize<u32>> for Size {
    fn from(size: PhysicalSize<u32>) -> Self {
        Self {
            width: size.width,
            height: size.height,
        }
    }
}
