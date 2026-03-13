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

@group(0) @binding(0)
var<storage, read> game_objects : array<GameObject>;

@group(1) @binding(0)
var<uniform> screen_size : vec2<f32>;

@fragment
fn fs_main(@builtin(position) fragCoord : vec4<f32>) -> @location(0) vec4<f32> {
    var uv = fragCoord.xy / screen_size;
    return vec4(uv, 0.0, 1.0);
}
