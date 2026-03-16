@vertex
fn vs_main(@builtin(vertex_index) i : u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>,3>(
        vec2(-1.0,-1.0),
        vec2( 3.0,-1.0),
        vec2(-1.0, 3.0)
    );
    return vec4(pos[i],0.0,1.0);
}

struct GameObject {
    position : vec3<f32>,
    size : f32,
    color : vec4<f32>,
}

const PI: f32 = 3.14159265359;

const fov = PI / 2.0;
const cam_origin = vec3(0.0, 0.0, 0.0);
const cam_direction = vec3(0.0, 0.0, 1.0); //normal

@group(0) @binding(0)
var<storage, read> game_objects : array<GameObject>;

@group(1) @binding(0)
var<uniform> screen_size : vec2<f32>;

fn sdSphere(x : vec3<f32>, radius : f32) -> f32 {
    return length(x) - radius;
}

fn rayMarch(origin : vec3<f32>, position : vec3<f32>, direction : vec3<f32>, radius : f32) -> vec4<f32> {
    let maxIterations: i32 = 100;
    let eps: f32 = 0.001;
    var dist: f32 = 0.0;

    for(var i = 0; i < maxIterations; i++) {
        let point = origin + direction * dist;

        var radius = sdSphere(position - point, radius);
        if(radius < eps) {
            let cos_angle = dot(aproximative_normal(point, radius), cam_direction);
            let brightness = clamp(cos_angle, 0.0, 1.0);
            //let color = vec3(abs(dist)) + brightness * vec3(1.0, 0.0, 0.0);
            let color = vec3(1.0 - abs(dist)) * brightness;

            return vec4(color, 1.0);
        }
        dist += radius;
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
fn fs_main(@builtin(position) fragCoord : vec4<f32>) -> @location(0) vec4<f32> {
    let ndc = (fragCoord.xy / screen_size) * 2.0 - vec2(1.0, 1.0);
    let aspect = screen_size.x / screen_size.y;
    let dir = normalize(vec3(ndc.x * tan(fov/2.0) * aspect, ndc.y * tan(fov/2.0), 1.0));

    let radius = 0.4;


    let color = rayMarch(cam_origin, vec3(0.0, 0.0, 0.5), dir, radius);
    return color;
}
