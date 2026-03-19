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
    length: u32,
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
    length: u32,
    num_root: i32,
    nodes: array<SyntaxNode>,
}

struct SyntaxNode {
    parent: i32, //negative for root nodes (eg. -1)
    left: i32,  //left node is always a gameobj
    left_neg: u32, //bool
    right: i32,
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

fn get_lowest_leaf(start: i32) -> i32 {
    var cur = start;
    while syntax_tree.nodes[cur].right_gameobj == 0 {
        cur = syntax_tree.nodes[cur].right;
    }
    return cur;
}

fn max(a: f32, b: f32) -> f32 {
    if a > b {
        return a;
    }
    return b;
}
fn min(a: f32, b: f32) -> f32 {
    if a < b {
        return a;
    }
    return b;
}

//gets smallest distance to any gameobject based on the syntax tree
fn get_smallest_distance(origin: vec3<f32>) -> SmallestRadiusResult {
    var min_radius: f32 = 1000000.0;
    var min_index: i32 = 0;
    for (var i: i32 = 0; i < syntax_tree.num_root; i++) {
        var cur_node_index: i32 = i;
        let cur_node_tmp = &syntax_tree.nodes[get_lowest_leaf(cur_node_index)];
        let right_gameobj = &game_objects.object[(*cur_node_tmp).right];
        var cur_dist: f32 = sdSphere((*right_gameobj).position - origin, (*right_gameobj).size);//always right node combined value, initially lowest right leaf distance
        var collided_gameobj_index = (*cur_node_tmp).right;
        while true {
            let cur_node = &syntax_tree.nodes[get_lowest_leaf(cur_node_index)];
            let left_gameobj = &game_objects.object[(*cur_node).left];
            var left_dist: f32 = sdSphere((*left_gameobj).position - origin, (*left_gameobj).size);
            if (*cur_node).left_neg != 0 {
                left_dist *= -1;
            }
            if (*cur_node).right_neg != 0 {
                cur_dist *= -1;
            }
            switch (*cur_node).min {
                case 0: {//max
                    cur_dist = max(cur_dist, left_dist);
                }
                case default: {//min
                    cur_dist = min(cur_dist, left_dist);
                }
            }
            if cur_dist == left_dist {
                collided_gameobj_index = (*cur_node).left;
            }
            cur_node_index = (*cur_node).parent;
            if cur_node_index < 0 {
                break;
            }
        }
        if cur_dist < min_radius {
            min_radius = cur_dist;
            min_index = collided_gameobj_index;
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

        var radius_result = get_smallest_distance(point);
        if radius_result.radius < eps {
            let normal = approximate_normal(point);

            let light_dir = normalize(vec3(1.0, 1.0, -1.0));
            let ambient = 0.008;
            let diffuse = clamp(dot(normal, light_dir), 0.0, 1.0);
            let brightness = ambient + diffuse * (1.0 - ambient);
            let color = game_objects.object[radius_result.index].color.rgb * brightness;

            return vec4(color, 1.0);
        }
        dist += max(radius_result.radius, 0.0);
    }
    return vec4(0.0, 0.0, 0.0, 0.0);
}

/*fn scene_sdf(p: vec3<f32>) -> f32 {
    var min_dist: f32 = 1000000.0;

    for (var i = 0; i < game_objects.length; i++) {
        let obj = game_objects.object[i];
        let d = sdSphere(p - obj.position, obj.size);

        if d < min_dist {
            min_dist = d;
        }
    }

    return min_dist;
}*/

fn approximate_normal(p: vec3<f32>) -> vec3<f32> {
    let eps: f32 = 0.001;

    let dx = get_smallest_distance(p + vec3(eps, 0.0, 0.0)).radius - get_smallest_distance(p - vec3(eps, 0.0, 0.0)).radius;
    let dy = get_smallest_distance(p + vec3(0.0, eps, 0.0)).radius - get_smallest_distance(p - vec3(0.0, eps, 0.0)).radius;
    let dz = get_smallest_distance(p + vec3(0.0, 0.0, eps)).radius - get_smallest_distance(p - vec3(0.0, 0.0, eps)).radius;

    return normalize(vec3(dx, dy, dz));
}

fn get_rot_x(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);

    return mat3x3<f32>(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, c, s),
        vec3<f32>(0.0, -s, c)
    );
}

fn get_rot_y(angle: f32) -> mat3x3<f32> {
    let c = cos(angle);
    let s = sin(angle);

    return mat3x3<f32>(
        vec3<f32>(c, 0.0, -s),
        vec3<f32>(0.0, 1.0, 0.0),
        vec3<f32>(s, 0.0, c)
    );
}

fn apply_camera_rotation(dir: vec3<f32>, rot: vec3<f32>) -> vec3<f32> {
    let rot_y = get_rot_y(rot.y);
    let rot_x = get_rot_x(rot.x);

    let combined_rot = rot_x * rot_y;

    return combined_rot * dir;
}

@fragment
fn fs_main(@builtin(position) fragCoord: vec4<f32>) -> @location(0) vec4<f32> {

    /*if syntax_tree.nodes[0].parent == -1
        && syntax_tree.nodes[0].left == 0
        && syntax_tree.nodes[0].left_neg == 0
        && syntax_tree.nodes[0].right == 1
        && syntax_tree.nodes[0].right_neg == 0
        && syntax_tree.nodes[0].right_gameobj == 1
        && syntax_tree.nodes[0].min == 0
        && game_objects.object[0].position.x == 0.0
        && game_objects.object[0].position.y == 0.0
        && game_objects.object[0].position.z == 2.0
        && game_objects.object[0].size == 1.0
        //&& game_objects.object[1].position.x == 0.7
        && game_objects.object[1].position.y == 0.0
        && game_objects.object[1].position.z == 2.0
        && game_objects.object[1].size == 1.0 {
        return vec4(1.0, 0.0, 0.0, 1.0);
    }*/

    let ndc = (fragCoord.xy / screen_size) * 2.0 - vec2(1.0, 1.0);
    let aspect = screen_size.x / screen_size.y;
    let base_dir = normalize(vec3(ndc.x * tan(cam.fov / 2.0) * aspect, ndc.y * tan(cam.fov / 2.0), 1.0));
    let dir = apply_camera_rotation(base_dir, cam.rot);

    let color = rayMarch(cam.pos + dir * 0.01, dir);
    return color;
}
