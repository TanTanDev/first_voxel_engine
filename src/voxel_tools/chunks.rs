use anyhow::Context;
use anyhow::*;
use cgmath::InnerSpace;
use lifeguard::*;
use std::{
    collections::{HashMap, VecDeque},
};

use crate::rendering::gpu_resources::GpuResources;

use super::mesh_builder;
use super::{
    chunk::Chunk,
    rendering::voxel_rendering::{self},
};
use super::{
    chunk::{ChunkMesh, LocalCoordinate, SIZE},
    voxel::Voxel,
};

// max amount of per-chunk data we can load
pub const DEFAULT_MAX_CHUNK_DATAS: usize = 10000;
// max amount of per-chunk meshes we can load
pub const DEFAULT_MAX_MESH_DATAS: usize = 10000;
pub const RENDER_DIST_RADIUS: i32 = 8;

pub const MAX_DATA_QUEUE: usize = 16;
pub const MAX_MESH_QUEUE: usize = 16;

pub const MAX_DATA_UNLOAD_QUEUE: usize = 16;
pub const MAX_MESH_UNLOAD_QUEUE: usize = 16;

pub struct Chunks {
    // chunk_map owns the current chunks, but when unloaded puts them back to chunk_pool
    chunk_data_map: HashMap<cgmath::Vector3<i32>, Chunk>,
    chunk_mesh_map: HashMap<cgmath::Vector3<i32>, ChunkMesh>,

    // chunk data is recycled from these pools
    chunk_pool: Pool<Chunk>,
    chunk_mesh_pool: Pool<ChunkMesh>,

    // chunk data are put in queue due to heavy data processing
    chunk_data_load_queue: VecDeque<cgmath::Vector3<i32>>,
    chunk_mesh_load_queue: VecDeque<cgmath::Vector3<i32>>,

    chunk_data_unload_queue: VecDeque<cgmath::Vector3<i32>>,
    chunk_mesh_unload_queue: VecDeque<cgmath::Vector3<i32>>,

    pub position: cgmath::Vector3<f32>,

    render_distance: i32,
}

impl Chunks {
    pub fn new() -> Self {
        let chunks = Self {
            chunk_data_map: HashMap::with_capacity(DEFAULT_MAX_CHUNK_DATAS),
            chunk_mesh_map: HashMap::with_capacity(DEFAULT_MAX_MESH_DATAS),
            chunk_pool: pool().with(StartingSize(DEFAULT_MAX_CHUNK_DATAS)).build(),
            chunk_mesh_pool: pool().with(StartingSize(DEFAULT_MAX_MESH_DATAS)).build(),
            // position of chunks to load in
            chunk_data_load_queue: VecDeque::with_capacity(MAX_DATA_QUEUE),
            chunk_mesh_load_queue: VecDeque::with_capacity(MAX_MESH_QUEUE),
            chunk_data_unload_queue: VecDeque::with_capacity(MAX_DATA_QUEUE),
            chunk_mesh_unload_queue: VecDeque::with_capacity(MAX_MESH_QUEUE),
            position: cgmath::Vector3::<f32>::new(0., 0., 0.),
            render_distance: RENDER_DIST_RADIUS,
        };
        chunks
    }

    pub fn build_chunk_data_in_queue(
        &mut self,
    ) {
        while let Some(chunk_pos) = self.chunk_data_load_queue.pop_front() {
            self.build_chunk_data(chunk_pos);
        }
    }

    pub fn make_coords_valid(
        chunk_pos: &mut cgmath::Vector3<i32>,
        local_pos: &mut LocalCoordinate,
    ) {
        let chunk_size = SIZE as i32;
        while local_pos.0 < 0 {
            local_pos.0 += chunk_size;
            chunk_pos.x -= 1;
        }
        while local_pos.0 > chunk_size {
            local_pos.0 -= chunk_size;
            chunk_pos.x += 1;
        }
        while local_pos.1 < 0 {
            local_pos.1 += chunk_size;
            chunk_pos.y -= 1;
        }
        while local_pos.1 > chunk_size {
            local_pos.1 -= chunk_size;
            chunk_pos.y += 1;
        }
        while local_pos.2 < 0 {
            local_pos.2 += chunk_size;
            chunk_pos.z -= 1;
        }
        while local_pos.2 > chunk_size {
            local_pos.2 -= chunk_size;
            chunk_pos.z += 1;
        }
    }

