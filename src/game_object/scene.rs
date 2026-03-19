use super::{
    FlatRenderTree, GameObjectBox, GameObjectTrait, SceneCommand,
    object_tree::{ObjectNode, ObjectNodeType},
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
            let mut keep = true;
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
                        keep = false;
                    } else {
                        commands.push(command);
                    }
                });
            keep
        });

        commands.into_iter().for_each(|command| match command {
            SceneCommand::Kill => unreachable!(),
            SceneCommand::Spawn(object) => self.game_objects.push(object),
        });

        self.last_time = current_time;
    }

    pub fn build_render_tree(&self) -> FlatRenderTree {
        let mut nodes: Vec<gpu_structs::SyntaxNode> = Vec::new();
        let mut leafs: Vec<gpu_structs::LeafObject> = Vec::new();

        if self.game_objects.is_empty() {
            panic!("game_objects is empty, not implemented!");
        }

        for game_obj in &self.game_objects {
            write_node(game_obj.get_syntax_tree(), &mut nodes, &mut leafs, -1);
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
    leafs: &mut Vec<gpu_structs::LeafObject>,
    parent: i32,
) -> bool {
    use ObjectNodeType::*;
    let index = nodes.len();
    match &node.typ {
        SDF(sdf) => {
            leafs.push(gpu_structs::LeafObject {
                position: [node.x, node.y, node.z],
                size: sdf.size,
                color: sdf.color,
                ..Default::default()
            });
            true
        }

        Union(left, right) => {
            nodes.push(gpu_structs::SyntaxNode {
                left_neg: 0,
                right_neg: 0,
                min: 0,
                parent: parent as i32,
                ..Default::default()
            });
            write_not_leaf(index, nodes, leafs, &left, &right);
            false
        }

        Intersection(left, right) => {
            nodes.push(gpu_structs::SyntaxNode {
                left_neg: 0,
                right_neg: 0,
                min: 1,
                parent: parent as i32,
                ..Default::default()
            });
            write_not_leaf(index, nodes, leafs, &left, &right);
            false
        }

        Subtraction(left, right) => {
            nodes.push(gpu_structs::SyntaxNode {
                left_neg: 0,
                right_neg: 1,
                min: 0,
                parent: parent as i32,
                ..Default::default()
            });
            write_not_leaf(index, nodes, leafs, left, right);
            false
        }
    }
}

fn write_not_leaf(
    index: usize,
    nodes: &mut Vec<gpu_structs::SyntaxNode>,
    leafs: &mut Vec<gpu_structs::LeafObject>,
    left: &ObjectNode,
    right: &ObjectNode,
) {
    //LEFT NODE
    let leafs_len = leafs.len() as i32;
    let nodes_len = nodes.len() as i32;
    let is_leaf_left = write_node(left, nodes, leafs, index as i32);

    if is_leaf_left {
        nodes[index].left = leafs_len;
        //nodes[index].left_gameobj = 1;
    } else {
        nodes[index].left = nodes_len;
        unreachable!();
        //nodes[index].left_gameobj = 0;
    };

    //RIGHT NODE
    let leafs_len = leafs.len() as i32;
    let nodes_len = nodes.len() as i32;
    let is_leaf_right = write_node(right, nodes, leafs, index as i32);

    if is_leaf_right {
        nodes[index].right = leafs_len;
        nodes[index].right_gameobj = 1;
    } else {
        nodes[index].right = nodes_len;
        nodes[index].right_gameobj = 0;
    };
}

#[cfg(test)]
mod tests {
    use super::super::{big_dummy::BigDummy, dummy::Dummy};
    use super::*;
    use crate::gpu_structs::{LeafObject, SyntaxNode};

    #[test]
    fn test_dummy() {
        let mut scene = Scene::new();
        scene.add(Dummy::new());

        let render_tree = scene.build_render_tree();

        assert_eq!(
            render_tree.nodes,
            vec![SyntaxNode {
                parent: -1,
                left: 0,
                left_neg: 0,
                right: 1,
                right_neg: 0,
                right_gameobj: 1,
                min: 1,
                _pad: [0,],
            },]
        );

        assert_eq!(
            render_tree.leafs,
            vec![
                LeafObject {
                    position: [0.0, 0.0, 2.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
                LeafObject {
                    position: [0.7, 0.0, 2.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
            ]
        );

        assert_eq!(render_tree.first_layer_length, 1);
    }

    #[test]
    fn test_big_dummy() {
        let mut scene = Scene::new();
        scene.add(BigDummy::new());

        let render_tree = scene.build_render_tree();

        assert_eq!(
            render_tree.nodes,
            vec![
                SyntaxNode {
                    parent: -1,
                    left: 0,
                    left_neg: 0,
                    right: 1,
                    right_neg: 0,
                    right_gameobj: 0,
                    min: 1,
                    _pad: [0,],
                },
                SyntaxNode {
                    parent: 0,
                    left: 1,
                    left_neg: 0,
                    right: 2,
                    right_neg: 0,
                    right_gameobj: 1,
                    min: 1,
                    _pad: [0,],
                },
            ]
        );

        assert_eq!(
            render_tree.leafs,
            vec![
                LeafObject {
                    position: [1.0, 0.0, 0.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
                LeafObject {
                    position: [2.0, 0.0, 0.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
                LeafObject {
                    position: [3.0, 0.0, 0.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
            ]
        );

        assert_eq!(render_tree.first_layer_length, 1);
    }

    #[test]
    fn test_big_and_small_dummy() {
        let mut scene = Scene::new();
        scene.add(Dummy::new());
        scene.add(BigDummy::new());

        let render_tree = scene.build_render_tree();

        assert_eq!(
            render_tree.nodes,
            vec![
                SyntaxNode {
                    parent: -1,
                    left: 0,
                    left_neg: 0,
                    right: 1,
                    right_neg: 0,
                    right_gameobj: 1,
                    min: 1,
                    _pad: [0,],
                },
                SyntaxNode {
                    parent: -1,
                    left: 2,
                    left_neg: 0,
                    right: 2,
                    right_neg: 0,
                    right_gameobj: 0,
                    min: 1,
                    _pad: [0,],
                },
                SyntaxNode {
                    parent: 1,
                    left: 3,
                    left_neg: 0,
                    right: 4,
                    right_neg: 0,
                    right_gameobj: 1,
                    min: 1,
                    _pad: [0,],
                },
            ]
        );

        assert_eq!(
            render_tree.leafs,
            vec![
                LeafObject {
                    position: [0.0, 0.0, 2.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
                LeafObject {
                    position: [0.7, 0.0, 2.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
                LeafObject {
                    position: [1.0, 0.0, 0.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
                LeafObject {
                    position: [2.0, 0.0, 0.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
                LeafObject {
                    position: [3.0, 0.0, 0.0,],
                    size: 1.0,
                    color: [1.0, 1.0, 1.0, 1.0,],
                    ..Default::default()
                },
            ]
        );

        assert_eq!(render_tree.first_layer_length, 2);
    }
}
