use engine::{
    Engine, GameObjectTrait, ObjectNode, ObjectNodeRaw, ObjectNodeType, Scene, SceneCommand,
    SignedDistanceFunction,
};

struct OrbitObject {
    object_tree: ObjectNode,
}

impl OrbitObject {
    fn new() -> Self {
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
                        z: 2.0,
                    }),
                    Box::new(ObjectNodeRaw {
                        typ: ObjectNodeType::SDF(SignedDistanceFunction {
                            color: [1.0, 1.0, 1.0, 1.0],
                            size: 1.0,
                        }),
                        x: 0.7,
                        y: 0.0,
                        z: 2.0,
                    }),
                ),
                x: 0.0,
                y: 0.0,
                z: 0.0,
            }),
        }
    }
}

impl GameObjectTrait for OrbitObject {
    fn update(&mut self, _delta_time: f64) -> Vec<SceneCommand> {
        vec![]
    }

    fn get_syntax_tree(&self) -> &ObjectNode {
        &self.object_tree
    }
}

fn main() {
    let mut scene = Scene::new();

    scene.add(OrbitObject::new());

    let engine = Engine::new(
        |_| {
            println!("das ist elon muskete");
        },
        scene,
    );
    engine.run();
}