    // if the local coordinate goes outside bounds, the adjacent chunk will be checked instead
    pub fn try_get_voxel(
        &self,
        chunk_pos: &cgmath::Vector3<i32>,
        local_pos: &LocalCoordinate,
    ) -> Result<&Voxel> {
        let mut chunk_pos = *chunk_pos;
        let mut local_pos = *local_pos;
        Self::make_coords_valid(&mut chunk_pos, &mut local_pos);

        let chunk = self.chunk_data_map.get(&chunk_pos).context("")?;
        chunk.get_voxel(local_pos).context("")
    }

    pub fn get_chunk_mesh_mut(
        &mut self,
        chunk_pos: &cgmath::Vector3<i32>,
    ) -> Option<&mut ChunkMesh> {
        self.chunk_mesh_map.get_mut(chunk_pos)
    }

    pub fn build_chunk_data(
        &mut self,
        chunk_pos: cgmath::Vector3<i32>,
    ) {
        let mut chunk = self.chunk_pool.detached();
        let chunk_world_pos = Self::chunk_to_world(chunk_pos);

        chunk.build_voxel_data(&chunk_world_pos);
        println!("loaded chunk data at world pos: {:?}", chunk_world_pos);
        self.chunk_data_map.insert(chunk_pos, chunk);
    }

    pub fn build_chunk_meshes_in_queue(
        &mut self,
        device: &wgpu::Device,
        gpu_resources: &mut GpuResources,
    ) {
        while let Some(chunk_pos) = self.chunk_mesh_load_queue.pop_front() {
            if self.chunk_mesh_map.len() >= DEFAULT_MAX_CHUNK_DATAS {
                return;
            }
            let chunk_mesh = self.chunk_mesh_pool.detached();
            self.chunk_mesh_map.insert(chunk_pos, chunk_mesh);

            println!("building chunk mesh at: {:?}", chunk_pos);
            let chunk_world_pos = Self::chunk_to_world(chunk_pos);
            if mesh_builder::build_chunk_mesh(
                self,
                device,
                gpu_resources,
                &chunk_pos,
                &chunk_world_pos,
            ) {
                // successfully built return for now, 'only one per frame'
                return;
            }
        }
    }

    pub fn is_chunk_processing(&self, chunk_pos: &cgmath::Vector3<i32>) -> bool {
        self.chunk_data_map.contains_key(chunk_pos)
            || self.chunk_data_load_queue.contains(chunk_pos)
    }

    pub fn is_mesh_processing(&self, chunk_pos: &cgmath::Vector3<i32>) -> bool {
        self.chunk_mesh_map.contains_key(chunk_pos)
            || self.chunk_mesh_load_queue.contains(chunk_pos)
    }

    pub fn chunk_to_world(chunk_pos: cgmath::Vector3<i32>) -> cgmath::Vector3<f32> {
        cgmath::Vector3::<f32>::new(
            chunk_pos.x as f32 * SIZE as f32,
            chunk_pos.y as f32 * SIZE as f32,
            chunk_pos.z as f32 * SIZE as f32,
        )
    }

    pub fn in_range(&self, chunk_pos: cgmath::Vector3<i32>) -> bool {
        // convert from i32 postion to world f32 pos
        let chunk_real_pos = Self::chunk_to_world(chunk_pos);
        let delta = self.position - chunk_real_pos;
        let distance_sq: f32 = delta.magnitude2().into();
        let render_dist = (self.render_distance as f32) * SIZE as f32;
        let render_distance_sq = render_dist * render_dist;
        distance_sq < render_distance_sq
    }

