use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct LeafObjectStorage {
    pub length: u32,
    pub _pad: [u32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct LeafObject {
    pub position: [f32; 3],
    pub size: f32,
    pub color: [f32; 4],
    pub _padding: [f32; 4],
}

impl PartialEq for LeafObject {
    fn eq(&self, other: &Self) -> bool {
        self.position
            .iter()
            .zip(other.position.iter())
            .all(|(a, b)| (a - b).abs() < 1e-6)
            && (self.size - other.size).abs() < 1e-6
            && self
                .color
                .iter()
                .zip(other.color.iter())
                .all(|(a, b)| (a - b).abs() < 1e-6)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct SyntaxNodeStorage {
    pub length: i32,
    pub num_root: i32,
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq)]
pub struct SyntaxNode {
    pub parent: i32,
    pub left: i32,
    pub left_neg: u32, // bool
    //pub left_gameobj: u32, // bool
    pub right: i32,
    pub right_neg: i32,     // bool
    pub right_gameobj: u32, // bool
    pub min: u32,           // bool
    pub _pad: [u32; 1],
}

impl Default for SyntaxNode {
    fn default() -> Self {
        Self {
            parent: 0,
            left: 0,
            left_neg: 0,
            //left_gameobj: 0,
            right: 0,
            right_neg: 0,
            right_gameobj: 0,
            min: 0,
            _pad: [0; 1],
        }
    }
}

impl Default for LeafObject {
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
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
pub struct ScreenSize {
    pub size: [f32; 2],
    pub _pad: [f32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, Debug)]
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
