use crate::rendering::gpu_resources::GpuResources;

use super::{
    chunk,
    chunks::{adjacent_voxels, Chunks},
};
use super::{
    direction::Direction,
    quad::Quad,
    rendering::voxel_vertex::VoxelVertex,
    voxel::Voxel,
};
use wgpu::util::DeviceExt;

pub fn build_chunk_mesh(
    chunks: &mut Chunks,
    //chunk: &mut Chunk,
    device: &wgpu::Device,
    gpu_resources: &mut GpuResources,
    chunk_pos: &cgmath::Vector3<i32>,
    chunk_world_pos: &cgmath::Vector3<f32>,
) -> bool {
    let chunk_size = chunk::SIZE as i32;
    let mut quads = Vec::<Quad>::new();
    for x in 0..chunk_size {
        for y in 0..chunk_size {
            for z in 0..chunk_size {
                let voxel_pos_local = cgmath::Vector3::<f32>::new(x as f32, y as f32, z as f32);
                let voxel_pos_world = chunk_world_pos + voxel_pos_local;
                if let Ok((voxel, back, left, down)) = adjacent_voxels(chunks, (x, y, z), chunk_pos)
                {
                    process_voxel(&voxel, voxel_pos_world, &left, &down, &back, &mut quads);
                }
            }
        }
    }
    if quads.is_empty() {
        //panic!("ouuuuh woops");
    }
    let mut voxel_vertices = Vec::<VoxelVertex>::new();
    let mut indices = Vec::<u32>::new();
    let mut vert_index = 0;
    for quad in quads {
        let normal = quad.direction.get_normal();
        (0..4).for_each(|index| {
            voxel_vertices.push(VoxelVertex {
                position: quad.corners[index].into(),
                normal: normal.into(),
                color_diffuse: quad.color.into(),
            });
        });
        indices.push(vert_index);
        indices.push(vert_index + 1);
        indices.push(vert_index + 2);
        indices.push(vert_index);
        indices.push(vert_index + 2);
        indices.push(vert_index + 3);
        vert_index += 4;
    }
    if let Some(chunk_mesh) = chunks.get_chunk_mesh_mut(chunk_pos) {
        let num_indices = indices.len() as u32;
        let num_vertices = voxel_vertices.len() as u32;
        let (v_buf, i_buf) = construct_buffers(device, voxel_vertices, indices);
        let v_buf = gpu_resources.buffer_arena.insert(v_buf);
        let i_buf = gpu_resources.buffer_arena.insert(i_buf);
        chunk_mesh.update_vertex_buffers(v_buf, i_buf, num_indices, num_vertices);
        return num_vertices != 0;
    }
    false
}

fn process_voxel(
    voxel: &Voxel,
    voxel_pos: cgmath::Vector3<f32>,
    left: &Voxel,
    down: &Voxel,
    back: &Voxel,
    quads: &mut Vec<Quad>,
) {
    match voxel.is_solid() {
        true => {
            // voxel is solid
            if !left.is_solid() {
                quads.push(Quad::from_direction(Direction::Left, voxel_pos));
            }
            if !down.is_solid() {
                quads.push(Quad::from_direction(Direction::Down, voxel_pos));
            }
            if !back.is_solid() {
                quads.push(Quad::from_direction(Direction::Back, voxel_pos));
            }
        }
        false => {
            // voxel is not solid
            if left.is_solid() {
                quads.push(Quad::from_direction(Direction::Right, voxel_pos));
            }
            if down.is_solid() {
                quads.push(Quad::from_direction(Direction::Up, voxel_pos));
            }
            if back.is_solid() {
                quads.push(Quad::from_direction(Direction::Forward, voxel_pos));
            }
        }
    }
}

fn construct_buffers(
    device: &wgpu::Device,
    vertices: Vec<VoxelVertex>,
    indices: Vec<u32>,
) -> (wgpu::Buffer, wgpu::Buffer) {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("voxel_chunk_vertices"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsage::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("voxel_chunk_indices"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsage::INDEX,
    });
    (vertex_buffer, index_buffer)
}
