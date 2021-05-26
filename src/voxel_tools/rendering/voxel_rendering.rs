use anyhow::*;

pub fn draw_chunk<'a, 'b>(
    render_pass: &mut wgpu::RenderPass<'a>,
    num_indices: u32,
    camera_u: &'a wgpu::BindGroup,
    light_u: &'a wgpu::BindGroup,
    vertex_buffer: &'a wgpu::Buffer,
    index_buffer: &'a wgpu::Buffer,
) -> Result<()> {
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
    render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
    render_pass.set_bind_group(0, &camera_u, &[]);
    render_pass.set_bind_group(1, &light_u, &[]);
    render_pass.draw_indexed(0..num_indices, 0, 0..1);
    Ok(())
}