    // based on current position load all meshes
    pub fn update_load_mesh_queue(&mut self) {
        if self.chunk_mesh_map.len() >= DEFAULT_MAX_MESH_DATAS
            || self.chunk_mesh_load_queue.len() >= MAX_MESH_QUEUE
        {
            return;
        }
        for y in -self.render_distance..self.render_distance {
            //for y in 0..1 {
            for z in -self.render_distance..self.render_distance {
                for x in -self.render_distance..self.render_distance {
                    let current_chunk_pos = cgmath::Vector3::<i32>::new(
                        (self.position.x / SIZE as f32) as i32,
                        (self.position.y / SIZE as f32) as i32,
                        (self.position.z / SIZE as f32) as i32,
                    );
                    let chunk_pos = current_chunk_pos + cgmath::Vector3::<i32>::new(x, y, z);

                    // chunk is already being loaded, or is loaded
                    let is_mesh_proccessing = self.is_mesh_processing(&chunk_pos);
                    if is_mesh_proccessing {
                        continue;
                    }

                    let in_range = self.in_range(current_chunk_pos);
                    // check if adjacent chunks are loaded

                    use cgmath::Vector3 as vec;
                    // check if all adjacent chunks data are loaded
                    let adj_chunk_data_bad = [
                        -vec::<i32>::unit_x(),
                        vec::<i32>::unit_x(),
                        -vec::<i32>::unit_y(),
                        vec::<i32>::unit_y(),
                        -vec::<i32>::unit_z(),
                        vec::<i32>::unit_z(),
                    ]
                    .iter_mut()
                    .map(|v| *v + chunk_pos)
                    .any(|v| !self.chunk_data_map.contains_key(&v));

                    // queue chunk for mesh creation
                    if in_range && !adj_chunk_data_bad {
                        self.chunk_mesh_load_queue.push_back(chunk_pos);
                        if self.chunk_mesh_load_queue.len() >= MAX_MESH_QUEUE {
                            return;
                        }
                    }
                }
            }
        }
    }

    pub fn update_unload_data_queue(&mut self) {
        let current_chunk_pos = cgmath::Vector3::<i32>::new(
            (self.position.x / SIZE as f32) as i32,
            (self.position.y / SIZE as f32) as i32,
            (self.position.z / SIZE as f32) as i32,
        );
        // find currently loaded meshes positions not contained in range
        // BOX BOUND CHECK IS FAST
        let outside = self
            .chunk_mesh_map
            .iter()
            .filter(|(p, _m)| {
                p.x < current_chunk_pos.x - self.render_distance
                    || p.x > current_chunk_pos.x + self.render_distance
                    || p.y < current_chunk_pos.y - self.render_distance
                    || p.y > current_chunk_pos.y + self.render_distance
                    || p.z < current_chunk_pos.z - self.render_distance
                    || p.z > current_chunk_pos.z + self.render_distance
            })
            .map(|(p, _m)| p)
            .collect::<Vec<_>>();

        for chunk_pos in outside {
            // already proccessing skip
            if self.chunk_data_unload_queue.contains(chunk_pos) {
                continue;
            }
            println!("queueing chunk for data unload: {:?}", chunk_pos);
            self.chunk_data_unload_queue.push_back(*chunk_pos);
            if self.chunk_data_unload_queue.len() >= MAX_DATA_UNLOAD_QUEUE {
                return;
            }
        }
    }

    // based on current position load all meshes
    pub fn update_unload_mesh_queue(&mut self) {
        let current_chunk_pos = cgmath::Vector3::<i32>::new(
            (self.position.x / SIZE as f32) as i32,
            (self.position.y / SIZE as f32) as i32,
            (self.position.z / SIZE as f32) as i32,
        );
        // find currently loaded meshes positions not contained in range
        // BOX BOUND CHECK IS FAST
        let outside = self
            .chunk_mesh_map
            .iter()
            .filter(|(p, _m)| {
                p.x < current_chunk_pos.x - self.render_distance
                    || p.x > current_chunk_pos.x + self.render_distance
                    || p.y < current_chunk_pos.y - self.render_distance
                    || p.y > current_chunk_pos.y + self.render_distance
                    || p.z < current_chunk_pos.z - self.render_distance
                    || p.z > current_chunk_pos.z + self.render_distance
            })
            .map(|(p, _m)| p)
            .collect::<Vec<_>>();

        for chunk_pos in outside {
            // already proccessing skip
            if self.chunk_mesh_unload_queue.contains(chunk_pos) {
                continue;
            }
            self.chunk_mesh_unload_queue.push_back(*chunk_pos);
            println!("queueing chunk for mesh unload: {:?}", chunk_pos);
            if self.chunk_mesh_unload_queue.len() >= MAX_MESH_UNLOAD_QUEUE {
                return;
            }
        }
    }

    pub fn unload_data_queue(&mut self) {
        while let Some(chunk_pos) = self.chunk_data_unload_queue.pop_front() {
            // detach chunk data
            if let Some(chunk_data) = self.chunk_data_map.remove(&chunk_pos) {
                println!("unloading data at: {:?}", chunk_pos);
                self.chunk_pool.attach(chunk_data);
            }
        }
    }

