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

fn scene_sdf(p: vec3<f32>) -> f32 {
    var min_dist: f32 = 1000000.0;

    for (var i = 0; i < game_objects.length; i++) {
        let obj = game_objects.object[i];
        let d = sdSphere(p - obj.position, obj.size);

        if d < min_dist {
            min_dist = d;
        }
    }

    return min_dist;
}

fn approximate_normal(p: vec3<f32>) -> vec3<f32> {
    let eps: f32 = 0.001;

    let dx = scene_sdf(p + vec3(eps, 0.0, 0.0)) - scene_sdf(p - vec3(eps, 0.0, 0.0));
    let dy = scene_sdf(p + vec3(0.0, eps, 0.0)) - scene_sdf(p - vec3(0.0, eps, 0.0));
    let dz = scene_sdf(p + vec3(0.0, 0.0, eps)) - scene_sdf(p - vec3(0.0, 0.0, eps));

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
    let ndc = (fragCoord.xy / screen_size) * 2.0 - vec2(1.0, 1.0);
    let aspect = screen_size.x / screen_size.y;
    let base_dir = normalize(vec3(ndc.x * tan(cam.fov / 2.0) * aspect, ndc.y * tan(cam.fov / 2.0), 1.0));
    let dir = apply_camera_rotation(base_dir, cam.rot);

    let color = rayMarch(cam.pos + dir * 0.01, dir);
    return color;
}
