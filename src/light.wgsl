[[block]]
struct CameraUniform {
    position: vec3<f32>;
    projection_view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> u_camera: CameraUniform;

struct VertexOutput {
    [[builtin(position)]] builtin_position: vec4<f32>;
    [[location(0)]] color: vec2<f32>;
};


[[block]]
struct LightUniform {
    position: vec3<f32>;
    color: vec3<f32>;
};

[[group(1), binding(0)]]
var<uniform> u_light: LightUniform;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] tex_coords: vec2<f32>,
    [[location(2)]] normal: vec3<f32>,
    [[location(3)]] diffuse_color: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let scale = 0.25;
    let scaled_position = position * scale + u_light.position;
    out.builtin_position = u_camera.projection_view * vec4<f32>(scaled_position, 1.0);
    return out;
}

[[stage(fragment), early_depth_test]]
fn fs_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(u_light.color, 1.0);
}