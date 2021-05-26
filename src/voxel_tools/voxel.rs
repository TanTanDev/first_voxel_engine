#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    density: u8,
}

// decorative argument type
pub struct Density(u8);

impl Voxel {
    pub fn new(density: Density) -> Self {
        Self { density: density.0 }
    }

    pub fn new_empty() -> Self {
        Self::new(Density(0u8))
    }

    #[allow(dead_code)]
    pub fn new_solid() -> Self {
        Self::new(Density(255u8))
    }

    pub fn is_solid(&self) -> bool {
        self.density > 0u8
    }

    #[allow(dead_code)]
    pub fn density_fraction(&self) -> f32 {
        self.density as f32 / 255f32
    }

    pub fn set_density_fraciton(&mut self, fraction: f32) {
        self.density = (fraction * 255f32) as u8;
    }
}
