struct VertexOutput {
    [[builtin(position)]] builtin_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] tex_coords: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = tex_coords;
    out.builtin_position = vec4<f32>(position, 1.0);
    return out;
}

[[group(0), binding(0)]]
var t_depth: texture_depth_2d;

[[group(0), binding(1)]]
var s_depth: sampler_comparison;

[[stage(fragment), early_depth_test]]
fn fs_main(
    in: VertexOutput,
) -> [[location(0)]] vec4<f32> {
    let near = 0.1;
    let far = 100.0;
    let depth = textureSampleCompare(t_depth, s_depth, in.tex_coords, 1.0);
    let r = (2.0 * near * far) / (far + near - depth * (far - near));
    return vec4<f32>(vec3<f32>(r), 1.0);
}