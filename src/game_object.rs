#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GameObjectStorage {
    pub length: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GameObject {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
    pub _padding: [f32; 4],
}

impl Default for GameObject {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            size: 1.0,
            color: [1.0, 1.0, 1.0, 1.0],
            _padding: [0.0, 0.0, 0.0, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenSize {
    pub size: [f32; 2],
    pub _pad: [f32; 2],
}
