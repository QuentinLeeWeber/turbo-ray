use super::{
    FlatRenderTree, GameObjectBox, GameObjectTrait, SceneCommand,
    object_tree::{ObjectNode, ObjectnodeType, SignedDistanceFunction},
};
use crate::gpu_structs;
use std::time::SystemTime;

pub struct Scene {
    last_time: SystemTime,
    game_objects: Vec<GameObjectBox>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            last_time: SystemTime::now(),
            game_objects: Vec::new(),
        }
    }

    pub fn add<T>(&mut self, game_object: T)
    where
        T: GameObjectTrait + 'static,
    {
        self.game_objects.push(Box::new(game_object));
    }

    pub fn update(&mut self) {
        let current_time = std::time::SystemTime::now();
        let mut commands: Vec<SceneCommand> = vec![];
        self.game_objects.retain_mut(|game_object| {
            let mut should_remove = false;
            game_object
                .update(
                    current_time
                        .duration_since(self.last_time)
                        .unwrap_or_default()
                        .as_secs_f64(),
                )
                .into_iter()
                .for_each(|command| {
                    if let SceneCommand::Kill = command {
                        should_remove = true;
                    } else {
                        commands.push(command);
                    }
                });
            should_remove
        });

        commands.into_iter().for_each(|command| match command {
            SceneCommand::Kill => unreachable!(),
            SceneCommand::Spawn(object) => self.game_objects.push(object),
        });

        self.last_time = current_time;
    }

    fn build_render_tree(&self) -> FlatRenderTree {
        let mut nodes: Vec<gpu_structs::SyntaxNode> = Vec::new();
        let mut leafs: Vec<gpu_structs::RenderObject> = Vec::new();

        if self.game_objects.is_empty() {
            panic!("game_objects is empty, not implemented!");
        }

        for game_obj in &self.game_objects {
            write_node(game_obj.get_syntax_tree(), &mut nodes, &mut leafs, 0);
        }

        FlatRenderTree {
            nodes,
            leafs,
            first_layer_length: self.game_objects.len() as u32,
        }
    }
}

fn write_node(
    node: &ObjectNode,
    nodes: &mut Vec<gpu_structs::SyntaxNode>,
    leafs: &mut Vec<gpu_structs::RenderObject>,
    parent: usize,
) -> bool {
    use ObjectnodeType::*;
    match &node.typ {
        SDF(sdf) => {
            leafs.push(gpu_structs::RenderObject {
                position: [node.x, node.y, node.z],
                size: sdf.size,
                color: sdf.color,
                ..Default::default()
            });
            true
        }

        Union(left, right) => {
            let index = nodes.len();
            nodes.push(gpu_structs::SyntaxNode {
                left_neg: 0,
                right_neg: 0,
                min: 0,
                parent: parent as u32,
                ..Default::default()
            });
            write_syntax_node(index, nodes, leafs, &left, &right);
            false
        }

        Intersection(left, right) => {
            let index = nodes.len();
            nodes.push(gpu_structs::SyntaxNode {
                left_neg: 0,
                right_neg: 0,
                min: 1,
                parent: parent as u32,
                ..Default::default()
            });
            write_syntax_node(index, nodes, leafs, &left, &right);
            false
        }

        Subtraction(left, right) => {
            let index = nodes.len();
            nodes.push(gpu_structs::SyntaxNode {
                left_neg: 0,
                right_neg: 1,
                min: 0,
                parent: parent as u32,
                ..Default::default()
            });
            write_syntax_node(index, nodes, leafs, left, right);
            false
        }
    }
}

fn write_syntax_node(
    index: usize,
    nodes: &mut Vec<gpu_structs::SyntaxNode>,
    leafs: &mut Vec<gpu_structs::RenderObject>,
    left: &ObjectNode,
    right: &ObjectNode,
) {
    let is_leaf_right = write_node(left, nodes, leafs, index);

    if is_leaf_right {
        nodes[index].right = leafs.len() as u32 - 1;
        nodes[index].right_gameobj = 1;
    } else {
        nodes[index].right = nodes.len() as u32 - 1;
        nodes[index].right_gameobj = 0;
    };

    let is_leaf_left = write_node(right, nodes, leafs, index);

    if is_leaf_left {
        nodes[index].left = leafs.len() as u32 - 1;
        nodes[index].left_gameobj = 1;
    } else {
        nodes[index].left = nodes.len() as u32 - 1;
        nodes[index].left_gameobj = 0;
    };
}
