use crate::game_object::{
    GameObjectTrait, SceneCommand,
    object_tree::{ObjectNode, ObjectNodeRaw, ObjectNodeType, SignedDistanceFunction},
};

pub struct Dummy {
    object_tree: ObjectNode,
}

impl Dummy {
    pub fn new() -> Self {
        Self {
            object_tree: Box::new(ObjectNodeRaw {
                typ: ObjectNodeType::Intersection(
                    Box::new(ObjectNodeRaw {
                        typ: ObjectNodeType::SDF(SignedDistanceFunction {
                            color: [1.0, 1.0, 1.0, 1.0],
                            size: 1.0,
                        }),
                        x: 0.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    Box::new(ObjectNodeRaw {
                        typ: ObjectNodeType::SDF(SignedDistanceFunction {
                            color: [1.0, 1.0, 1.0, 1.0],
                            size: 1.0,
                        }),
                        x: 0.7,
                        y: 0.0,
                        z: 0.0,
                    }),
                ),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }
}

impl GameObjectTrait for Dummy {
    fn update(&mut self, delta_time: f64) -> Vec<SceneCommand> {
        vec![]
    }

    fn get_syntax_tree(&self) -> &ObjectNode {
        &self.object_tree
    }
}
