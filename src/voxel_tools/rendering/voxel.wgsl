[[block]]
struct CameraUniform {
    position: vec3<f32>;
    projection_view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> u_camera: CameraUniform;

struct VertexOutput {
    [[builtin(position)]] builtin_position: vec4<f32>;
    [[location(1)]] diffuse_color: vec3<f32>;
    [[location(2)]] normal: vec3<f32>;
    [[location(3)]] position: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] normal: vec3<f32>,
    [[location(2)]] diffuse_color: vec3<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.normal = normal;
    let model_space = vec4<f32>(position, 1.0);
    out.position = model_space.xyz;
    out.diffuse_color = diffuse_color;

    out.builtin_position = u_camera.projection_view * model_space;
    return out;
}


[[block]]
struct LightUniform {
    position: vec3<f32>;
    color: vec3<f32>;
};

[[group(1), binding(0)]]
var<uniform> u_light: LightUniform;

[[stage(fragment), early_depth_test]]
fn fs_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    let ambient_strength = 0.2;
    let ambient_color = u_light.color * ambient_strength;

    let normal = normalize(in.normal);
    let light_dir = normalize(u_light.position - in.position);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_color = u_light.color * diffuse_strength;

    let view_dir = normalize(u_camera.position - in.position);
    let half_dir = normalize(view_dir + light_dir);

    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * u_light.color;

    let surface_color = vec4<f32>(in.diffuse_color, 1.0);

    let result = (diffuse_color + ambient_color + specular_color) * surface_color.xyz;

    return vec4<f32>(result, surface_color.a);
}