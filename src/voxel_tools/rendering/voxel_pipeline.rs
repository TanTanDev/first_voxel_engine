use crate::{
    create_render_pipeline,
    rendering::{render_utils, vertex_desc::VertexDesc},
    texture,
    voxel_tools::rendering::voxel_vertex::VoxelVertex,
};

pub fn create_voxel_pipeline(
    device: &wgpu::Device,
    texture_format: wgpu::TextureFormat,
    light_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let visibility = wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT;
    let camera_bind_group_layout =
        render_utils::create_bind_group_layout(&device, "camera_bind_layout", 0, visibility);

    let shader_module = render_utils::create_shader_module(
        &device,
        include_str!("voxel.wgsl"),
        "voxel_shader_module",
    );

    let bind_group_layouts = &[&camera_bind_group_layout, &light_bind_group_layout];
    let pipeline_layout =
        render_utils::create_pipeline_layout(&device, "voxel_pipeline", bind_group_layouts);

    println!("creating pipeline");
    let render_pipeline = create_render_pipeline(
        &device,
        &pipeline_layout,
        texture_format,
        Some(texture::Texture::DEPTH_FORMAT),
        &[VoxelVertex::desc()],
        shader_module,
        "voxel_pipeline",
    );
    render_pipeline
}
