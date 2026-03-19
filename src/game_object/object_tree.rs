pub type ObjectNode = Box<ObjectNodeRaw>;

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
