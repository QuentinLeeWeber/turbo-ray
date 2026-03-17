use crate::gpu_structs;

pub mod object_tree;
pub mod scene;

pub type GameObjectBox = Box<dyn GameObjectTrait>;

pub trait GameObjectTrait {
    fn update(&mut self, delta_time: f64) -> Vec<SceneCommand>;
    fn get_syntax_tree(&self) -> &object_tree::ObjectNode;
}

pub enum SceneCommand {
    Kill,
    Spawn(GameObjectBox),
}

struct FlatRenderTree {
    nodes: Vec<gpu_structs::SyntaxNode>,
    leafs: Vec<gpu_structs::RenderObject>,
    first_layer_length: u32,
}
