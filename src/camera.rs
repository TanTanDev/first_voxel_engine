use cgmath::{Angle, InnerSpace, Rad, Vector3};
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Uniform {
    view_position: [f32; 4],
    view_proj: [[f32; 4]; 4],
}

impl Uniform {
    fn new() -> Self {
        // needed to access ::identity()
        use cgmath::SquareMatrix;
        Self {
            view_position: [0.0; 4],
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, eye: &cgmath::Point3<f32>, view_proj: cgmath::Matrix4<f32>) {
        // We don't specifically need homogeneous coordinates since we're just using
        // a vec3 in the shader. We're using Point3 for the camera.eye, and this is
        // the easiest way to convert to Vector4. We're using Vector4 because of
        // the uniforms 16 byte spacing requirement
        self.view_position = eye.to_homogeneous().into();
        self.view_proj = view_proj.into();
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub yaw: Rad<f32>,
    pub pitch: Rad<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub uniform: Uniform,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: (0., 1., 2.).into(),
            yaw: Rad::<f32>(-90f32),
            pitch: Rad::<f32>(-20f32),
            aspect,
            fovy: 45.0,
            z_near: 0.1,
            z_far: 500.0,
            uniform: Uniform::new(),
        }
    }

    pub fn update_uniform(&mut self) {
        self.uniform
            .update(&self.position, self.build_view_projection_matrix());
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_to_rh(
            self.position,
            Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.sin()).normalize(),
            Vector3::unit_y(),
        );
        let proj =
            cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.z_near, self.z_far);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
