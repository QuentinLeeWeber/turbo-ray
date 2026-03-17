@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>,3>(
        vec2(-1.0, -1.0),
        vec2(3.0, -1.0),
        vec2(-1.0, 3.0)
    );
    return vec4(pos[i], 0.0, 1.0);
}

const PI: f32 = 3.14159265359;

@group(0) @binding(0)
var<storage, read> game_objects: GameObjectStorage;
struct GameObjectStorage {
    length: i32,
    object: array<GameObject>,
}
struct GameObject {
    position: vec3<f32>,
    size: f32,
    color: vec4<f32>,
}

@group(3) @binding(0)
var<storage,read> syntax_tree: SyntaxTree;
struct SyntaxTree {
    length: i32,
    num_root: u32,
    nodes: array<SyntaxNode>,
}

struct SyntaxNode {
    left: u32,
    left_neg: u32, //bool
    left_gameobj: u32, //bool
    right: u32,
    right_neg: u32, //bool
    right_gameobj: u32, //bool
    min: u32, //bool
}

@group(1) @binding(0)
var<uniform> screen_size: vec2<f32>;

@group(2) @binding(0)
var<uniform> cam: Camera;
struct Camera {
    pos: vec3<f32>,
    rot: vec3<f32>,
    fov: f32,
}

fn sdSphere(x: vec3<f32>, radius: f32) -> f32 {
    return length(x) - radius;
}

struct SmallestRadiusResult {
    radius: f32,
    index: i32,
}

fn smallest_radius(origin: vec3<f32>) -> SmallestRadiusResult {
    var min_radius: f32 = 1000000.0;
    var min_index: i32 = 0;
    for (var i = 0; i < game_objects.length; i++) {
        let next_radius = sdSphere(game_objects.object[i].position - origin, game_objects.object[i].size);
        if next_radius < min_radius {
            min_radius = next_radius;
            min_index = i;
        }
    }
    return SmallestRadiusResult(min_radius, min_index);
}

fn rayMarch(origin: vec3<f32>, direction: vec3<f32>) -> vec4<f32> {
    let maxIterations: i32 = 100;
    let eps: f32 = 0.001;
    var dist: f32 = 0.0;

    for (var i = 0; i < maxIterations; i++) {
        let point = origin + direction * dist;

        var radius_result = smallest_radius(point);
        if radius_result.radius < eps {
            let cos_angle = dot(aproximative_normal(point, radius_result.radius), cam.rot);
            let brightness = clamp(cos_angle, 0.0, 1.0);

            let color = game_objects.object[radius_result.index].color.rgb * brightness * (1.0 - dist);

            return vec4(color, 1.0);
        }
        dist += radius_result.radius;
    }
    return vec4(0.0, 0.0, 0.0, 0.0);
}

fn aproximative_normal(p: vec3<f32>, radius: f32) -> vec3<f32> {
    let eps = 0.001;
    return normalize(vec3(
        sdSphere(p + vec3(eps, 0.0, 0.0), radius) - sdSphere(p - vec3(eps, 0.0, 0.0), radius),
        sdSphere(p + vec3(0.0, eps, 0.0), radius) - sdSphere(p - vec3(0.0, eps, 0.0), radius),
        sdSphere(p + vec3(0.0, 0.0, eps), radius) - sdSphere(p - vec3(0.0, 0.0, eps), radius),
    ));
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {
    let ndc = (fragCoord.xy / screen_size) * 2.0 - vec2(1.0, 1.0);
    let aspect = screen_size.x / screen_size.y;
    let dir = normalize(vec3(ndc.x * tan(cam.fov / 2.0) * aspect, ndc.y * tan(cam.fov / 2.0), 1.0));

    let color = rayMarch(cam.pos, dir);
    return color;
}
