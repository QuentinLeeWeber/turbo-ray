use crate::game_object::{
    GameObjectTrait, SceneCommand,
    object_tree::{ObjectNode, ObjectNodeRaw, ObjectNodeType, SignedDistanceFunction},
};

pub struct BigDummy {
    object_tree: ObjectNode,
}

impl BigDummy {
    pub fn new() -> Self {
        Self {
            object_tree: Box::new(ObjectNodeRaw {
                typ: ObjectNodeType::Intersection(
                    Box::new(ObjectNodeRaw {
                        typ: ObjectNodeType::SDF(SignedDistanceFunction {
                            color: [1.0, 1.0, 1.0, 1.0],
                            size: 1.0,
                        }),
                        x: 1.0,
                        y: 0.0,
                        z: 0.0,
                    }),
                    Box::new(ObjectNodeRaw {
                        typ: ObjectNodeType::Intersection(
                            Box::new(ObjectNodeRaw {
                                typ: ObjectNodeType::SDF(SignedDistanceFunction {
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    size: 1.0,
                                }),
                                x: 2.0,
                                y: 0.0,
                                z: 0.0,
                            }),
                            Box::new(ObjectNodeRaw {
                                typ: ObjectNodeType::SDF(SignedDistanceFunction {
                                    color: [1.0, 1.0, 1.0, 1.0],
                                    size: 1.0,
                                }),
                                x: 3.0,
                                y: 0.0,
                                z: 0.0,
                            }),
                        ),
                        x: 0.0,
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

impl GameObjectTrait for BigDummy {
    fn update(&mut self, delta_time: f64) -> Vec<SceneCommand> {
        let _ = delta_time;
        vec![]
    }

    fn get_syntax_tree(&self) -> &ObjectNode {
        &self.object_tree
    }
}
