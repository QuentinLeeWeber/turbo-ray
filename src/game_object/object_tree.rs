pub type ObjectNode = Box<SyntaxNodeRaw>;

pub struct SyntaxNodeRaw {
    pub typ: ObjectnodeType,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

pub enum ObjectnodeType {
    SDF(SignedDistanceFunction),
    Union(ObjectNode, ObjectNode),
    Intersection(ObjectNode, ObjectNode),
    Subtraction(ObjectNode, ObjectNode),
}

pub struct SignedDistanceFunction {
    pub color: [f32; 4],
    pub size: f32,
}
