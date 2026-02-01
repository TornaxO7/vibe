//! Type wrappers of types which are used in the wgsl language.

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
pub struct Vec2f([f32; 2]);

impl From<[f32; 2]> for Vec2f {
    fn from(value: [f32; 2]) -> Self {
        Self(value)
    }
}

impl From<cgmath::Vector2<f32>> for Vec2f {
    fn from(value: cgmath::Vector2<f32>) -> Self {
        Self(value.into())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
pub struct Vec3f([f32; 3]);

impl From<[f32; 3]> for Vec3f {
    fn from(value: [f32; 3]) -> Self {
        Self(value)
    }
}

impl From<cgmath::Vector3<f32>> for Vec3f {
    fn from(value: cgmath::Vector3<f32>) -> Self {
        Self(value.into())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
pub struct Vec4f([f32; 4]);

impl From<[f32; 4]> for Vec4f {
    fn from(value: [f32; 4]) -> Self {
        Self(value)
    }
}

impl From<cgmath::Vector4<f32>> for Vec4f {
    fn from(value: cgmath::Vector4<f32>) -> Self {
        Self(value.into())
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, Default)]
pub struct Mat2x2([[f32; 2]; 2]);

impl From<[[f32; 2]; 2]> for Mat2x2 {
    fn from(value: [[f32; 2]; 2]) -> Self {
        Self(value)
    }
}

impl From<cgmath::Matrix2<f32>> for Mat2x2 {
    fn from(value: cgmath::Matrix2<f32>) -> Self {
        Self(value.into())
    }
}
