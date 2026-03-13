#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GameObject {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenSize {
    pub size: [f32; 2],
    pub _pad: [f32; 2],
}

pub enum SignedDistanceFunction {
    Sphere,
}
