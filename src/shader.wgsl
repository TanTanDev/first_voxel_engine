[[block]]
struct CameraUniform {
    position: vec3<f32>;
    projection_view: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> u_camera: CameraUniform;

struct VertexOutput {
    [[builtin(position)]] builtin_position: vec4<f32>;
    [[location(0)]] vex_coords: vec2<f32>;
    [[location(1)]] diffuse_color: vec3<f32>;
    [[location(2)]] normal: vec3<f32>;
    [[location(3)]] position: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] tex_coords: vec2<f32>,
    [[location(2)]] normal: vec3<f32>,
    [[location(3)]] diffuse_color: vec3<f32>,

    [[location(5)]] model_matrix_0: vec4<f32>,
    [[location(6)]] model_matrix_1: vec4<f32>,
    [[location(7)]] model_matrix_2: vec4<f32>,
    [[location(8)]] model_matrix_3: vec4<f32>,

    [[location(9)]]  normal_matrix_0: vec4<f32>,
    [[location(10)]] normal_matrix_1: vec4<f32>,
    [[location(11)]] normal_matrix_2: vec4<f32>,
    [[location(12)]] normal_matrix_3: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    let model_matrix = mat4x4<f32>(model_matrix_0, model_matrix_1, model_matrix_2, model_matrix_3);
    out.vex_coords = tex_coords;

    let normal_matrix = mat3x3<f32>(normal_matrix_0.xyz, normal_matrix_1.xyz, normal_matrix_2.xyz);

    //let normal_matrix = transpose(mat3x3<f32>(model_matrix.x.xyz, model_matrix.y.xyz, model_matrix.z.xyz));
    //let normal_matrix = mat3x3<f32>(u_camera.projection_view.x.xyz, u_camera.projection_view.y.xyz, u_camera.projection_view.z.xyz);
    out.normal = normal_matrix * normal;
    let model_space = model_matrix * vec4<f32>(position, 1.0);
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

[[group(2), binding(0)]]
var<uniform> u_light: LightUniform;

[[group(1), binding(0)]] var t_diffuse: texture_2d<f32>;
[[group(1), binding(1)]] var s_diffuse: sampler;

[[stage(fragment), early_depth_test]]
fn fs_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    let ambient_strength = 0.01;
    let ambient_color = u_light.color * ambient_strength;

    let normal = normalize(in.normal);
    let light_dir = normalize(u_light.position - in.position);
    let diffuse_strength = max(dot(normal, light_dir), 0.0);
    let diffuse_color = u_light.color * diffuse_strength;

    //let view_dir = normalize(u_camera.position - vec3<f32>(in.builtin_position.xyz));
    let view_dir = normalize(u_camera.position - in.position);
    let half_dir = normalize(view_dir + light_dir);

    let specular_strength = pow(max(dot(normal, half_dir), 0.0), 32.0);
    let specular_color = specular_strength * u_light.color;
    //let specular_color = 0.0;

    var surface_color: vec4<f32> = textureSample(t_diffuse, s_diffuse, in.vex_coords);
    surface_color = surface_color * vec4<f32>(in.diffuse_color, 1.0);

    let result = (diffuse_color) * surface_color.xyz;

    return vec4<f32>(result, surface_color.a);
    //return surface_color * vec4<f32>(light_ambient_color, 1.0);
}