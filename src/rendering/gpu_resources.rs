use generational_arena::*;
pub struct GpuResources {
    pub buffer_arena: Arena<wgpu::Buffer>,
}

impl GpuResources {
    pub fn new() -> Self {
        Self {
            buffer_arena: Arena::with_capacity(32),
        }
    }
}
