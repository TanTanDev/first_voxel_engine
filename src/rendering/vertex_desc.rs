pub trait VertexDesc {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a>;
}
