use super::voxel::Voxel;

// argument-flavor struct
#[derive(Debug, Clone, Copy)]
pub struct LocalCoordinate(pub i32, pub i32, pub i32);

// dimension size
pub const SIZE: usize = 16;
// chunk size in bits (5 = 32) (4 = 16)
//pub const BIT_SIZE: i32 = 4;
use lazy_static::*;
lazy_static! {
    // when SIZE 16, BIT_SIZE is 4
    // by shifting 16 << 4 we get 1
    // we with this get indexes from the collapsed array
    pub static ref BIT_SIZE: i32 = (SIZE as f32).log2() as i32;
}

pub struct ChunkMesh {
    pub vertex_buffer: Option<generational_arena::Index>,
    pub index_buffer: Option<generational_arena::Index>,
    pub num_indices: u32,
    // debug info
    pub num_vertices: u32,
}

impl ChunkMesh {
    pub fn new() -> Self {
        Self {
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            num_vertices: 0,
        }
    }

    pub fn update_vertex_buffers(
        &mut self,
        vertex_buffer: generational_arena::Index,
        index_buffer: generational_arena::Index,
        num_indices: u32,
        num_vertices: u32,
    ) {
        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.num_indices = num_indices;
        self.num_vertices = num_vertices;
    }
}

impl lifeguard::Recycleable for ChunkMesh {
    fn new() -> Self {
        ChunkMesh::new()
    }

    fn reset(&mut self) {
        self.vertex_buffer = None;
        self.index_buffer = None;
        self.num_indices = 0u32;
    }
}

pub struct Chunk {
    pub voxels: [Voxel; SIZE * SIZE * SIZE],
}

impl lifeguard::Recycleable for Chunk {
    fn new() -> Self {
        Chunk::new()
    }

    fn reset(&mut self) {
        for voxel in self.voxels.iter_mut() {
            voxel.set_density_fraciton(0f32);
        }
    }
}

impl Chunk {
    // convert 3d coordinate to array index
    pub fn get_index(coordinate: LocalCoordinate) -> usize {
        (coordinate.2 | (coordinate.1 << *BIT_SIZE) | (coordinate.0 << (*BIT_SIZE * 2))) as usize
    }

    pub fn get_local_coordinate(index: i32) -> LocalCoordinate {
        LocalCoordinate(
            (index as f32 / (SIZE * SIZE) as f32) as i32,
            ((index as f32 / SIZE as f32) % SIZE as f32) as i32,
            (index as f32 % SIZE as f32) as i32,
        )
    }

    pub fn get_voxel(&self, coordinate: LocalCoordinate) -> Option<&Voxel> {
        let index = Self::get_index(coordinate);
        self.get_voxel_from_index(index)
    }

    #[allow(dead_code)]
    pub fn get_voxel_from_index_mut(&mut self, index: usize) -> Option<&mut Voxel> {
        self.voxels.get_mut(index).map_or(None, |v| Some(v))
    }

    pub fn get_voxel_from_index(&self, index: usize) -> Option<&Voxel> {
        //self.voxels.get(index).map_or(Voxel::new_empty(), |v| *v)
        self.voxels.get(index)
    }

    pub fn new() -> Self {
        let chunk = Self {
            voxels: [Voxel::new_empty(); SIZE * SIZE * SIZE],
        };
        chunk
    }

    pub fn build_voxel_data(&mut self, chunk_world_pos: &cgmath::Vector3<f32>) {
        use noise::{NoiseFn, Perlin, Seedable};
        let perlin = Perlin::new();
        perlin.set_seed(484);
        for (index, voxel) in self.voxels.iter_mut().enumerate() {
            let local_coord = Self::get_local_coordinate(index as i32);
            let (l_x, l_y, l_z) = (local_coord.0, local_coord.1, local_coord.2);

            // convert noise to world
            let down_scale = 0.027f64;
            let x = (chunk_world_pos.x as f64 + l_x as f64) * down_scale;
            let y = (chunk_world_pos.y as f64 + l_y as f64) * down_scale;
            let z = (chunk_world_pos.z as f64 + l_z as f64) * down_scale;
            let density = perlin.get([x, y, z]);
            if density > 0.3f64 {
                voxel.set_density_fraciton(1f32);
            }
        }
    }
}
