pub type ObjectNode = Box<ObjectNodeRaw>;

pub struct ObjectNodeRaw {
    pub typ: ObjectNodeType,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub enum ObjectNodeType {
    SDF(SignedDistanceFunction),
    Union(ObjectNode, ObjectNode),
    Intersection(ObjectNode, ObjectNode),
    Subtraction(ObjectNode, ObjectNode),
}

pub struct SignedDistanceFunction {
    pub color: [f32; 4],
    pub size: f32,
}
