use crate::{renderer::gpu_structs, scene::SceneCommand};
use std::{cell::RefCell, rc::Rc};

pub type GameObjectBox = Box<dyn GameObjectTrait>;

pub trait GameObjectTrait {
    fn update(&mut self, delta_time: f64) -> Vec<SceneCommand>;
    fn get_syntax_tree(&self) -> &ObjectNode;
}

#[derive(Debug)]
pub struct FlatRenderTree {
    pub nodes: Vec<gpu_structs::SyntaxNode>,
    pub leafs: Vec<gpu_structs::LeafObject>,
    pub first_layer_length: u32,
}

pub type ObjectNode = Rc<RefCell<ObjectNodeRaw>>;

#[derive(Debug)]
pub struct ObjectNodeRaw {
    pub typ: ObjectNodeType,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug)]
pub enum ObjectNodeType {
    SDF(SignedDistanceFunction),
    Union(ObjectNode, ObjectNode),
    Intersection(ObjectNode, ObjectNode),
    Subtraction(ObjectNode, ObjectNode),
}

#[derive(Debug)]
pub struct SignedDistanceFunction {
    pub color: [f32; 4],
    pub size: f32,
}