    // generate meshes queued up
    pub fn unload_mesh_queue(&mut self, gpu_resources: &mut GpuResources) {
        while let Some(chunk_pos) = self.chunk_mesh_unload_queue.pop_front() {
            // detach mesh data
            if let Some(chunk_mesh) = self.chunk_mesh_map.remove(&chunk_pos) {
                println!("unloading mesh at: {:?}", chunk_pos);
                if let Some(v_buf_key) = chunk_mesh.vertex_buffer {
                    if let Some(v_buffer) = gpu_resources.buffer_arena.get_mut(v_buf_key) {
                        v_buffer.destroy();
                    }
                    gpu_resources.buffer_arena.remove(v_buf_key);
                }
                if let Some(i_buf_key) = chunk_mesh.index_buffer {
                    if let Some(i_buffer) = gpu_resources.buffer_arena.get_mut(i_buf_key) {
                        i_buffer.destroy();
                    }
                    gpu_resources.buffer_arena.remove(i_buf_key);
                }
                self.chunk_mesh_pool.attach(chunk_mesh);
            }
        }
    }

    // based on current position load all meshes
    pub fn update_load_data_queue(&mut self) {
        if self.chunk_data_map.len() >= DEFAULT_MAX_CHUNK_DATAS
            || self.chunk_data_load_queue.len() >= MAX_DATA_QUEUE
        {
            return;
        }
        for y in -self.render_distance..self.render_distance {
            //for y in 0..1 {
            for z in -self.render_distance..self.render_distance {
                for x in -self.render_distance..self.render_distance {
                    let current_chunk_pos = cgmath::Vector3::<i32>::new(
                        (self.position.x / SIZE as f32) as i32,
                        (self.position.y / SIZE as f32) as i32,
                        (self.position.z / SIZE as f32) as i32,
                    );
                    let chunk_pos = current_chunk_pos + cgmath::Vector3::<i32>::new(x, y, z);

                    // chunk is already being loaded, or is loaded
                    let is_chunk_proccessing = self.is_chunk_processing(&chunk_pos);
                    if is_chunk_proccessing {
                        continue;
                    }

                    let in_range = self.in_range(current_chunk_pos);
                    if in_range {
                        // load chunk
                        self.chunk_data_load_queue.push_back(chunk_pos);
                    }
                    // check if we don't wan to load any more
                    if self.chunk_data_load_queue.len() >= MAX_DATA_QUEUE {
                        println!("done");
                        return;
                    }
                }
            }
        }
    }

    pub fn draw<'a, '_b>(
        &mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        camera_bind_group: &'a wgpu::BindGroup,
        light_bind_group: &'a wgpu::BindGroup,
        gpu_resources: &'a GpuResources,
    ) -> anyhow::Result<()> {
        for (_pos, chunk_mesh) in self.chunk_mesh_map.iter() {
            let vertex_buffer_index = chunk_mesh.vertex_buffer.as_ref().context("no vertices")?;
            let index_buffer_index = chunk_mesh.index_buffer.as_ref().context("no indices")?;
            let num_indices = chunk_mesh.num_indices;
            let vertex_buffer = gpu_resources
                .buffer_arena
                .get(*vertex_buffer_index)
                .context("no vertex buf")?;
            let index_buffer = gpu_resources
                .buffer_arena
                .get(*index_buffer_index)
                .context("no vertex buf")?;
            let _ = voxel_rendering::draw_chunk(
                render_pass,
                num_indices,
                camera_bind_group,
                light_bind_group,
                vertex_buffer,
                index_buffer,
            );
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_vertex_count(&self) -> u32 {
        self.chunk_mesh_map
            .iter()
            .map(|(_i, m)| m.num_vertices)
            .sum()
    }
}
pub fn adjacent_voxels<'a>(
    chunks: &'a mut Chunks,
    local_pos: (i32, i32, i32),
    chunk_pos: &cgmath::Vector3<i32>,
) -> Result<(&'a Voxel, &'a Voxel, &'a Voxel, &'a Voxel)> {
    let (x, y, z) = (local_pos.0, local_pos.1, local_pos.2);
    let voxel = chunks
        .try_get_voxel(chunk_pos, &LocalCoordinate(x, y, z))
        .context("no voxel")?;
    let back = chunks
        .try_get_voxel(chunk_pos, &LocalCoordinate(x, y, z - 1))
        .context("no back voxel")?;
    let left = chunks
        .try_get_voxel(chunk_pos, &LocalCoordinate(x - 1, y, z))
        .context("no left voxel")?;
    let down = chunks
        .try_get_voxel(chunk_pos, &LocalCoordinate(x, y - 1, z))
        .context("no down voxel")?;
    Ok((voxel, back, left, down))
}
