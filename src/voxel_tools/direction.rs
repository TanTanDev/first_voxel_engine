pub enum Direction {
    Left,
    Right,
    Down,
    Up,
    Back,
    Forward,
}

impl Direction {
    pub fn get_normal(&self) -> cgmath::Vector3<f32> {
        match self {
            Direction::Left => -cgmath::Vector3::<f32>::unit_x(),
            Direction::Right => cgmath::Vector3::<f32>::unit_x(),
            Direction::Down => -cgmath::Vector3::<f32>::unit_y(),
            Direction::Up => cgmath::Vector3::<f32>::unit_y(),
            Direction::Back => -cgmath::Vector3::<f32>::unit_z(),
            Direction::Forward => cgmath::Vector3::<f32>::unit_z(),
        }
    }
}
