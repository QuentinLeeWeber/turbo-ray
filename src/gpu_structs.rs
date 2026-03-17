#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GameObjectStorage {
    pub length: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderObject {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
    pub _padding: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SyntaxNode {
    pub left: u32,
    pub left_neg: u32,     // bool
    pub left_gameobj: u32, // bool
    pub right: u32,
    pub right_neg: u32,     // bool
    pub right_gameobj: u32, // bool
    pub min: u32,           // bool
    pub parent: u32,
}

impl Default for SyntaxNode {
    fn default() -> Self {
        Self {
            left: 0,
            left_neg: 0,
            left_gameobj: 0,
            right: 0,
            right_neg: 0,
            right_gameobj: 0,
            min: 0,
            parent: 0,
        }
    }
}

impl Default for RenderObject {
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

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub pos: [f32; 3],
    pub _pad0: f32,
    pub rot: [f32; 3],
    pub fov: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: [0.0, 0.0, 0.0],
            _pad0: 0.0,
            rot: [0.0, 0.0, 1.0],
            fov: std::f32::consts::PI * 2.0,
        }
    }
}
